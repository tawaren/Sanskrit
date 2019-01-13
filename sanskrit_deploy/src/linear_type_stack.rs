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
use alloc::prelude::*;


#[derive(Default)]
pub struct LinearTypeStack {
    stack:Vec<Elem<Crc<ResolvedType>, Vec<usize>>>,     //The actual Elems
    frames:Vec<Frame>,                                  //Frame borders, to record consumes in branches
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ExprResult<'a>{
    Return(&'a[ValueRef],&'a[ValueRef]),      //Defines the elements returned & dropedfrom a block (bernouli indexes)
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
pub struct BranchInfo{
    results:Option<(
        Vec<Elem<Crc<ResolvedType>,Vec<usize>>>,              // Vec<Elem> are the elems returned from a previous branch
        BTreeSet<usize>         // BTreeSet<usize> are the cross frame consumes from a previous branch (absolute indexes)
    )>
}

//migration structure
#[derive(Clone, Copy, Eq, PartialEq)]
enum Migrate {
    Free,
    Drop,
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
        self.stack[res].consume()?;
        //Return the original
        Ok(())
    }
}

//LinearTypeStack Specific stack
impl LinearTypeStack {
    //Generates an empty Stack
    pub fn new() -> Self {
        LinearTypeStack { stack: vec![], frames:vec![Frame{ start_index: 0, consumes: BTreeSet::new() }] }
    }

    //calcs the absolute index from a relative one (relative to the end) -- bernouli index to vec index
    fn absolute_index(&self, ValueRef(index):ValueRef) -> Result<usize> {
        //ensure that the resulting index will be valid
        if index as usize >= self.stack_depth() {return out_of_range_stack_addressing()}
        Ok(self.stack_depth() - (index as usize) -1)
    }

    //steals the borrows from an element recursively
    // This is only save if after this a return to free_border happens
    // This will never steal across free border
    // If unlock is true it will unlock the element at index if it is borrowed (after stealing)
    //   Must be calle bottom up to work as it expects
    fn steal_ret(&mut self, elem_pos:usize, free_border:usize, migrate:&[Migrate], index:usize, res:&mut BTreeSet<usize>, is_first:bool) -> Result<()>{
        if index < free_border {
            //as elem pos is > free_border we do not need stealing check
            if !res.contains(&index) {
                self.stack[index].lock(1, false)?;
                res.insert(index);
            }
        } else {
            let frame_pos = index - free_border;
            match migrate[frame_pos] {
                Migrate::Free | Migrate::Drop => {
                    if self.stack[index].status.borrowing.is_empty() {
                        return steal_violation();
                    } else {
                        for b in self.stack[index].status.borrowing.clone() {
                            //Prevents stealing from an element that was already processed with steal_ret
                            // and enforces borrowing order ( can not borrow from an element deeper on the stack )
                            // -- I'm not sure if this can ever happen: But better save than sorry --
                            if b >= index { return steal_violation(); }
                            self.steal_ret(elem_pos, free_border, migrate, b, res, false)?
                        }
                    }
                },
                Migrate::Move(pos) => {
                    //Prevents stealing from an element that will end up nearer to the top then the borrower
                    // and enforces borrowing order ( can not borrow from an element deeper on the stack )
                    // will be checked later again (but is here to prevent inconsistencies)
                    if pos >= elem_pos { return steal_violation(); }
                    if !res.contains(&pos) {
                        //Tell it that it has a lock more on it
                        self.stack[index].lock(1, false)?;
                        res.insert(pos);
                    }
                }
            }
        }
        //we have stolen the lock so unlock it (if this is the first steal)
        if is_first { self.stack[index].unlock()?; }
        Ok(())
    }

    //Cleans up a frame by freeing or returning each parameter
    // it does additionally adapt the borrow indexes of the returned elements to account for the dropped ones
    fn ret(&mut self, start_index:usize, rets:&[ValueRef], drops:&[ValueRef]) -> Result<()> {
        //Find out how many elements the frame has accumulated
        let frame_size = self.stack_depth()-start_index;
        //Make a vector that records from where to where returned elements are moved
        //  The index is from where and the entry to where, if a None is in their it is not returned/moved
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

        //initialize the migration vector with drops
        for val in drops {
            // find the absolute index of the droped element
            let index = self.absolute_index(*val)?;
            //an element can only be droped if it is contained in the current frame
            // Note: with an explicit Fetch or Copy this is still possible by moving it first into the frame
            if index < start_index {return elem_out_of_frame()}
            // find out to which index this correspond in the frame
            let frame_index = index-start_index;
            // check that the migration entry still free, if not this means the same element is returned twice
            if migrate[frame_index] != Migrate::Free {return consuming_moved_error()}
            // Make the migration entry
            migrate[frame_index] = Migrate::Drop
        }

        //check that the frame is not to big
        assert!(frame_size <= u16::max_value() as usize);

        // Free everything that will not be returned & return the rest
        //  It will steal_borrows if necessary and recalculate indexes
        for i in 0..frame_size {
            //start at the newest in the frame
            let index = self.absolute_index(ValueRef(i as u16))?;
            // find out to which index this correspond in the frame
            let frame_index = index-start_index;
            //check if returned or freed
            match migrate[frame_index] {
                Migrate::Free => self.free_internal(ValueRef(i as u16), false)?,
                Migrate::Drop => self.stack[index].consume()?,
                Migrate::Move(elem_pos) => {
                    //if the returned is borrowed, steal borrowes of non-returned
                    if !self.stack[index].status.borrowing.is_empty() {
                        let mut new_borrows = BTreeSet::new();
                        //steal from each borrowed
                        for b in self.stack[index].status.borrowing.clone() {
                            //unlocks it and then relocks the new borrow (done by steal_ret)
                            self.steal_ret(elem_pos,start_index,&migrate,b,&mut new_borrows, true)?;
                        }
                        //Set the new borrows
                        self.stack[index].status.borrowing = new_borrows.into_iter().collect();
                    }
                    self.stack[index].ensure_return()?
                }
            }
        }

        let mut returns = Vec::with_capacity(rets.len());
        //Save  the returned items
        for val in rets {
            // find the absolute index of the returned element
            let index = self.absolute_index(*val)?;
            // save the element
            returns.push(self.stack[index].clone())
        }

        //Clean up the stack by removing all slots (
        for _ in 0..frame_size {
            self.stack.pop().unwrap();
        }

        //push the retrns back onto the stack
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
        for _ in 0..frame_size {
            self.stack.pop().unwrap();
        }

        Ok(())
    }

    //Decides based on the result of an expresion if a return or an exit has to happen and returns false if an exit happened
    fn ret_or_exit(&mut self, start_index:usize, ret:&ExprResult) -> Result<bool>{
        //find out if we have to throw or to return and indicate it over the returned bool
        match ret {
            //We can return
            ExprResult::Return(rets, drops) => {
                //do the return
                self.ret(start_index, rets, drops)?;
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
            if let ExprResult::Return(res,_) = rets {
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
                        for (old, new) in old_res.into_iter().zip((0..res.len()).rev()) {
                            //Note: The Vec<usize>s in the were sorted by steal ret  -- so this is ok
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

            let bor_set = elem.status.borrowing.slice().into_iter().map(|c|*c).collect::<BTreeSet<usize>>();

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
    pub fn check_function_param_signature<T>(&mut self, params:T, ) -> Result<()>
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
            if consumes != elem.status.consumed | !elem.status.borrowing.is_empty() {
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
        self.check_function_param_signature(params)?;
        //everything is fine
        Ok(())
    }

}