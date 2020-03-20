

#[cfg(test)]
mod tests {
    use sanskrit_deploy::linear_stack::*;
    use sanskrit_common::model::*;
    use sanskrit_core::utils::*;
    use sanskrit_core::model::resolved::*;
    use std::ops::DerefMut;
    use std::ops::Deref;
    use sanskrit_core::model::bitsets::{CapSet, BitSet};
    use sanskrit_core::accounting::Accounting;
    use std::cell::Cell;

    fn max_accounting() -> Accounting {
        Accounting {
            load_byte_budget: Cell::new(usize::max_value()),
            store_byte_budget: Cell::new(usize::max_value()),
            process_byte_budget: Cell::new(usize::max_value()),
            stack_elem_budget: Cell::new(usize::max_value()),
            nesting_limit: 10,
            input_limit: 1000000
        }
    }
    
    struct Tester<'a> {
        dedup: CrcDeDup<ResolvedType>,
        inner: LinearStack<'a, Crc<ResolvedType>>,
        types: Vec<Crc<ResolvedType>>
    }

    impl<'a> Deref for Tester<'a>{
        type Target = LinearStack<'a, Crc<ResolvedType>>;
        fn deref(&self) -> &LinearStack<'a, Crc<ResolvedType>> {
            &self.inner
        }
    }

    impl<'a>  DerefMut for Tester<'a> {
        fn deref_mut(&mut self) -> &mut LinearStack<'a, Crc<ResolvedType>>{
            &mut self.inner
        }
    }

    impl<'a> Tester<'a> {
        pub fn new(accounting: &'a Accounting) -> Self {
            Tester{
                dedup: CrcDeDup::new(),
                inner: LinearStack::new(accounting),
                types: Vec::new(),
            }
        }

        fn new_type(&mut self, len:usize) -> Crc<ResolvedType> {
            assert!(len <= u8::max_value() as usize);
            //Reminder that this will fail
            self.dedup.dedup(ResolvedType::Generic {
                caps: CapSet::empty(),
                offset: len as u8,
                is_phantom: false
            })
        }

        pub fn provide_owned(&mut self) -> Crc<ResolvedType>{
            let elem = self.new_type(self.types.len());
            self.types.push(elem.clone());
            self.inner.provide(elem.clone()).unwrap();
            elem
        }

        pub fn provide_borrowed(&mut self) -> Crc<ResolvedType>{
            let elem = self.new_type(self.types.len());
            self.types.push(elem.clone());
            self.inner.borrow(elem.clone()).unwrap();
            elem
        }

        pub fn introduce_new(&mut self) -> Crc<ResolvedType>{
            let elem = self.new_type(self.types.len());
            self.types.push(elem.clone());
            elem
        }


        pub fn get_type(&self, pos:usize) -> Crc<ResolvedType> {
            let len = self.types.len();
            self.types[len-pos-1].clone()
        }

        pub fn owned(&self, pos:usize) -> Crc<ResolvedType>{
            self.get_type(pos)
        }


        pub fn param(&self, pos:usize) -> Crc<ResolvedType> {
            self.get_type(pos)
        }

    }

    fn test<'a, F:FnOnce(& mut Tester<'a>) -> (Vec<Crc<ResolvedType>>, Vec<Crc<ResolvedType>>)>(accounting:&'a Accounting, f:F){
        let mut tester = Tester::new(accounting);
        //Generate some garbage that should be untouched by test
        for _ in 0..20 {
            tester.provide_borrowed();
        }
        let (new_params, rets) = f(&mut tester);

        //Test types where we still allowed to
        let depth = new_params.len() + rets.len();
        for (i, typ) in new_params.iter().enumerate() {
            //consumed are no longer accessible
            assert_eq!(tester.value_of(ValueRef((depth-1-i) as u16)).unwrap(), *typ)
        }

        let depth = rets.len();
        for (i, typ) in rets.iter().enumerate() {
            //consumed are no longer accessible
            match tester.value_of(ValueRef((depth-1-i) as u16)) {
                Ok(t) => assert_eq!(t, *typ),
                _ => {}
            }
        }

        tester.inner.check_function_return_signature(rets.len() as u8).unwrap();
        tester.inner.check_function_param_signature(20 + new_params.len() as u16, false).unwrap();
    }


    //First some test that provoke a specific error

    //consume
    // "Consuming moved slot is forbidden"
    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn consume_twice_test() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            stack.drop(ValueRef(0)).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            (vec![stack.param(0)], vec![])
        });
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn consume_twice_test2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0), &vec![t], FetchMode::Consume).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            (vec![stack.param(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn consume_twice_test3() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.unpack(ValueRef(0), &vec![t], FetchMode::Consume).unwrap();
            (vec![stack.param(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn consume_twice_test4() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0)], t, FetchMode::Consume).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            (vec![stack.param(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn consume_twice_test5() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.pack(&[ValueRef(0)], t, FetchMode::Consume).unwrap();
            (vec![stack.param(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn consume_twice_test6() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0),ValueRef(0)], t, FetchMode::Consume).unwrap();
            (vec![stack.param(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn consume_twice_test7() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Consume).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            (vec![stack.param(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn consume_twice_test8() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.transform(ValueRef(0), t, FetchMode::Consume).unwrap();
            (vec![stack.param(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn consume_twice() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0),ValueRef(0)], t, FetchMode::Consume).unwrap();
            (vec![], vec![])
        });
    }

    #[test]
    #[should_panic(expected="Can only discard Consumed or Borrowed values on return")]
    fn free_unconsumed2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let block = stack.start_block();
            stack.provide_owned();
            stack.end_block(block, 0).unwrap();
            (vec![], vec![])
        });
    }

    //ensure_return (Can be achieved over sub ock)
    //"A consumed slots can not be returned"
    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn return_consumed() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let block = stack.start_block();
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Consume).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            stack.fetch(ValueRef(1), FetchMode::Consume).unwrap();
            stack.end_block(block, 1).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Only owned values can be the result of an expression")]
    fn return_consumed2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let block = stack.start_block();
            stack.provide_owned();
            stack.drop(ValueRef(0)).unwrap();
            stack.end_block(block, 1).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn return_consumed3() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let block = stack.start_block();
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0), &[t], FetchMode::Consume).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            stack.fetch(ValueRef(1), FetchMode::Consume).unwrap();
            stack.end_block(block, 1).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }



    //ensure_return (Can be achieved over sub ock)
    //"A consumed slots can not be returned"
    #[test]
    #[should_panic(expected="Only owned values can be the result of an expression")]
    fn return_borrowed() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let block = stack.start_block();
            stack.provide_borrowed();
            stack.end_block(block, 1).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn return_borrowed2() {
        let accounting = max_accounting();
                test(&accounting,|stack|{
            let block = stack.start_block();
            stack.provide_borrowed();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Copy).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            stack.fetch(ValueRef(1), FetchMode::Consume).unwrap();
            stack.end_block(block, 1).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }


    // absolute_index
    // "Targeted element lies outside of stack"

    #[test]
    #[should_panic(expected="Index out of bounds")]
    fn reach_outside() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.drop(ValueRef(20)).unwrap();
            (vec![], vec![])
        });
    }

    #[test]
    #[should_panic(expected="Index out of bounds")]
    fn reach_outside2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t = stack.introduce_new();
            stack.transform(ValueRef(20), t, FetchMode::Consume).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Index out of bounds")]
    fn reach_outside3() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t = stack.introduce_new();
            stack.unpack(ValueRef(20), &[t], FetchMode::Consume).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }


    #[test]
    #[should_panic(expected="Index out of bounds")]
    fn reach_outside4() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.fetch(ValueRef(20), FetchMode::Copy).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Index out of bounds")]
    fn reach_outside5() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(20)], t, FetchMode::Consume).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }


    //hide
    //"Can not access already moved slot"
    #[test]
    #[should_panic(expected="A consumed, locked or hidden element can not be hidden")]
    fn hide_test() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            stack.consume_params(&[(ValueRef(0), false), (ValueRef(0), false)]).unwrap();
            (vec![], vec![])
        });
    }
    // ret
    //"Consuming moved slot is forbidden"
    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn double_frame_return_test() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let block = stack.start_block();
            stack.provide_owned();
            stack.fetch(ValueRef(0), FetchMode::Consume).unwrap();
            stack.fetch(ValueRef(1), FetchMode::Consume).unwrap();
            stack.end_block(block, 2).unwrap();
            (vec![], vec![stack.owned(0),stack.owned(0)])
        });
    }

    //ret_or_exit_branch
    //"branches must induce the same post state"

    #[test]
    #[should_panic(expected="Branches must produce same returns")]
    fn mismatching_branch_return_test() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let mut branching = stack.start_branching(2);
                stack.provide_owned();
            stack.next_branch(&mut branching, 1).unwrap();
                stack.provide_owned();
            stack.end_branching(branching, 1).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Branches must produce same returns")]
    fn mismatching_branch_return_test2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            let mut branching = stack.start_branching(2);
                let t1 = stack.introduce_new();
                stack.transform(ValueRef(0), t1, FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching, 1).unwrap();
                let t2 = stack.introduce_new();
                stack.transform(ValueRef(0), t2, FetchMode::Consume).unwrap();
            stack.end_branching(branching, 1).unwrap();
            (vec![stack.param(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Branches must produce same returns")]
    fn mismatching_branch_return_test3() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching(2);
                stack.provide(t0.clone()).unwrap();
                stack.provide(t0.clone()).unwrap();
                stack.transform(ValueRef(1), t1.clone(), FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching, 2).unwrap();
                stack.provide(t0.clone()).unwrap();
                stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.end_branching(branching, 1).unwrap();
            (vec![], vec![stack.owned(1),stack.owned(1)])
        });
    }

    //"branches must induce the same post state"
    #[test]
    #[should_panic(expected="Branches must consume same stack slots")]
    fn mismatching_branch_capture_test() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t0 = stack.introduce_new();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t0.clone()).unwrap();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching(2);
                stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching, 1).unwrap();
                stack.transform(ValueRef(1), t1.clone(), FetchMode::Consume).unwrap();
            stack.end_branching(branching, 1).unwrap();
            (vec![stack.param(1), stack.param(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Branches must consume same stack slots")]
    fn mismatching_branch_capture_test2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t0 = stack.introduce_new();
            stack.provide(t0.clone()).unwrap();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching(2);
                stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching, 1).unwrap();
                stack.provide(t0.clone()).unwrap();
                stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.end_branching(branching, 1).unwrap();
            (vec![stack.param(1)], vec![stack.owned(0)])
        });
    }

    // check_function_signature
    // "Number of params is wrong"


    #[test]
    #[should_panic(expected="Number of elements on stack is must match number of parameters")]
    fn signature_size_missmatch() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            (vec![], vec![])
        });
    }


    #[test]
    #[should_panic(expected="Number of elements on stack is must match number of parameters")]
    fn signature_size_missmatch3() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            (vec![stack.param(0)], vec![])
        });
    }

    //"Function signature mismatch"
    #[test]
    #[should_panic(expected="Returns must be owned at the end of a function body")]
    fn signature_result_mode_missmatch() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            stack.drop(ValueRef(0)).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    //Old test need remoddeling int pure success tests
    // make some test to each exception that trigger the test but pass it
    // May be not for all in a usefull manner possible

    //consume
    // [not] "Consuming moved slot is forbidden"
    // [not] "Consuming moved slot is forbidden"

    #[test]
    fn consume_slot() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Consume).unwrap();
            (vec![stack.param(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    fn consume_slot2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0)], t, FetchMode::Consume).unwrap();
            (vec![stack.param(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    fn consume_slot3() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(1),ValueRef(0)], t, FetchMode::Consume).unwrap();
            (vec![stack.param(2), stack.param(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    fn consume_slot4() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0), &[t], FetchMode::Consume).unwrap();
            (vec![stack.param(1)], vec![stack.owned(0)])
        });
    }


    #[test]
    fn consume_slot5() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.unpack(ValueRef(0), &[t0, t1], FetchMode::Consume).unwrap();
            (vec![stack.param(2)], vec![stack.owned(1), stack.owned(0)])
        });
    }

    #[test]
    fn consume_slot6() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            stack.drop(ValueRef(0)).unwrap();
            (vec![stack.param(0)], vec![])
        });
    }
    //ensure_freed
    //[not] "Only consumed and not locked elem slots can be freed"
    //[not] "Only consumed and not locked elem slots can be freed"



    #[test]
    fn free_consumed5() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let block = stack.start_block();
            stack.provide_owned();
            stack.drop(ValueRef(0)).unwrap();
            stack.end_block(block, 0).unwrap();
            (vec![], vec![])
        });
    }

    #[test]
    fn free_consumed7() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let block = stack.start_block();
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Consume).unwrap();
            stack.end_block(block, 1).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    fn free_consumed8() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let block = stack.start_block();
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0)], t, FetchMode::Consume).unwrap();
            stack.end_block(block,1).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    fn free_consumed9() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let block = stack.start_block();
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0), &[t], FetchMode::Consume).unwrap();
            stack.end_block(block,1).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    fn free_consumed10() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let mut branching = stack.start_branching(2);
            stack.provide_owned();
                stack.drop(ValueRef(0)).unwrap();
            stack.next_branch(&mut branching, 0).unwrap();
                stack.provide_owned();
                stack.drop(ValueRef(0)).unwrap();
            stack.end_branching(branching, 0).unwrap();
            (vec![], vec![])
        });
    }

    #[test]
    fn free_consumed11() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let mut branching = stack.start_branching(2);
            stack.provide_owned();
                stack.drop(ValueRef(0)).unwrap();
            stack.next_branch(&mut branching, 0).unwrap();
                stack.provide_owned();
                stack.drop(ValueRef(0)).unwrap();
            stack.end_branching(branching, 0).unwrap();
            (vec![], vec![])
        });
    }

    #[test]
    fn free_consumed12() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching(2);
                stack.provide(t0.clone()).unwrap();
                stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching, 1).unwrap();
                stack.provide(t0).unwrap();
                stack.transform(ValueRef(0), t1, FetchMode::Consume).unwrap();
            stack.end_branching(branching, 1).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    fn free_consumed13() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching(2);
                stack.provide(t0.clone()).unwrap();
                stack.pack(&[ValueRef(0)], t1.clone(), FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching, 1).unwrap();
                stack.provide(t0).unwrap();
                stack.pack(&[ValueRef(0)], t1, FetchMode::Consume).unwrap();
            stack.end_branching(branching, 1).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    fn free_consumed14() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching(2);
                stack.provide(t0.clone()).unwrap();
                stack.unpack(ValueRef(0), &[t1.clone()], FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching, 1).unwrap();
                stack.provide(t0).unwrap();
                stack.unpack(ValueRef(0), &[t1], FetchMode::Consume).unwrap();
            stack.end_branching(branching, 1).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    //ensure_return (Can be achieved over sub ock)
    //"A consumed slots can not be returned"

    #[test]
    fn return_unconsumed_test() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let block = stack.start_block();
            stack.provide_owned();
            stack.end_block(block, 1).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    fn return_unconsumed_test2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t = stack.introduce_new();
            let mut branching = stack.start_branching(2);
                stack.provide(t.clone()).unwrap();
            stack.next_branch(&mut branching, 1).unwrap();
                stack.provide(t).unwrap();
            stack.end_branching(branching, 1).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    // non_consuming_fetch
    // [Not] "Can not access already moved slot"
    #[test]
    fn access_elem() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_borrowed();
            assert_eq!(stack.value_of(ValueRef(0)).unwrap(), stack.get_type(0));
            (vec![stack.param(0)], vec![])
        });
    }

    #[test]
    fn access_elem2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_borrowed();
            stack.fetch(ValueRef(0), FetchMode::Copy).unwrap();
            (vec![stack.param(0)], vec![stack.owned(0)])
        });
    }

    //hide
    //[Not] "Can not access already moved slot"

    #[test]
    fn apply_once() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            stack.provide_borrowed();
            stack.consume_params(&[(ValueRef(1), true),(ValueRef(0), false)]).unwrap();
            (vec![stack.param(1), stack.param(0)], vec![])
        });
    }

    #[test]
    fn apply_once2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            stack.provide_owned();
            stack.consume_params(&[(ValueRef(1), true),(ValueRef(0), true)]).unwrap();
            (vec![stack.param(1), stack.param(0)], vec![])
        });
    }

    #[test]
    fn apply_once3() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_borrowed();
            stack.provide_borrowed();
            stack.consume_params(&[(ValueRef(1), false),(ValueRef(0), false)]).unwrap();
            (vec![stack.param(1), stack.param(0)], vec![])
        });
    }

    // ret
    // [not] "Can not handle element from outside of the active frame"
    // [not] "Consuming moved slot is forbidden"

    #[test]
    fn return_elems() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let block = stack.start_block();
            stack.provide_owned();
            stack.provide_owned();
            stack.provide_owned();
            stack.end_block(block,3).unwrap();
            (vec![], vec![stack.owned(2),stack.owned(1),stack.owned(0)])
        });
    }

    #[test]
    fn return_elems2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let t2 = stack.introduce_new();
            let mut branching = stack.start_branching(2);
                stack.provide(t0.clone()).unwrap();
                stack.provide(t1.clone()).unwrap();
                stack.provide(t2.clone()).unwrap();
            stack.next_branch(&mut branching,3).unwrap();
                stack.provide(t0.clone()).unwrap();
                stack.provide(t1.clone()).unwrap();
                stack.provide(t2.clone()).unwrap();
            stack.end_branching(branching,3).unwrap();
            (vec![], vec![stack.owned(2),stack.owned(1),stack.owned(0)])
        });
    }

    //ret_or_exit_branch
    //[not] "branches must induce the same post state"
    //[not] "branches must induce the same post state"

    #[test]
    fn return_elems3() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching(2);
                stack.provide(t0.clone()).unwrap();
                stack.provide(t1.clone()).unwrap();
            stack.next_branch(&mut branching,2).unwrap();
                stack.provide(t0.clone()).unwrap();
                stack.provide(t1.clone()).unwrap();
            stack.end_branching(branching,2).unwrap();
            (vec![], vec![stack.owned(1),stack.owned(0)])
        });
    }

    #[test]
    fn return_elems4() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            stack.provide_owned();
            let t = stack.introduce_new();
            let mut branching = stack.start_branching(2);
                stack.transform(ValueRef(1), t.clone(), FetchMode::Consume).unwrap();
                stack.transform(ValueRef(1), t.clone(), FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching,2).unwrap();
                stack.transform(ValueRef(1), t.clone(), FetchMode::Consume).unwrap();
                stack.transform(ValueRef(1), t.clone(), FetchMode::Consume).unwrap();
            stack.end_branching(branching,2).unwrap();
            (vec![stack.param(2), stack.param(1)], vec![stack.owned(0), stack.owned(0)])
        });
    }

    #[test]
    fn return_elems5() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            stack.provide_owned();
            let t = stack.introduce_new();
            let mut branching = stack.start_branching(2);
                stack.transform(ValueRef(1), t.clone(), FetchMode::Consume).unwrap();
                stack.transform(ValueRef(1), t.clone(), FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching,2).unwrap();
                stack.transform(ValueRef(0), t.clone(), FetchMode::Consume).unwrap();
                stack.transform(ValueRef(2), t.clone(), FetchMode::Consume).unwrap();
            stack.end_branching(branching,2).unwrap();
            (vec![stack.param(2), stack.param(1)], vec![stack.owned(0), stack.owned(0)])
        });
    }

    //test the base functions deeply
    //value_of
    #[test]
    fn value_of_test() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t = stack.introduce_new();
            stack.borrow(t.clone()).unwrap();
            assert_eq!(stack.value_of(ValueRef(0)).unwrap(),t);
            (vec![stack.param(0)], vec![])
        });
    }

    #[test]
    fn value_of_test2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t = stack.introduce_new();
            stack.borrow(t.clone()).unwrap();
            stack.fetch(ValueRef(0), FetchMode::Copy).unwrap();
            assert_eq!(stack.value_of(ValueRef(1)).unwrap(),t.clone());
            assert_eq!(stack.value_of(ValueRef(0)).unwrap(),t.clone());
            (vec![stack.param(0)], vec![stack.owned(0)])
        });
    }

    //copy
    #[test]
    fn copy_test() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_borrowed();
            stack.fetch(ValueRef(0), FetchMode::Copy).unwrap();
            (vec![stack.param(0)], vec![stack.owned(0)])
        });
    }

    #[test]
    fn copy_test2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_borrowed();
            stack.fetch(ValueRef(0), FetchMode::Copy).unwrap();
            stack.fetch(ValueRef(0), FetchMode::Copy).unwrap();
            stack.fetch(ValueRef(2), FetchMode::Copy).unwrap();
            stack.fetch(ValueRef(0), FetchMode::Copy).unwrap();
            (vec![stack.param(0)], vec![stack.owned(0), stack.owned(0), stack.owned(0), stack.owned(0)])
        });
    }

    //provide
    #[test]
    fn provide_test() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t = stack.introduce_new();
            stack.borrow(t).unwrap();
            (vec![stack.param(0)], vec![])
        });
    }

    #[test]
    fn provide_test2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t = stack.introduce_new();
            stack.provide(t).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    fn provide_test3() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.borrow(t0.clone()).unwrap();
            stack.borrow(t1.clone()).unwrap();
            stack.borrow(t0.clone()).unwrap();
            stack.borrow(t1.clone()).unwrap();
            (vec![stack.param(1), stack.param(0), stack.param(1), stack.param(0)], vec![])
        });
    }

    #[test]
    fn provide_test4() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.borrow(t0.clone()).unwrap();
            stack.borrow(t1.clone()).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t1.clone()).unwrap();
            (vec![stack.param(1), stack.param(0)], vec![stack.owned(1), stack.owned(0)])
        });
    }

    //drop
    #[test]
    fn drop_test() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            stack.drop(ValueRef(0)).unwrap();
            (vec![stack.param(0)], vec![])
        });
    }

    #[test]
    fn drop_test2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_borrowed();
            stack.provide_owned();
            stack.provide_borrowed();
            stack.provide_owned();
            stack.provide_borrowed();
            stack.drop(ValueRef(3)).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            (vec![stack.param(4), stack.param(3), stack.param(2), stack.param(1), stack.param(0)], vec![])
        });
    }


    // transform
    #[test]
    fn transform_test() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Consume).unwrap();
            (vec![stack.param(1)], vec![stack.owned(0)])
        });
    }

    //fetch
    #[test]
    fn fetch_test() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            stack.fetch(ValueRef(0),FetchMode::Consume).unwrap();
            (vec![stack.param(0)], vec![stack.owned(0)])
        });
    }


    //pack
    #[test]
    fn pack_test() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0)], t, FetchMode::Consume).unwrap();
            (vec![stack.param(1)], vec![stack.owned(0)])
        });
    }

    //pack
    #[test]
    fn pack_test2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            stack.provide_owned();
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0),ValueRef(1),ValueRef(2)], t, FetchMode::Consume).unwrap();
            (vec![stack.param(3), stack.param(2), stack.param(1)], vec![stack.owned(0)])
        });
    }

    //unpack
    #[test]
    fn unpack_test() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0),&[t],FetchMode::Consume).unwrap();
            (vec![stack.param(1)], vec![stack.owned(0)])
        });
    }

    //blocks
    #[test]
    fn block_test() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let block = stack.start_block();
            stack.provide_owned();
            stack.end_block(block, 1).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    fn block_test3() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            let block = stack.start_block();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Consume).unwrap();
            stack.end_block(block, 1).unwrap();
            (vec![stack.param(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    fn block_test5() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let block = stack.start_block();
            stack.provide_owned();
            stack.provide_owned();
            stack.fetch(ValueRef(0), FetchMode::Consume).unwrap();
            stack.fetch(ValueRef(2), FetchMode::Consume).unwrap();
            stack.end_block(block, 2).unwrap();
            (vec![], vec![stack.owned(0), stack.owned(1)])
        });
    }

    //branching
    #[test]
    fn branching_test() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching(3);
                stack.provide(t0.clone()).unwrap();
                stack.provide(t1.clone()).unwrap();
                stack.drop(ValueRef(0)).unwrap();
                stack.fetch(ValueRef(1), FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching, 1).unwrap();
                stack.provide(t1.clone()).unwrap();
                stack.provide(t0.clone()).unwrap();
                stack.drop(ValueRef(1)).unwrap();
            stack.next_branch(&mut branching, 1).unwrap();
                stack.provide(t0.clone()).unwrap();
            stack.end_branching(branching, 1).unwrap();
            (vec![], vec![stack.owned(1)])
        });
    }

    #[test]
    fn branching_test2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.provide(t1.clone()).unwrap();
            let mut branching = stack.start_branching(3);
                stack.provide(t0.clone()).unwrap();
                stack.transform(ValueRef(1), t1.clone(), FetchMode::Consume).unwrap();
                stack.drop(ValueRef(0)).unwrap();
                stack.fetch(ValueRef(1), FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching, 1).unwrap();
                stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
                stack.provide(t0.clone()).unwrap();
                stack.drop(ValueRef(1)).unwrap();
            stack.next_branch(&mut branching, 1).unwrap();
                stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
                stack.drop(ValueRef(0)).unwrap();
                stack.provide(t0.clone()).unwrap();
            stack.end_branching(branching, 1).unwrap();
            (vec![stack.param(0)], vec![stack.owned(1)])
        });
    }

    //apply
    #[test]
    fn apply_test() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            stack.provide_owned();
            stack.provide_borrowed();
            stack.provide_borrowed();
            let typ = stack.introduce_new();
            stack.consume_params(&[(ValueRef(0), false), (ValueRef(2), true)]).unwrap();
            stack.provide(typ.clone()).unwrap();
            (vec![stack.param(3), stack.param(2), stack.param(1)], vec![stack.owned(0)])
        });
    }

    //Make some nested Bigger tests touching on multiple aspects
    #[test]
    fn different_deps() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let t2 = stack.introduce_new();
            let mut br0 = stack.start_branching(3);
                let bl1 = stack.start_block();
                    stack.provide(t0.clone()).unwrap();
                stack.end_block(bl1, 1).unwrap();
            stack.next_branch(&mut br0, 1).unwrap();
                let mut br1 = stack.start_branching(2);
                    stack.provide(t1.clone()).unwrap();
                    stack.provide(t0.clone()).unwrap();
                    stack.fetch(ValueRef(0), FetchMode::Consume).unwrap();
                    stack.fetch(ValueRef(2), FetchMode::Consume).unwrap();
            stack.next_branch(&mut br1, 2).unwrap();
                    stack.provide(t0.clone()).unwrap();
                    stack.provide(t1.clone()).unwrap();
                    stack.provide(t2.clone()).unwrap();
                    stack.drop(ValueRef(0)).unwrap();
                    stack.fetch(ValueRef(2), FetchMode::Consume).unwrap();
                    stack.fetch(ValueRef(2), FetchMode::Consume).unwrap();
            stack.end_branching(br1, 2).unwrap();
                stack.drop(ValueRef(0)).unwrap();
                stack.fetch(ValueRef(1), FetchMode::Consume).unwrap();
            stack.next_branch(&mut br0,1).unwrap();
                stack.provide(t0.clone()).unwrap();
            stack.end_branching(br0,1).unwrap();
            (vec![], vec![stack.owned(2)])
        });
    }

    #[test]
    fn different_deps2() {
        let accounting = max_accounting();
        test(&accounting,|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let t2 = stack.introduce_new();
            stack.provide_owned();
            let mut br0 = stack.start_branching(3);
                let bl1 = stack.start_block();
                    stack.provide(t0.clone()).unwrap();
                    stack.drop(ValueRef(1)).unwrap();
                stack.end_block(bl1, 1).unwrap();
            stack.next_branch(&mut br0, 1).unwrap();
                let mut br1 = stack.start_branching(2);
                    stack.provide(t1.clone()).unwrap();
                    stack.provide(t0.clone()).unwrap();
                    stack.drop(ValueRef(2)).unwrap();
                    stack.fetch(ValueRef(0), FetchMode::Consume).unwrap();
                    stack.fetch(ValueRef(2), FetchMode::Consume).unwrap();
            stack.next_branch(&mut br1, 2).unwrap();
                    stack.provide(t0.clone()).unwrap();
                    stack.drop(ValueRef(1)).unwrap();
                    stack.provide(t1.clone()).unwrap();
                    stack.provide(t2.clone()).unwrap();
                    stack.drop(ValueRef(0)).unwrap();
                    stack.fetch(ValueRef(2), FetchMode::Consume).unwrap();
                    stack.fetch(ValueRef(2), FetchMode::Consume).unwrap();
            stack.end_branching(br1, 2).unwrap();
                stack.drop(ValueRef(0)).unwrap();
                stack.fetch(ValueRef(1), FetchMode::Consume).unwrap();
            stack.next_branch(&mut br0,1).unwrap();
                stack.drop(ValueRef(0)).unwrap();
                stack.provide(t0.clone()).unwrap();
            stack.end_branching(br0,1).unwrap();
            (vec![stack.param(0)], vec![stack.owned(3)])
        });
    }
}