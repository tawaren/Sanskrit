//! A single assign stack managing values and enforcing linearity and borrow constraints
//!
//! The stacks is responsible for tracking the types of the elements as well as ensure that every element is used exactly once.
//!  Exception to the used exactly once rule (for example for values with the Copy or Drop capability) are triggered over explicit function calls to `copy` and `drop`.
//!  The stack further supports borrowing which allow to transfer a value from one stack slot to another (or even multiple) and back as soon as the new stack slots all leave the scope.
//!   While the element is in its temporary slot the previous slot is inaccessible (as if the value was used)
//!   The new element must not be an exact copy but instead can be a new element containing the borrowed or it could be a sub part of the borrowed element (a field of it)
//!
//! Every Element on the Stack represents a single value where the stack tracks the following properties:
//!  1. The Value itself (queryable over `value_of`)
//!  2. If it does borrow (queryable over `is_borrowing`)
//!
//! All elements on the stack are addressed by their offset from the topmost element
//!
//! The stack provides function's to do stack transformations including but not-limited to:
//!   1. apply -- which simulates the application of a stack transformation that was already compiled in an earlier pass
//!   2. pack -- which simulates the construction of an adt by consuming parameters and producing a new value on top of the stack
//!   3. unpack -- which simulates the consummation of an adt by consuming the element and producing the contained values on top of the stack
//!
use sanskrit_common::errors::*;
use sanskrit_common::model::ValueRef;
use alloc::vec::Vec;
use alloc::rc::Rc;
use core::cell::Cell;
use sanskrit_core::accounting::Accounting;


#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Status {
    Owned,
    Borrowed,
    Consumed
}

//An elem has a type and a status (type is used by type checker)
#[derive(Eq, PartialEq, Debug)]
pub struct Elem<T:Clone> {status:Rc<Cell<Status>>, locked:bool, mark:bool, value:T}

pub struct LinearStack<'a, T:Clone> {
    stack:Vec<Elem<T>>,           //The actual Elems
    frames:Vec<Frame>,            //Frame borders, to record consumes in branches
    status_owned:Rc<Cell<Status>>,
    status_borrowed:Rc<Cell<Status>>,
    accounting:&'a Accounting,
}

struct Frame{
    start_index:usize,                  //Where does the frame start
    consume_cell:Rc<Cell<Status>>,      //active consume cell
    new_marks:usize,
    remarked:usize,
}

//Information allowing type checker to manage frames and blocks
//it is a route through
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct BlockInfo(usize);                                    // usize is Block Start Index

pub struct BranchInfo<T:Clone>{
    locks:Option<usize>,
    results:Option<(Vec<T>,  usize)>,           // usize are the expected number of marked consumes expected by the branches
    remaining_branches: usize
}


#[derive(Copy, Clone, Eq, PartialEq)]
pub enum FetchMode {
    Consume,
    Copy
}

impl<T:Clone> Elem<T> {

    pub fn new(value:T, status: Rc<Cell<Status>>) -> Self {
        Elem{
            status,
            value,
            locked: false,
            mark: false
        }
    }

    //Consumes the Elem (which then is no longer usable)
    fn consume(&mut self, active_consume_cell:Rc<Cell<Status>>) -> Result<()>{
        // Nothing can be consumed twice (linear types must be consumed exactly once)
        if !self.is_owned() {
            return error(||"Consumed, borrowed, or locked element can not be consumed")
        }
        //Mark it consumed
        assert_eq!(active_consume_cell.get(), Status::Consumed);
        self.status = active_consume_cell;
        Ok(())
    }

    //Checks if an element can be discarded
    pub fn can_be_freed(&self) -> bool{
        //Locked elements can never be freed
        (self.status.get() != Status::Owned) && !self.locked
    }

    //Checks if an element can be discarded
    pub fn is_owned(&self) -> bool{
        //Locked elements can never be freed
        (self.status.get() == Status::Owned) && !self.locked
    }

    //Checks if an element can be discarded
    pub fn is_active(&self) -> bool{
        //Locked elements can never be freed
        (self.status.get() != Status::Consumed) && !self.locked
    }

    pub fn lock(&mut self) -> Result<()>{
        if !self.is_active() {
            return error(||"Consumed or locked element can not be locked")
        }
        self.locked = true;
        Ok(())
    }

    pub fn unlock(&mut self) {
        assert_ne!(self.status.get(), Status::Consumed);
        assert!(self.locked);
        self.locked = false;
    }
}

impl<'acc, T:Clone+Eq> LinearStack<'acc, T>  {
    //Generates an empty Stack
    pub fn new(accounting:&'acc Accounting) -> Self {
        LinearStack {
            stack: vec![],
            frames:vec![Frame{
                start_index: 0,
                consume_cell:Rc::new(Cell::new(Status::Consumed)),
                new_marks: 0,
                remarked: 0
            }],
            status_owned: Rc::new(Cell::new(Status::Owned)),
            status_borrowed: Rc::new(Cell::new(Status::Borrowed)),
            accounting
        }
    }


    //Returns the stack depth. Important to later be able to drop a frame
    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }

    //gets the element ignoring consume tests
    pub fn get_elem(&self, index:ValueRef) -> Result<&Elem<T>>{
        // Get the index (as we not consuming it it is safe to access non-locals they are not consumed)
        let res = self.absolute_index(index)?;
        // Get the element
        let elem = &self.stack[res];
        //Return it
        Ok(elem)
    }

    fn get_elem_absolute(&mut self, index: usize) -> Result<&mut Elem<T>> {
        if index >= self.stack.len() {
            return error(||"Accesed value lies outside of the stack");
        }
        Ok(&mut self.stack[index])
    }

    fn push_elem(&mut self, elem: Elem<T>) -> Result<()> {
        self.accounting.stack_elems(1)?;
        self.stack.push(elem);
        Ok(())
    }

    fn push_elems(&mut self, elem: Vec<Elem<T>>) -> Result<()> {
        self.accounting.stack_elems(elem.len())?;
        self.stack.extend(elem);
        Ok(())
    }

    fn get_consume_cell(&self) -> Rc<Cell<Status>> {
        self.frames.last().unwrap().consume_cell.clone()
    }

    //gets an element and consumes it
    pub fn consume(&mut self, index:ValueRef) -> Result<()>{
        // Get the index (as we consume it is not safe to access non-locals as they are modified)
        //get the absolute index
        let res = self.absolute_index(index)?;
        //as we consume the elem we have to record the acccess on each frame border
        //Start with first Frame
        let frame_index = self.frames.len() -1;
        let frame = &mut self.frames[frame_index];

        //get the elem
        let elem = &mut self.stack[res];

        //do the marking and counting needed to ensure that all branches consume the same elems
        if elem.mark {
            frame.remarked +=1;
        } else {
            frame.new_marks +=1;
            elem.mark = true;
        }

        //get the consume cell
        let consume_status = self.get_consume_cell();

        //consume it
        self.stack[res].consume(consume_status)
    }

    //calcs the absolute index from a relative one (relative to the end) -- bernouli index to vec index
    fn absolute_index(&self, vref:ValueRef) -> Result<usize> {
        //extract index
        let index = vref.0;
        //ensure that the resulting index will be valid
        if index as usize >= self.stack_depth() {
            return error(||"Stack access out of bounds")
        }
        Ok(self.stack_depth() - (index as usize) -1)
    }


    //gets an element without consuming it (returns it immutable)
    fn non_consuming_fetch(&self, index:ValueRef) -> Result<&Elem<T>>{
        // Get the element
        let elem = &self.get_elem(index)?;
        // Check that it is still alive and available (neither locked nor consumed)
        if !elem.is_active() {
            return error(||"A consumed or locked element can not be fetched")
        }
        //Return it
        Ok(elem)
    }

    //hides an element (by marking consuming it), the caller is responsible for calling show
    // a hidden element can not be consumed, locked, or hidden again
    fn hide(&mut self, index:ValueRef) -> Result<()>{
        // Get the index (as we not consuming it it is safe to access non-locals they are not consumed)
        let res = self.absolute_index(index)?;
        // Get the element
        let elem = self.get_elem_absolute(res)?;
        // Check that it is still alive and available (neither locked nor consumed)
        if !elem.is_active() {
            return error(||"A consumed, locked or hidden element can not be hidden")
        }
        //Mark it consumed
        elem.locked = true;
        Ok(())
    }

    //shows an element (by ubconsuming it)
    fn show(&mut self, index:ValueRef) -> Result<()>{
        // Get the index (as we not consuming it it is safe to access non-locals they are not consumed)
        let res = self.absolute_index(index)?;
        // Get the element
        let elem =  self.get_elem_absolute(res)?;
        // Ensure that it is still alive and available (neither locked nor consumed)
        assert_ne!(elem.status.get(), Status::Consumed);
        assert!(elem.locked);

        //Mark it consumed
        elem.locked = false;
        Ok(())
    }

    //returns the type of a stack elem without modifying anything
    pub fn value_of(&self, index:ValueRef) -> Result<T> {
        let elem = self.get_elem(index)?;
        Ok(elem.value.clone())
    }

    //allows to put elems on the stack that appear out of nowhere (literals, empties, parameters, unpacks)
    pub fn provide(&mut self, typ:T) -> Result<()> {
        self.push_elem(Elem::new(typ,self.status_owned.clone()))
    }

    //allows to put a read only param on the stack that appear out of nowhere (parameters, switch unpacks)
    pub fn borrow(&mut self, value:T) -> Result<()> {
        self.push_elem(Elem::new(value, self.status_borrowed.clone()))
    }


    //drops a not needed element (this simply consumes it, it then can be freed on frame drop)
    pub fn drop(&mut self, index:ValueRef) -> Result<()>{
        self.consume(index)?;
        Ok(())
    }

    //lifts an element to the top and changes the type of the new one
    //  It does automatically re borrow if necessary
    pub fn transform(&mut self, index:ValueRef, value:T, mode:FetchMode)-> Result<()> {
        //if the element is borrowed we only lock it so we can still return it after the transformed is freed
        match mode {
            FetchMode::Consume => self.consume(index)?,
            FetchMode::Copy => {self.non_consuming_fetch(index)?;}
        };

        // push the elem with same status but new typ
        // new status does already contain borrowed if needed
        self.push_elem( Elem::new(value, self.status_owned.clone()))
    }

    //Like a transform but keeps the same type
    pub fn fetch(&mut self, index:ValueRef, mode:FetchMode)-> Result<()> {
        //get the type
        let typ = self.value_of(index)?;
        //do the transform
        self.transform(index,typ.clone(),mode)
    }

    //Takes some values on the stack and uses it to construct another value from it (Adt creation)
    pub fn pack(&mut self, vals: &[ValueRef], res:T, mode:FetchMode) -> Result<()> {
        match mode {
            //Consume pack fields
            FetchMode::Consume => for index in vals {
                self.consume(*index)?;
            },
            //Copy pack fields
            FetchMode::Copy => for index in vals {
                self.non_consuming_fetch(*index)?;
            }
        };
        self.push_elem(Elem::new(res, self.status_owned.clone()))
    }

    //Takes a value and returns its content
    // it consumes the old value
    pub fn unpack(&mut self, index:ValueRef, results: &[T],  mode:FetchMode) -> Result<()> {
        assert!(results.len() <= u8::max_value() as usize);
        //fetches the input
        match mode {
            //it is consumed
            FetchMode::Consume => self.consume(index)?,
            //it is copied
            FetchMode::Copy => {self.non_consuming_fetch(index)?;},
        };

        // unpacked values
        for value in results {
            //put each unpacked value on the stack
            self.push_elem( Elem::new(value.clone(),self.status_owned.clone()))?;
        }
        Ok(())
    }

    //Takes a value and returns its content
    // it consumes the old value
    pub fn inspect(&mut self, index:ValueRef, results: &[T]) -> Result<()> {
        assert!(results.len() <= u8::max_value() as usize);
        //check that the input was locked
        if !self.get_elem(index)?.locked {
            return error(||"A stack elem must be locked by the enclosing frame in order to be inspected")
        };

        // unpacked values
        for value in results {
            //put each unpacked value on the stack
            self.push_elem(Elem::new(value.clone(),self.status_borrowed.clone()))?;
        }
        Ok(())
    }

    pub fn field(&mut self, index:ValueRef, result: T, mode:FetchMode) -> Result<()>{
        self.transform(index,result,mode)
    }


    //this applies a Func transformation to the stack
    pub fn consume_params<'a>(&mut self, inputs: &[(ValueRef,bool)]) -> Result<()> {
        //Hide all (ensuring that they are unconsumed and not used twice)
        for (index,_) in inputs {
            //Avoids consume cross frame checks
            //But still ensures that it is only used once as input
            self.hide(*index)?;
        }

        //Process the inputs
        for (index, consume) in inputs {
            //make the element visible again (it is sure that each is used only once)
            self.show(*index)?;
            //The Function consumes the Element
            if *consume  {
                self.consume(*index)?;
            }
        }
        Ok(())
    }

    //Cleans up a frame by freeing or returning each parameter
    fn ret(&mut self, start_index:usize, rets:u8) -> Result<usize> {
        //to capture the returns temporary in correct order
        //todo: we could get more efficient if we free first & than directly copy down and thus eliminate the extra return vector
        let mut returns = Vec::with_capacity(rets as usize);
        //check each return
        for val in (0..(rets as u16)).rev() {
            // find the absolute index of the returned element
            let index = self.absolute_index(ValueRef(val))?;
            //an element can only be returned if it is contained in the current frame (This should always be true in the new return model)
            assert!(index >= start_index);

            let elem = &self.stack[index];
            // save the element for later re pushing it
            returns.push(Elem::new(elem.value.clone(), self.status_owned.clone()));
            //ensure its owned
            if !self.stack[index].is_owned() {
                return error(||"Only owned values can be the result of an expression")
            }
        }

        //count marks
        let mut lost_marks = 0;
        // Free everything that is not returned
        for index in start_index..(self.stack_depth() - rets as usize) {
            //start at the newest in the frame
            //check it is freed
            if !self.stack[index].can_be_freed() {
                return error(||"Can only discard Consumed or Borrowed values on return");
            }
            //check if it is marked
            if self.stack[index].mark {
                lost_marks += 1;
            }
        }

        //Clean up the stack by removing all entries in the current frame
        self.stack.truncate(start_index);
        //push the returns back onto the stack
        self.push_elems(returns)?;

        assert!(self.stack.len() <= u16::max_value() as usize);

        Ok(lost_marks)
    }

    //open a new basic frame which does not branch and thus allowing it to consume outside elements
    pub fn start_block(&mut self) -> BlockInfo {
        //Remember the blocks start (needed for ret later)
        BlockInfo(self.stack_depth())
    }

    //open a new frame which does branch, it also starts the first branch immediately
    //the returned branch info then can be used to switch to the next branch or ending the branching
    fn start_branching_internal(&mut self, branches:usize, locks:Option<usize>) -> BranchInfo<T> {
        //Remember the frames start
        let start_index = self.stack_depth();

        //new consume cell
        let consume_cell = if branches == 1 {
            //The last branch is not rollbacked by this branching operator so we use the parents cell
            //delegating the unsetting to them
            self.frames.last().unwrap().consume_cell.clone()
        } else {
            Rc::new(Cell::new(Status::Consumed))
        };

        //Create the new frame and push it
        self.frames.push(Frame{
            start_index,
            consume_cell,
            new_marks: 0,
            remarked: 0
        });
        BranchInfo{ results: None, locks, remaining_branches: branches }
    }

    //open a new frame which does branch, it also starts the first branch immediately
    //the returned branch info then can be used to switch to the next branch or ending the branching
    pub fn start_branching(&mut self, branches:usize) -> BranchInfo<T> {
        self.start_branching_internal(branches, None)
    }

    //open a new frame which does branch, it also starts the first branch immediately
    //it temporarly makes an element unavaiable until the branching finishes
    //the returned branch info then can be used to switch to the next branch or ending the branching
    pub fn start_locked_branching(&mut self, branches:usize, locked:ValueRef) -> Result<BranchInfo<T>> {
        //get the elem to lock
        let index = self.absolute_index(locked)?;
        //lock the elem
        self.stack[index].lock()?;
        //star the branch
        Ok(self.start_branching_internal(branches, Some(index)))
    }

    //Helper to exit/ret a branch and check the returns (must be the same)
    fn ret_branch(&mut self, br:&mut BranchInfo<T>, res: u8, frame:&mut Frame) -> Result<()> {
        //do the return
        let lost_marks = self.ret(frame.start_index, res)?;
        frame.new_marks -= lost_marks;
        //check that the return is in bound
        match br {
            // Capture the Information of this branch run for the remaining ones
            BranchInfo{results:ref mut inner,.. } if inner.is_none() => {
                let parent_frame = self.frames.last_mut().unwrap();
                parent_frame.new_marks += frame.new_marks;
                parent_frame.remarked += frame.remarked;

                *inner = Some((
                    //Calculate the returned elements
                    (0..res).rev().map(|new| Ok(self.get_elem(ValueRef(new as u16))?.value.clone())).collect::<Result<Vec<T>>>()?,
                    frame.new_marks + frame.remarked
                ))
            }
            BranchInfo{results:Some((old_res, expected_remarks)), ..} => {
                //Compare the old elems with the new one
                for (old, new) in old_res.iter().zip((0..res).rev()) {
                    if old != &self.get_elem(ValueRef(new as u16))?.value {
                        return error(||"Branches must produce same returns");
                    }
                }

                //Compare the old captures with the new one
                if frame.new_marks != 0 || frame.remarked != *expected_remarks {
                    return error(||"Branches must consume same stack slots");
                }
            },
            _ => unreachable!() //To please compiler as it can not se that the if inner.is_none() guard covers None
        };
        Ok(())
    }

    //Switch to the next Branch in the current Branching frame
    // It needs to know which elements the current branch results in (or if it even throws)
    pub fn next_branch(&mut self, br:&mut BranchInfo<T>, rets: u8) -> Result<()> {
        //get the current frame (pop it so we can modify and repush it)
        let mut frame = self.frames.pop().unwrap();
        //Do the return or the exit
        let frame_start = frame.start_index;
        //capture the returns into the br and clean the frame
        self.ret_branch(br, rets, &mut frame)?;
        //Remove the results (they are already freed)
        self.stack.truncate(frame_start);
        //Undo the Captures for next frame
        frame.consume_cell.set(Status::Owned);
        //remove finished branch
        br.remaining_branches -= 1;
        assert_ne!(br.remaining_branches, 0);
        //new consume cell
        let consume_cell = if br.remaining_branches == 1 {
            //The last branch is not rollbacked by this branching operator so we use the parents cell
            //delegating the unsetting to them
            self.frames.last().unwrap().consume_cell.clone()
        } else {
            Rc::new(Cell::new(Status::Consumed))
        };

        //Push new empty frame for the next branch
        self.frames.push(Frame{
            start_index:frame_start,
            consume_cell,
            new_marks: 0,
            remarked: 0
        });
        Ok(())
    }

    //Ends the current frame for good if it is a branching one this indicates that their is no more branch
    //  The rets parameter is the result from the last branch, where as the recovery is the the expected result in case the block/last branch threw an exception.
    pub fn end_branching(&mut self, mut br: BranchInfo<T>, rets: u8) -> Result<()>{
        assert_eq!(br.remaining_branches, 1);

        //pop the frame
        let mut frame = self.frames.pop().unwrap();

        //Release locks if any
        if let Some(idx) = br.locks {
            self.stack[idx].unlock();
        }

        //Clean up the branch
        self.ret_branch(&mut br, rets, &mut frame)
    }

    //Ends the current Block
    pub fn end_block(&mut self, bi: BlockInfo, rets: u8) -> Result<()>{
        //do the return
        let lost_marks = self.ret(bi.0, rets)?;
        self.frames.last_mut().unwrap().new_marks -= lost_marks;
        Ok(())
    }

    // Correlates the stack to the Function retrn signature, consuming the returns in the process
    pub fn check_function_return_signature(&mut self, returns:u8) -> Result<()>  {
        //Check that the stack is big enough
        if self.stack_depth() < returns as usize{
            return error(||"Not enough elements on the stack to cover returns")
        }

        //First check the returns, starting at the end
        for _ in 0..returns {
            //We pop and free them while we process them
            let elem = self.stack.pop().unwrap();
            // if it is not-owned or locked it can not be return
            if !elem.is_owned() {
                return error(||"Returns must be owned at the end of a function body")
            }
        }

        //everything is fine
        Ok(())
    }

    // Correlates the stack to the Function signature, consuming the stack in the process and finishing the roundtrip
    pub fn check_function_param_signature(&mut self, params:u16) -> Result<()>  {
        //Check that the stack is big enough
        if self.stack_depth() != params as usize {
            return error(||"Number of elements on stack must match number of parameters")
        }

        //Second check the params, starting at the end
        for _ in 0..params {
            //We pop and free them while we process them
            let elem = self.stack.pop().unwrap();
            //if a regular return check that signature hold
            if !elem.can_be_freed() {
                return error(||"Parameters must be borrowed or consumed at the end of a function body")
            }
        }

        //just a check that everiting is back to beginning
        assert_eq!(self.frames.len(), 1);
        assert_eq!(self.frames[0].start_index, 0);
        Ok(())
    }
}