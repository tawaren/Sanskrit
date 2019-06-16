//! This is an extension of the linear stack that specializes the value to a type and adds some functionality
//!
//! In addition to tracking elements this stack does track frames/blocks.
//!  The difference is that frames can contain multiple branches meaning the split the control flow, where blocks do not
//!  Both Frames and Branches open a separated scope that when closed discards all its elements from the stack except for the explicitly returned
//!   The stack ensures that elements are only dropped if they have been consumed and are not borrowed
//!   When a borrowing element is dropped it releases its borrow and may make the borrowed element available again (as soon as all borrows are released)
//!  The stack ensures that each Branch of a Frame has the same effect of the stack such as the stack state is deterministically known when the frame finishes
//!
//! At the end the stack allows to check if a series of operations on it result in the signature of a Function

use sanskrit_common::errors::*;
use sanskrit_core::model::resolved::*;
use sanskrit_common::linear_stack::*;
use alloc::collections::BTreeSet;
use sanskrit_common::model::ValueRef;
use sanskrit_core::utils::Crc;
use alloc::vec::Vec;
use core::mem;


#[derive(Default)]
pub struct LinearTypeStack {
    stack:Vec<Elem<Crc<ResolvedType>, Vec<usize>>>,     //The actual Elems
    frames:Vec<Frame>,                                  //Frame borders, to record consumes in branches
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ExprResult<'a>{
    Return(&'a[ValueRef]),      //Defines the elements returned from a block (bernouli indexes)
    Throw,                      // Tells that the block throws
}

struct Frame{
    start_index:usize,          //Where does the frame start
    consumes:BTreeSet<usize>    //what element does the frame consume form outside the frame (is popolated over time)
}

//Information allowing type checker to manage frames and blocks
//it is a route through
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct BlockInfo(usize);                                    // usize is Block Start Index
type PreviousReturns = Vec<Elem<Crc<ResolvedType>,Vec<usize>>>; // Vec<Elem> are the elems returned from a previous branch

pub struct BranchInfo{
    results:Option<(PreviousReturns, BTreeSet<usize>)>         // BTreeSet<usize> are the cross frame consumes from a previous branch (absolute indexes)
}

//migration structure
#[derive(Clone, Copy, Eq, PartialEq)]
enum Migrate {
    Free,
    Move(usize)
}

//Impl of BasStack for TypeStack
impl LinearStack<Crc<ResolvedType>,Vec<usize>> for LinearTypeStack {
    fn generate_vec(&self, elems: usize) -> Result<Vec<usize>> {
        Ok(Vec::with_capacity(elems))
    }

    //Returns the stack depth. Important to later be able to drop a frame
    fn stack_depth(&self) -> usize {
        self.stack.len()
    }

    //gets the element ignoring consume tests
    fn get_elem(&self, index:ValueRef) -> Result<&Elem<Crc<ResolvedType>,Vec<usize>>>{
        // Get the index (as we not consuming it it is safe to access non-locals they are not consumed)
        let res = self.absolute_index(index)?;
        // Get the element
        let elem = &self.stack[res];
        //Return it
        Ok(elem)
    }

    fn get_elem_absolute(&mut self, index: usize) -> Result<&mut Elem<Crc<ResolvedType>,Vec<usize>>> {
        if index >= self.stack.len() {return out_of_range_stack_addressing()}
        Ok(&mut self.stack[index])
    }

    fn push_elem(&mut self, elem: Elem<Crc<ResolvedType>,Vec<usize>>) -> Result<()> {
        self.stack.push(elem);
        Ok(())
    }

    //gets an element and consumes it
    fn consume(&mut self, index:ValueRef) -> Result<()>{
        // Get the index (as we consume it is not safe to access non-locals as they are modified)
        //get the absolute index
        let res = self.absolute_index(index)?;
        //as we consume the elem we have to record the acccess on each frame border
        //Start with first Frame
        let mut frame_index = self.frames.len() -1;
        let mut frame = &mut self.frames[frame_index];
        //while the consumed elem is outside the border continue
        while frame.start_index > res {
            //enter the cross border consumption
            frame.consumes.insert(res);
            //Next frame
            frame_index -= 1;
            //could be a function param which are not in a frame
            if self.frames.is_empty() { break; }
            frame = &mut self.frames[frame_index];
        }
        //Consume the stored one
        self.stack[res].consume()
    }
}

//LinearTypeStack Specific stack
impl LinearTypeStack {
    //Generates an empty Stack
    pub fn new() -> Self {
        LinearTypeStack { stack: vec![], frames:vec![Frame{ start_index: 0, consumes: BTreeSet::new() }] }
    }

    //Cleans up a frame by freeing or returning each parameter
    // it does additionally adapt the borrow indexes of the returned elements to account for the dropped ones
    fn ret(&mut self, start_index:usize, rets:&[ValueRef]) -> Result<()> {
        //Find out how many elements the frame has accumulated
        let frame_size = self.stack_depth()-start_index;
        //Make a vector that records from where to where returned elements are moved
        //  The index is from where and the entry to where (in case of Move, in case of Free it is not returned)
        let mut migrate:Vec<Migrate> = vec![Migrate::Free;frame_size];

        //initialize the migration vector with moves
        for (targ,val) in rets.iter().enumerate() {
            // find the absolute index of the returned element
            let index = self.absolute_index(*val)?;
            //an element can only be returned if it is contained in the current frame
            // Note: with an explicit Fetch or Copy this is still possible by moving it first into the frame
            if index < start_index {
                return elem_out_of_frame()
            }
            // find out to which index this correspond in the frame
            let frame_index = index-start_index;
            // check that the migration entry still free, if not this means the same element is returned twice
            if migrate[frame_index] != Migrate::Free  {return consuming_moved_error()}
            // Make the migration entry (they will end up on top of the stack in order)
            migrate[frame_index] = Migrate::Move(targ+start_index);
        }

        //ensure that the frame is not to big
        assert!(frame_size <= u16::max_value() as usize);

        // Free everything that will not be returned & return the rest
        //  It will recalculate indexes
        for i in 0..frame_size {
            //start at the newest in the frame
            let index = self.absolute_index(ValueRef(i as u16))?;
            // find out to which index this correspond in the frame
            let frame_index = index-start_index;
            //check if returned or freed
            match migrate[frame_index] {
                //Ensure that it is already freed
                Migrate::Free => self.stack[index].ensure_freed(false)?,
                //Adapt the moved elem
                Migrate::Move(elem_pos) => {
                    //if the returned is borrowed, we may need to change borrow index to the new position
                    if !self.stack[index].status.borrowing.is_empty() {
                        let borrow_len = self.stack[index].status.borrowing.len();
                        let mut new_borrows = Vec::with_capacity(borrow_len);
                        //adapt each borrowed if necessary
                        for b in &self.stack[index].status.borrowing {
                            //check if it borrows from inside or outside the current frame
                            if *b < start_index {
                                //borrows from outside
                                //as elem pos is > free_border we can keep it as it is
                                new_borrows.push(*b);
                            } else {
                                //borrows from inside
                                //check where it will end up
                                match migrate[*b - start_index] {
                                    //we can not borrow something that is not returned (as it will go out of scope)
                                    Migrate::Free => {return free_error()}
                                    //if we borrow from a return the position has to be changed
                                    Migrate::Move(pos) => {
                                        // Enforce borrowing order ( can not borrow from an element deeper on the stack )
                                        if pos >= elem_pos { return borrow_order_violation(); }
                                        //record the adapted borrow
                                        new_borrows.push(pos);
                                    }
                                }
                            }
                        }
                        //Set the new borrows (Sort them so post state checks pass)
                        new_borrows.sort();
                        self.stack[index].status.borrowing = new_borrows;
                    }
                    //Make sure we can return it  (meaning it is not consumed, except if locked - in that case the locker is returned as well)
                    //if it is locked, the locker is returned as well (this is guaranteed, as everything is returned or ensured to be freed (freed do not borrow))
                    if self.stack[index].status.consumed && self.stack[index].status.locked == 0 {
                        return consumed_cannot_be_returned_error()
                    }
                }
            }
        }

        let mut returns = Vec::with_capacity(rets.len());
        //Capture the returned items (need to be repushed after the frame is dropped)
        for val in rets {
            // find the absolute index of the returned element
            let index = self.absolute_index(*val)?;
            // save the element
            returns.push(self.stack[index].clone())
        }

        //Clean up the stack by removing all entries in the current frame (
        let new_len = self.stack.len() - frame_size;
        self.stack.truncate(new_len);


        //push the returns back onto the stack
        self.stack.extend(returns);

        Ok(())
    }

    //exit frame with error
    fn exit(&mut self, start_index:usize) -> Result<()>{
        //Find out how many elements the frame has accumulated
        let frame_size = self.stack_depth()-start_index;
        //check that the frame is not to big
        assert!(frame_size <= u16::max_value() as usize);
        // Free everything
        for i in 0..frame_size {
            //we force the free as the elems can be consumed or unconsumed
            // but free is still necessary to ensure the locks are lifted
            // and it ensures that no locked element is freed
            // this ensures that num locks == num borrows still holds
            self.free_internal(ValueRef(i as u16), true)?;
        }

        //Clean up the stack by removing all elements (they are no longer needed (rollback))
        let new_len = self.stack.len() - frame_size;
        self.stack.truncate(new_len);

        Ok(())
    }

    //Decides based on the result of an expresion if a return or an exit has to happen and returns false if an exit happened
    fn ret_or_exit(&mut self, start_index:usize, ret:&ExprResult) -> Result<bool>{
        //find out if we have to throw or to return and indicate it over the returned bool
        match ret {
            //We can return
            ExprResult::Return(rets) => {
                //do the return
                self.ret(start_index, rets)?;
                Ok(true)
            },
            //We have to throw
            ExprResult::Throw => {
                //do the exit
                self.exit(start_index)?;
                Ok(false)
            },
        }
    }

    pub fn steal(&mut self, src:ValueRef, trg:ValueRef) -> Result<()>{
        let src_index:usize = self.absolute_index(src)?;
        let trg_index:usize = self.absolute_index(trg)?;
        //an alternative would be clone but that would alloc and this does not
        let mut old = mem::replace(&mut self.stack[src_index].status.borrowing, Vec::with_capacity(0));
        let repl = self.stack[trg_index].status.borrowing.slice();
        if old.is_empty() || repl.is_empty() { return steal_violation() }

        let mut borrow_dedup = BTreeSet::new();
        let mut new_borrows = Vec::with_capacity(old.len()); //this suffices in most cases
        for n in repl {
            assert_eq!(borrow_dedup.insert(*n), true); // repl on itself should already be deduped
            new_borrows.push(*n);
        }
        //seperate loop because first loop borrows immutable
        for n in &new_borrows { self.stack[*n].lock(1,false)?; }
        let mut found = false;
        for b in old{
            if b == trg_index {
                //ups it was multiple times in the set
                if found {return steal_violation()}
                found = true;
            } else {
                if !borrow_dedup.contains(&b) {
                    //borrow_dedup.insert(*b): old on itself should already be deduped
                    new_borrows.push(b);
                } else {
                    //we already have it so it is locked one time to many
                    &self.stack[b].unlock()?;
                }
            }
        }
        if !found {return steal_violation()}
        self.stack[trg_index].unlock()?;
        self.stack[src_index].status.borrowing = new_borrows;

        if self.stack[trg_index].status.locked == 0 {
            self.free_internal(trg, false)?
        }

        Ok(())
    }

    //open a new basic frame which does not branch and thus allowing it to consume outside elements
    pub fn start_block(&mut self) -> BlockInfo {
        //Remember the blocks start (needed for ret later)
        BlockInfo(self.stack_depth())
    }

    //open a new frame which does branch, it also starts the first branch immediately
    //the returned branch info then can be used to switch to the next branch or ending the branching
    pub fn start_branching(&mut self) -> BranchInfo {
        //Remember the frames start
        let start_index = self.stack_depth();

        //Create the new frame and push it
        self.frames.push(Frame{
            start_index,
            consumes:BTreeSet::new(),
        });
        BranchInfo{ results: None}
    }

    //Helper to exit/ret a branch and check the returns (must be the same)
    fn ret_or_exit_branch(&mut self, br:&mut BranchInfo, rets: ExprResult, frame:Frame) -> Result<bool> {
        //Do the return or the exit
        let is_res = if self.ret_or_exit(frame.start_index, &rets)? {
            //if this was a normal return check the result to be consistent
            if let ExprResult::Return(res) = rets {
                //check that the return is in bound
                assert!(res.len() <= u16::max_value() as usize);
                match br {
                    //We have our first branch result (previous branches throw or this is the initial)
                    // Capture the Information of this branch run for the remaining ones
                    BranchInfo{results:ref mut inner,.. } if inner.is_none() => *inner = Some((
                        //Calculate the returned elements
                        (0..res.len()).rev().map(|new| Ok(self.get_elem(ValueRef(new as u16))?.clone())).collect::<Result<Vec<Elem<Crc<ResolvedType>,Vec<usize>>>>>()?,
                        //Captures the consumes
                        frame.consumes
                    )),
                    BranchInfo{results:Some((old_res, old_captures)), ..} => {
                        //Compare the old elems with the new one
                        for (old, new) in old_res.iter().zip((0..res.len()).rev()) {
                            //Note: The Vec<usize>s in the were ret  -- so this is ok
                            if old != self.get_elem(ValueRef(new as u16))? {
                                return branch_ret_mismatch();
                            }
                        }
                        //Compare the old captures with the new one
                        if *old_captures != frame.consumes {
                            return branch_ret_mismatch();
                        }
                    },
                    _ => unreachable!() //To please compiler as it can not se that the if inner.is_none() guard covers None
                };
            }
            //Signal everything ok and it was not a throw
            true
        } else {
            //Signal everything ok and it was a throw
            false
        };

        Ok(is_res)
    }

    //Switch to the next Branch in the current Branching frame
    // It needs to know which elements the current branch results in (or if it even throws)
    pub fn next_branch(&mut self, br:&mut BranchInfo, rets: ExprResult) -> Result<()> {
        //get the current frame (pop it so we can modify and repush it)
        let frame = self.frames.pop().unwrap();
        //Undo Captures
        for c in &frame.consumes {
            self.stack[*c].status.consumed = false
        }
        //Do the return or the exit
        let frame_start = frame.start_index;
        //the weak drop (true param) automatically does undo captures while keeping single drop guarantees
        if self.ret_or_exit_branch(br, rets, frame)? {
            //Remove the results (they are already freed)
            while self.stack_depth() > frame_start {
                //always 0 as we pop right after it, forced true as these are results
                self.free_internal(ValueRef(0), true)?;
                //pop it
                self.stack.pop();
            }
        }

        //Push new empty frame for the next branch
        self.frames.push(Frame{
            start_index:frame_start,
            consumes:BTreeSet::new()
        });
        Ok(())
    }

    //Ends the current frame for good if it is a branching one this indicates that their is no more branch
    //  The rets parameter is the result from the last branch, where as the recovery is the the expected result in case the block/last branch threw an exception.
    pub fn end_branching(&mut self, mut br: BranchInfo, rets: ExprResult) -> Result<()>{
        //pop the frame
        let frame = self.frames.pop().unwrap();
        //this is done here instead after ret_or_exit_branch so we do not have to copy consumes
        if rets == ExprResult::Throw {
            //first undo the throwing frames captures (to get a clean state)
            for c in &frame.consumes {
                self.stack[*c].status.consumed = false
            }
        }

        //Clean up the branch and discover if it was a normal or exceptional return
        if !self.ret_or_exit_branch(&mut br, rets, frame)?{
            //it was exceptional meaning stack and consumes are not equal to the ones of the others and must be recovered
            match br.results {
                //Nothing to do all branches returned Error
                None => {},
                //Recover from previous results
                Some((old_res, old_captures)) => {
                    //Simulate the captures of a regular branch
                    for oc in old_captures {
                        //restore consume state
                        // Note: this is ok, as current frame is dropped and does not need consume entry any more and all other frames still have it
                        self.stack[oc].consume()?;
                    }
                    //Simulate the borrowing and result of a regular branch
                    for or in old_res {
                        //restore lock state (redo the borrowing)
                        for b in or.status.borrowing.slice() {
                            self.stack[*b].lock(1, false)?;
                        }
                        //Restore return Stack
                        self.stack.push(or);
                    }
                },
            }
        }
        Ok(())
    }

    //Ends the current Block
    pub fn end_block(&mut self, bi: BlockInfo, rets: ExprResult) -> Result<()>{
        //Clean up the branch
        self.ret_or_exit(bi.0, &rets)?;
        Ok(())
    }

    // Correlates the stack to the Function retrn signature, consuming the returns in the process
    pub fn check_function_return_signature<'b, T>(&mut self, returns:T) -> Result<()>
        where T:ExactSizeIterator<Item = &'b [ValueRef]> + DoubleEndedIterator<Item = &'b [ValueRef]> {
        //Check that thestack is big enough
        if self.stack_depth() < returns.len() {
            return num_return_mismatch();
        }

        //First check the returns, starting at the end
        for borrows in returns.rev() {
            //We pop and free them while we process them
            let mut elem = self.stack.pop().unwrap();
            // if it is consumed or locked it can not be return
            //  Note: as we free elems after processing returned locks are unlocked in time to be returned
            if elem.status.consumed {
                return fun_sig_mismatch();
            }

            let bor_set = elem.status.borrowing.slice().iter().cloned().collect::<BTreeSet<usize>>();

            //check len beforehand to ensure borrows has no duplicates (if it has the eq check afterward would fail=
            if bor_set.len() != borrows.len() {
                return fun_sig_mismatch();
            }

            //Turn the bernouli indexes in the return classifier to absolute ones
            // As the top of the stack is now (after drop) the bernouli index 0 we just have to apply absolute_index
            let borrows = borrows.iter().map(|val| Ok(self.absolute_index(*val)?)).collect::<Result<_>>()?;
            //check that the result is identical with what is specified
            if bor_set != borrows {
                return fun_sig_mismatch();
            }

            //Free the element and remove pending locks
            self.free_elem(&mut elem, true)?
        }

        //everything is fine
        Ok(())
    }

    // Correlates the stack to the Function signature, consuming the stack in the process and finishing the roundtrip
    pub fn check_function_param_signature<T>(&mut self, params:T, is_throw:bool) -> Result<()>
        where T:ExactSizeIterator<Item = bool> + DoubleEndedIterator<Item = bool> {
        //Chech that the number of elements on the stack matches the param of the Function in length
        if self.stack_depth() != params.len() {
            return num_param_mismatch();
        }

        //Second check the params, starting at the end
        for consumes in params.rev() {
            //We pop and free them while we process them
            let mut elem = self.stack.pop().unwrap();
            //The status of params must match the status specified and they are not allowed to be borrowed
            if (!is_throw && consumes != elem.status.consumed) | !elem.status.borrowing.is_empty() {
                return fun_sig_mismatch()
            }
            //Free the element
            self.free_elem(&mut elem, true)?;
        }

        //just a check that everiting is back to beginning
        assert!(self.stack.is_empty());
        assert_eq!(self.frames.len(), 1);
        assert_eq!(self.frames[0].start_index, 0);
        assert!(self.frames[0].consumes.is_empty());

        //everything is fine
        Ok(())
    }


    // Correlates the stack to the Function signature, consuming the stack in the process and finishing the roundtrip
    pub fn check_function_signature<'b, P,R>(&mut self, params:P, returns:R) -> Result<()>
        where R:ExactSizeIterator<Item = &'b [ValueRef]> + DoubleEndedIterator<Item = &'b [ValueRef]>,
              P:ExactSizeIterator<Item = bool> + DoubleEndedIterator<Item = bool>, {
        self.check_function_return_signature(returns)?;
        self.check_function_param_signature(params, false)?;
        //everything is fine
        Ok(())
    }

}