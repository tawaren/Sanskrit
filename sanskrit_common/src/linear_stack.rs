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
use errors::*;
use model::ValueRef;
use core::mem;


//An elem has a type and a status (type is used by type checker)
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Elem<T:Clone, V:MicroVec<usize>> {pub status:Status<V>, pub value:T}

#[derive(Clone, Copy, Eq, PartialEq,Debug)]
pub struct Status<V:MicroVec<usize>> {
    pub consumed:bool,               //flag if the element is consumed or avaiable (also used to make it unavaiable while borrowed)
    pub locked:u8,                   //number of borrowers (can be more then one durin an inspect)
    pub borrowing:V                  //from whom is this borrowing (absolute indexes) (can be more than one whe constructed from borrows)
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum FetchMode {
    Consume,
    Copy,
    Borrow
}

pub trait MicroVecBuilder<T> {
    type MicroVec:MicroVec<T>;
    fn push(&mut self, val:T) -> Result<()>;
    fn finish(self) -> Self::MicroVec;
}

pub trait MicroVec<T> : Clone + Sized{
    fn zero() -> Self;
    fn is_empty(&self) -> bool;
    fn slice(&self) -> &[T];
}

impl<T:Clone,V:MicroVec<usize>> Elem<T,V> {
    //Generate a new fresh element with the specified type
    pub fn owned(typ:T) -> Self {
        Elem{
            status: Status{
                consumed: false,
                locked: 0,
                borrowing: V::zero(),
            },
            value: typ,
        }
    }

    pub fn borrowed(typ:T, borrowing:V) -> Self {
        Elem{
            status: Status{
                consumed: false,
                locked: 0,
                borrowing,
            },
            value: typ,
        }
    }

    //Consumes the Elem (which then is no longer usable)
    pub fn consume(&mut self) -> Result<()>{
        // Nothing can be consumed twice (linear types must be consumed exactly once)
        if self.status.consumed {return consuming_moved_error()}
        // Borrows can not be consumed -- but they can be locked see lock()
        if !self.status.borrowing.is_empty() {return consuming_moved_error()}
        //Mark it consumed
        self.status.consumed = true;
        Ok(())
    }

    //Consumes the Elem temporally (can later be restored over unlock)
    //Caller is responsible to ensure that unlock is called eventually (is done by recoding it in the element)
    pub fn lock(&mut self, amount:u8, initial:bool) -> Result<()> {
        //lock needs at least 1 burrower
        assert_ne!(amount, 0);
        //Check if it is already locked or not
        if self.status.locked == 0 || initial {
            // Nothing can be consumed twice (linear types must be consumed exactly once)
            if self.status.consumed {return locking_moved_error()}
            //Mark it consumed
            self.status.consumed = true;
        }
        //increase the lock count (used to kow when to unlock)
        self.status.locked += amount;
        Ok(())
    }

    //Restores a locked Elem (the caller is responsible to ensure that it was locked before and not already freed)
    pub fn unlock(&mut self) -> Result<()> {
        //Checks that it was locked (or consumed but we can not prevent this here)
        //The caller should ensure this so it is a precondition
        assert!(self.status.consumed);
        //we can not unlock twice
        assert_ne!(self.status.locked, 0);
        //decrease the lock counter
        self.status.locked -= 1;
        //if we removed the last borrower we can unconsume
        if self.status.locked == 0 {
            //Mark it as no longer consumed
            self.status.consumed = false;
        }
        Ok(())
    }

    //Checks that an element can savely be discarded
    // a free is forced if it is discarded through a throw or a branch reset
    //   a forced free is allowed to be unconsumed
    pub fn ensure_freed(&self, is_forced:bool) -> Result<()> {
        //Locked elements can never be freed
        if self.status.locked != 0 {
            return free_error()
        }

        //Borrowed elements can not be freed without firs unlocking the target
        assert!(self.status.borrowing.is_empty());
        //unconsumed elements can not be freed except if they are returned or rollbacked (indicated by is forced)
        if !self.status.consumed && !is_forced{
            return free_error()
        }
        Ok(())
    }

    //Checks if an element can savely be discarded
    pub fn can_be_freed(&self) -> bool{
        //Locked elements can never be freed
        self.status.locked == 0 && self.status.consumed
    }


    //Checks that an element can safely be returned
    pub fn ensure_return(&self) -> Result<()> {
        //Requires that the element is not consumed unless locked
        // Locked requires that the borrower is returned as well
        //  But This can not be checked here
        if self.status.consumed && self.status.locked == 0 {
            return consumed_cannot_be_returned_error()
        }
        Ok(())
    }

}

//Can be shared Between TypeStack and ScriptStack
pub trait LinearStack<T:Clone, V:MicroVecBuilder<usize>>  {

    //generate a micro vec
    fn generate_vec(&self, elems:usize) -> Result<V>;

    //gets the depth of the stack
    fn stack_depth(&self) -> usize;

    //gets the element ignoring consume tests
    fn get_elem(&self, index:ValueRef) -> Result<&Elem<T,V::MicroVec>>;

    //gets the element mutable by index ignoring consume tests
    fn get_elem_absolute(&mut self, index:usize) -> Result<&mut Elem<T,V::MicroVec>>;

    //pushes an element
    fn push_elem(&mut self,  elem:Elem<T,V::MicroVec>) -> Result<()>;

    //gets an element and consumes it
    //tracks cross block consumption if necessary
    fn consume(&mut self, index:ValueRef) -> Result<()>;

    //calcs the absolute index from a relative one (relative to the end) -- bernouli index to vec index
    fn absolute_index(&self, vref:ValueRef) -> Result<usize> {
        //extract index
        let index = vref.0;
        //ensure that the resulting index will be valid
        if index as usize >= self.stack_depth() {return out_of_range_stack_addressing()}
        Ok(self.stack_depth() - (index as usize) -1)
    }


    //gets an element without consuming it (returns it immutable)
    fn non_consuming_fetch(&self, index:ValueRef) -> Result<&Elem<T,V::MicroVec>>{
        // Get the element
        let elem = &self.get_elem(index)?;
        // Check that it is still alive and available (neither locked nor consumed)
        if elem.status.consumed {return cannot_access_consumed()}
        //Return it
        Ok(elem)
    }

    //hides an element (by consuming it), the caller is responsible for calling show
    // a hidden element can not be consumed, locked, or hidden again
    fn hide(&mut self, index:ValueRef) -> Result<()>{
        // Get the index (as we not consuming it it is safe to access non-locals they are not consumed)
        let res = self.absolute_index(index)?;
        // Get the element
        let elem = self.get_elem_absolute(res)?;
        // Check that it is still alive and available (neither locked nor consumed)
        if elem.status.consumed {return cannot_access_consumed()}
        //Mark it consumed
        elem.status.consumed = true;
        Ok(())
    }

    //hides an element (by consuming it), the caller is responsible for calling unhide
    // a hidden element can not be consumed, locked, or hidden again
    fn show(&mut self, index:ValueRef) -> Result<()>{
        // Get the index (as we not consuming it it is safe to access non-locals they are not consumed)
        let res = self.absolute_index(index)?;
        // Get the element
        let elem =  self.get_elem_absolute(res)?;
        // Ensure that it is still alive and available (neither locked nor consumed)
        assert!(elem.status.consumed);
        //Mark it consumed
        elem.status.consumed = false;
        Ok(())
    }

    //borrows an element and locks it while borrowing a copy of it
    fn borrow(&mut self, index:ValueRef, amount:u8) -> Result<Elem<T,V::MicroVec>> {
        // Get the index (non-locals are safe even as they are modified because they are restored before the frame ends or returned from the frame)
        let res = self.absolute_index(index)?;
        // Get the element
        let elem = self.get_elem_absolute(res)?;
        //Copy the original
        let mut orig =  elem.clone();
        //Lock the existing one
        elem.lock(amount, true)?;
        //overwrite the locking with a single one
        let mut mv_builder = self.generate_vec(1)?;
        mv_builder.push(res)?;
        orig.status.borrowing = mv_builder.finish();
        //Return the original
        Ok(orig)
    }

    //borrows multiple targets into one
    fn pack_borrow(&mut self, indexes:&[ValueRef], new_typ:T) -> Result<Elem<T,V::MicroVec>> {
        //create the new elem to return
        let mut borrowing = self.generate_vec(indexes.len())?;
        //Lock each borrowed element and enter it into the new one
        for index in indexes {
            let res = self.absolute_index(*index)?;
            // Get the element and lock it count
            self.get_elem_absolute(res)?.lock(1, true)?;
            //add to borrowing the locked target
            borrowing.push(res)?;
        }
        //Return the new element
        Ok(Elem::borrowed(new_typ, borrowing.finish()))
    }

    //frees an element
    fn free_elem(&mut self, elem:&mut Elem<T,V::MicroVec>, forced:bool) -> Result<()>{
        //if it is borrowed we need to unlock the targets
        if !elem.status.borrowing.is_empty() {
            //Release the borrows (unlock the target and consume the borrow)
            for unlock in elem.status.borrowing.slice() {
                self.get_elem_absolute(*unlock)?.unlock()?;
            }
            //it does no longer have borrow
            elem.status.borrowing = V::MicroVec::zero();
            //reconsume it to ensure it can be dropped and prevent it from being returned
            elem.status.consumed = true;

        }

        //check that it is free to go
        elem.ensure_freed(forced)?;

        Ok(())
    }

    //prepares an element to be released/droped from the stack
    fn free_internal(&mut self, index:ValueRef, forced:bool) -> Result<()> {
        // Get the index (as we consume it is not safe to access non-locals as they are modified)
        let res = self.absolute_index(index)?;

        //swap out borrows with empty ones to process full ones
        let borrows = mem::replace(
            &mut self.get_elem_absolute(res)?.status.borrowing,
            V::MicroVec::zero()
        );

        //Free the element
        if !borrows.is_empty() {
            //Release the borrows (unlock the target and consume the borrow)
            for unlock in borrows.slice() {
                self.get_elem_absolute(*unlock)?.unlock()?;
            }

            //re consume it to ensure it can be dropped and prevent it from being returned or used
            self.get_elem_absolute(res)?.status.consumed = true;
        }

        //check that it is free to go
        self.get_elem_absolute(res)?.ensure_freed(forced)
    }

    //returns the type of a stack elem without modifying anything
    fn value_of(&self, index:ValueRef) -> Result<T> {
        let elem = self.get_elem(index)?;
        Ok(elem.value.clone())
    }

    //returns the status of a stack elem without modifying anything
    fn is_borrowed(&self, index:ValueRef) -> Result<bool> {
        let elem = self.get_elem(index)?;
        Ok(!elem.status.borrowing.is_empty())
    }

    //allows to put elems on the stack that appear out of nowhere (literals, empties, parameters)
    fn provide(&mut self, typ:T) -> Result<()> {
        self.push_elem(Elem::owned(typ))
    }

    //frees an element, releases all borrows. Has no effect on a consumed
    // can not be used on locked
    fn free(&mut self, index:ValueRef) -> Result<()>{
        self.free_internal(index, false)
    }

    //drops a not needed element (this simply consumes it, it then can be freed on frame drop)
    fn drop(&mut self, index:ValueRef) -> Result<()>{
        self.consume(index)?;
        Ok(())
    }

    //lifts an element to the top and changes the type of the new one
    //  It does automatically re borrow if necessary
    fn transform(&mut self, index:ValueRef, typ:T, mode:FetchMode)-> Result<()> {
        //if the element is borrowed we only lock it so we can still return it after the transformed is freed
        let status = match mode {
            FetchMode::Consume => {
                //Fetch elem as consume in contrast to borrow not returns the element
                self.consume(index)?;
                Status{ consumed: false, locked: 0, borrowing: V::MicroVec::zero() }
            },
            FetchMode::Copy => {
                //Fetch elem as copy in contrast to borrow not returns the element
                self.non_consuming_fetch(index)?; //checks that it is unconsumed
                Status{ consumed: false, locked: 0, borrowing: V::MicroVec::zero() }
            },
            FetchMode::Borrow => self.borrow(index, 1)?.status,
        };

        // push the elem with same status but new typ
        // new status does already contain borrowed if needed
        self.push_elem(Elem{ status, value: typ, })
    }

    //Like a transform but keeps the same type
    fn fetch(&mut self, index:ValueRef, mode:FetchMode)-> Result<()> {
        //get the type
        let typ = self.value_of(index)?;
        //do the transform
        self.transform(index,typ.clone(),mode)
    }

    //Takes some values on the stack and uses it to construct another value from it (Adt creation)
    fn pack(&mut self, vals: &[ValueRef], res:T, mode:FetchMode) -> Result<()> {

        //When none are burrowed we can create a normal new adt (normal has prio in case of zero params)
        let elem = match mode {
            FetchMode::Consume => {
                //Consume pack fields
                for index in vals { self.consume(*index)?; }
                //build a new plain elem
                Elem::owned(res)
            },
            FetchMode::Copy => {
                //Copy pack fields
                for index in vals { self.non_consuming_fetch(*index)?; }
                //build a new plain elem
                Elem::owned(res)
            },
            //use the special pack burrow
            FetchMode::Borrow => self.pack_borrow(vals,res)?,
        };
        self.push_elem(elem)
    }

    //Takes a value and returns its content
    // it consumes the old value unless it is burrowed in that case the results are burrowing fro, it
    fn unpack(&mut self, index:ValueRef, results: &[T], is_borrow:bool) -> Result<()> {
        //check if the input is burrowed and create the status usable for the returns
        let new_status = if is_borrow {
            //if borrowed we only lock (so it is unlocked when the borrowed contained values leave scope)
            assert!(results.len() <= u8::max_value() as usize);
            //if their are no fields, the value is basically immediately released
            // This implies that tag is not linear in case of switch
            if !results.is_empty() {
                self.borrow(index, results.len() as u8)?.status
            } else {
                //Any return status goes as it is not used
                Status { consumed: false, locked: 0, borrowing: V::MicroVec::zero()}
            }
        } else {
            //if not borrowed it is consumed
            //Fetch elem as consume in contrast to borrow not returns the element
            let status = self.get_elem(index)?.status.clone();
            self.consume(index)?;
            status
        };
        // the unpacked values but preserve the status of the original
        for typ in results {
            //put each unpacked value on the stack
            self.push_elem(Elem{ status:new_status.clone(), value:typ.clone()})?;
        }
        Ok(())
    }

    fn field(&mut self, index:ValueRef, result: T, mode:FetchMode) -> Result<()>{
        self.transform(index,result,mode)
    }

    //this applies a Func transformation to the stack
    fn apply<'a>(&mut self, inputs: &[(ValueRef,bool)], results:&[(T,&'a [ValueRef])]) -> Result<()> {
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
            // will trigger insertion into captures (thats why just keeping it hidden would not work)
            if *consume  {
               self.consume(*index)?;
            }
        }

        //Process the Outputs
        for (index,(ref typ, borrows)) in results.iter().enumerate() {
            //if the Output is not borrowed, just push a fresh plain one
            if borrows.is_empty() {
                self.push_elem(Elem::owned(typ.clone()))?
            } else {
                //The output os burrowed, so we need to lock the targeted elems and change the refs to absolute ones
                let mut absolute = self.generate_vec(borrows.len())?;
                //Borrow the regular borrows
                for borrow in borrows.iter() {
                    //when the burrow is bigger then its position in the result it burrows from the inputs
                    let pos = if borrow.0 as usize >= index {
                        let num_vals = inputs.len();
                        //ensure the target exists
                        assert!((borrow.0 as usize - index) < num_vals);
                        //calculate the position of the referenced input
                        let pos_in_vals = num_vals-((borrow.0 as usize)-index)-1;
                        //resolve the absolute position (the index is added to counteract the push in the loop)
                        self.absolute_index(ValueRef((inputs[pos_in_vals].0).0+index as u16))?
                    //When it is smaller then its from the output
                    } else {
                        //the value can be used directly as it aligns with stack (0 is the top elem and this elem comes next)
                        self.absolute_index(*borrow)?
                    };
                    //lock the target
                    self.get_elem_absolute(pos)?.lock(1, false)?;
                    //record the borrow dependency
                    absolute.push(pos)?;
                }

                // Generate a new borrowed element on the stack
                self.push_elem(Elem::borrowed(typ.clone(), absolute.finish()))?
            }
        }
        Ok(())
    }
}