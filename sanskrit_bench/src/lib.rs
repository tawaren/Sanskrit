#![feature(test)]
#![feature(associated_type_defaults)]

pub mod bench;
pub mod externals;
extern crate test;

#[cfg(test)]
mod test_utils {
    use test::Bencher;
    use sanskrit_interpreter::model::{Exp, Entry};
    use sanskrit_interpreter::interpreter::{ExecutionContext, Frame};
    use sanskrit_common::arena::{Heap, VirtualHeapArena};
    use sanskrit_common::model::{SlicePtr, Ptr};
    use sanskrit_common::errors::*;
    use crate::externals::BenchExternals;

    pub trait TestCode {
        fn get_initials<'l,'h>(&self, alloc:&'l VirtualHeapArena<'h>) -> SlicePtr<'l, Entry<'l>>;
        fn get_code<'l,'h>(&self, alloc:&'l VirtualHeapArena<'h>) -> Ptr<'l, Exp<'l>>;
    }

    pub fn run_ops<T:TestCode>(t:T, b: &mut Bencher) -> Result<()> {
        let heap = Heap::new(512000000, 2.0);
        let alloc = heap.new_virtual_arena(128000000);
        let code = t.get_code(&alloc);
        let initials = t.get_initials(&alloc);
        let arena = heap.new_arena(64000000);
        let heap_alloc = heap.new_virtual_arena(64000000);
        let functions = vec![code];
        b.iter(||{
            let tmp_arena = arena.temp_arena();
            let local_heap = heap_alloc.temp_arena().unwrap();
            let mut frames = tmp_arena.alloc_stack::<Frame>(256);
            let mut stack = tmp_arena.alloc_stack::<Entry>(1024*64);
            let mut ret_stack = tmp_arena.alloc_stack::<Entry>(256);
            for v in initials.iter() { stack.push(*v).unwrap(); }
            ExecutionContext::interpret::<BenchExternals>(&functions, &mut stack, &mut frames, &mut ret_stack, &local_heap).unwrap();
        });
        Ok(())
    }
}
