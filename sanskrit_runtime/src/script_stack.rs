
use sanskrit_common::errors::*;
use sanskrit_common::linear_stack::*;
use sanskrit_common::model::*;
use sanskrit_common::capabilities::CapSet;
use model::*;
use elem_store::ElemStore;
use sanskrit_common::store::Store;
use sanskrit_common::arena::*;
use script_interpreter::HashingDomain;


#[derive(Clone, Copy, Debug)]
pub struct StackEntry<'a> {
    pub store_borrow: bool,
    pub val: Ptr<'a, Object<'a>>,
    pub typ: Ptr<'a, RuntimeType<'a>>,
}

impl<'a> StackEntry<'a> {
    pub fn new(val:Ptr<'a,Object<'a>>, typ:Ptr<'a,RuntimeType<'a>>) -> Self {
        StackEntry{
            store_borrow: false,
            val,
            typ
        }
    }

    pub fn new_store_borrowed(val:Ptr<'a,Object<'a>>, typ:Ptr<'a,RuntimeType<'a>>) -> Self {
        StackEntry{
            store_borrow: true,
            val,
            typ
        }
    }
}

pub struct LinearScriptStack<'a,'h> {
    stack:HeapStack<'h, Elem<StackEntry<'a>,SlicePtr<'a, usize>>>,     //The actual Elems
    alloc:&'a VirtualHeapArena<'h>
}


//Impl of BasStack for TypeStack
impl<'a, 'h> LinearStack<StackEntry<'a>,SliceBuilder<'a, usize>> for LinearScriptStack<'a,'h> {
    fn generate_vec(&self, elems: usize) -> Result<SliceBuilder<'a, usize>>{
        self.alloc.slice_builder(elems)
    }
    
    //Returns the stack depth. Important to later be able to drop a frame
    fn stack_depth(&self) -> usize {
        self.stack.len()
    }

    //gets the element ignoring consume tests
    fn get_elem(&self, index:ValueRef) -> Result<&Elem<StackEntry<'a>,SlicePtr<'a, usize>>>{
        // Get the index (as we not consuming it it is safe to access non-locals they are not consumed)
        let res = self.absolute_index(index)?;
        // Get the element
        self.stack.get(res)
    }

    fn get_elem_absolute(&mut self, index: usize) -> Result<&mut Elem<StackEntry<'a>, SlicePtr<'a, usize>>> {
        if index >= self.stack.len() {return out_of_range_stack_addressing()}
        self.stack.get_mut(index)
    }

    fn push_elem(&mut self, elem: Elem<StackEntry<'a>, SlicePtr<'a, usize>>) -> Result<()> {
        self.stack.push(elem)
    }

    //gets an element and consumes it
    fn consume(&mut self, index:ValueRef) -> Result<()>{
        // Get the index (as we not consuming it it is safe to access non-locals they are not consumed)
        let res = self.absolute_index(index)?;
        //Consume the stored one
        self.get_elem_absolute(res)?.consume()
    }
}

impl<'a,'h> LinearScriptStack<'a,'h> {

    //todo: seperate methods per section
    //todo can we merge with executor gen
    pub fn new(alloc:&'a VirtualHeapArena<'h>, stack:HeapStack<'h,Elem<StackEntry<'a>,SlicePtr<'a, usize>>>) -> Result<Self> {
        let mut script_stack = LinearScriptStack {
            stack,
            alloc
        };

        //add the special store sentinel -- used to borrow from store
        script_stack.provide(StackEntry{
            store_borrow: false,
            val: alloc.alloc(Object::Data(SlicePtr::empty()))?, //is irrelevant -- so use something safe
            typ: alloc.alloc(RuntimeType::NativeType{
                caps: CapSet::empty(),                //better safe then sorry
                typ: NativeType::Data(0),
                applies: SlicePtr::empty(),
            })?
        })?;
        //lock it so it is not used by other stuff except for provide_borrowed
        script_stack.get_elem_absolute(0)?.lock(1,true)?;
        Ok(script_stack)
    }

    //allows to put elems on the stack that appear out of nowhere (literals, empties, parameters)
    pub fn setup_push(&mut self, typ: StackEntry<'a>) -> Result<()> {
        self.provide(typ)
    }

    pub fn store_borrow(&mut self, val:StackEntry<'a>) -> Result<()> {
        assert!(val.store_borrow);


        let elem = Elem::borrowed(val, self.alloc.copy_alloc_slice(&[0usize])?);
        //Make sure it is borrowed from sentinel
        self.get_elem_absolute(0)?.lock(1,false)?;
        // put it onto the stack
        self.push_elem(elem)?;
        Ok(())
    }

    pub fn checked_clean_up<S:Store>(&mut self, store:&mut ElemStore<S>) -> Result<()> {
        //ensure we did not go over the limit
        if self.stack.len() > u16::max_value() as usize {
            return size_limit_exceeded_error()
        }
        //free all except store
        for i in 0..(self.stack.len() - 1) {
            let v_ref = ValueRef(i as u16);
            let elem = self.get_elem(v_ref)?;
            if !elem.status.borrowing.is_empty() {
                //if it is borrowed from the store we have to release it before we free it
                if elem.value.store_borrow {
                    let key = elem.value.val.extract_key();
                    store.free(key)
                }
            } else {
                //check if we have to / can drop (we should drop if it has the drop cap and can not be handled by free
                if !elem.status.consumed && elem.value.typ.get_caps().contains(NativeCap::Drop) {
                    self.drop(v_ref)?;
                }
            }
            self.free(v_ref)?;  //Calls ensure_freed
        }
        //free the store
        let store_rep = self.get_elem_absolute(0)?;
        //remove the lock we artificially put onto it to prevent interaction
        store_rep.unlock()?;
        //consume the no longer needed store
        store_rep.consume()?;
        //check that it is free (means their is no longer anything borrowed from the store)
        store_rep.ensure_freed(false)?;
        Ok(())
    }
}