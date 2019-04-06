

#[cfg(test)]
mod tests {
    use sanskrit_common::capabilities::CapSet;
    use sanskrit_deploy::linear_type_stack::LinearTypeStack;
    use sanskrit_common::linear_stack::*;
    use sanskrit_common::model::*;
    use sanskrit_deploy::linear_type_stack::ExprResult;
    use sanskrit_core::utils::*;
    use sanskrit_core::model::resolved::*;
    use std::ops::DerefMut;
    use std::ops::Deref;

    fn new_type(len:usize) -> Crc<ResolvedType> {
        assert!(len <= u8::max_value() as usize);
        Crc::new(ResolvedType::Generic {
            extended_caps: CapSet::empty(),
            caps: CapSet::empty(),
            offset: len as u8,
            is_phantom: false
        })
    }

    struct Tester {
        inner: LinearTypeStack,
        types: Vec<Crc<ResolvedType>>
    }

    impl Deref for Tester{
        type Target = LinearTypeStack;
        fn deref(&self) -> &LinearTypeStack {
            &self.inner
        }
    }

    impl DerefMut for Tester {
        fn deref_mut(&mut self) -> &mut LinearTypeStack{
            &mut self.inner
        }
    }

    impl Tester {
        pub fn new() -> Self {
            Tester{
                inner: LinearTypeStack::new(),
                types: Vec::new(),
            }
        }

        pub fn provide_new(&mut self) -> Crc<ResolvedType>{
            let elem = new_type(self.types.len());
            self.types.push(elem.clone());
            self.inner.provide(elem.clone()).unwrap();
            elem
        }

        pub fn introduce_new(&mut self) -> Crc<ResolvedType>{
            let elem = new_type(self.types.len());
            self.types.push(elem.clone());
            elem
        }


        pub fn get_type(&self, pos:usize) -> Crc<ResolvedType> {
            let len = self.types.len();
            self.types[len-pos-1].clone()
        }

        pub fn owned(&self, pos:usize) -> ResolvedReturn {
            ResolvedReturn {
                borrows: Vec::new(),
                typ: self.get_type(pos),
            }
        }

        pub fn borrowed(&self, pos:usize, borrows:Vec<ValueRef>) -> ResolvedReturn {
            ResolvedReturn {
                borrows,
                typ: self.get_type(pos),
            }
        }

        pub fn consumed(&self, pos:usize) -> ResolvedParam {
            ResolvedParam {
                consumes: true,
                typ: self.get_type(pos),
            }
        }


        pub fn untouched(&self, pos:usize) -> ResolvedParam {
            ResolvedParam {
                consumes: false,
                typ: self.get_type(pos),
            }
        }

    }

    fn test<F:FnOnce(&mut Tester) -> (Vec<ResolvedParam>, Vec<ResolvedReturn>)>(f:F){
        let mut tester = Tester::new();
        //Generate some garbage that should be untouched by test
        let mut params = Vec::with_capacity(20);
        for _ in 0..params.capacity() {
            tester.provide_new();
            params.push(false)
        }
        let (new_params, rets) = f(&mut tester);

        //Test types where we still allowed to
        let depth = new_params.len() + rets.len();
        for (i, ResolvedParam{typ, ..}) in new_params.iter().enumerate() {
            //consumed are no longer accessible
            assert_eq!(tester.value_of(ValueRef((depth-1-i) as u16)).unwrap(), *typ)
        }

        let depth = rets.len();
        for (i, ResolvedReturn{typ, ..}) in rets.iter().enumerate() {
            //consumed are no longer accessible
            match tester.value_of(ValueRef((depth-1-i) as u16)) {
                Ok(t) => assert_eq!(t, *typ),
                _ => {}
            }
        }

        let res = rets.iter().map(|r|&r.borrows[..]);
        params.extend(new_params.iter().map(|p|p.consumes));

        tester.inner.check_function_signature(params.into_iter(), res).unwrap();
    }

    #[test]
    fn test_type_individuality() {
        let t1 = new_type(1);
        let t2  = new_type(2);
        assert_ne!(t1,t2);
        assert_eq!(t1,t1);
        assert_eq!(t1,t1.clone());
        assert_eq!(t2,t2);
        assert_eq!(t2,t2.clone());
    }

    //First some test that provoke a specific error

    //consume
    // "Consuming moved slot is forbidden"
    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_twice_test() {
        test(|stack|{
            stack.provide_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            (vec![stack.consumed(0)], vec![])
        });
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_twice_test2() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0), &vec![t], FetchMode::Consume).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_twice_test3() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.unpack(ValueRef(0), &vec![t], FetchMode::Consume).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_twice_test4() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0)], t, FetchMode::Consume).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_twice_test5() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.pack(&[ValueRef(0)], t, FetchMode::Consume).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_twice_test6() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0),ValueRef(0)], t, FetchMode::Consume).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_twice_test7() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Consume).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_twice_test8() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.transform(ValueRef(0), t, FetchMode::Consume).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_borrow_test() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0), &vec![t], FetchMode::Borrow).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_borrow_test2() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0)], t, FetchMode::Borrow).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_twice() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0),ValueRef(0)], t, FetchMode::Consume).unwrap();
            (vec![], vec![])
        });
    }

    // "Consuming moved slot is forbidden"

    #[test]
    #[should_panic(expected = "Consuming moved slot is forbidden")]
    fn consume_borrowed_test() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0), &vec![t], FetchMode::Borrow).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected = "Consuming moved slot is forbidden")]
    fn consume_borrowed_test2() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0)], t, FetchMode::Borrow).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_borrow_test3() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    //lock
    //"Locking moved slot is forbidden"

    #[test]
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn borrow_consume_test() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn borrow_consume_test2() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.unpack(ValueRef(0), &vec![t], FetchMode::Borrow).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn borrow_consume_test3() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.pack(&[ValueRef(0)], t, FetchMode::Borrow).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn lock_twice() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0), ValueRef(0)], t, FetchMode::Borrow).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(1),ValueRef(1)])])
        });
    }

    //ensure_freed
    //"Only consumed and not locked elem slots can be freed"

    #[test]
    #[should_panic(expected="Only consumed and not locked elem slots can be freed")]
    fn free_locked() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0)], t, FetchMode::Borrow).unwrap();
            stack.free(ValueRef(1)).unwrap();
            (vec![stack.consumed(0)], vec![stack.borrowed(1, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="Only consumed and not locked elem slots can be freed")]
    fn free_locked2() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0), &[t], FetchMode::Borrow).unwrap();
            stack.free(ValueRef(1)).unwrap();
            (vec![stack.consumed(0)], vec![stack.borrowed(1, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="Only consumed and not locked elem slots can be freed")]
    fn free_locked3() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            stack.free(ValueRef(1)).unwrap();
            (vec![stack.consumed(0)], vec![stack.borrowed(1, vec![ValueRef(0)])])
        });
    }

    //"Only consumed and not locked elem slots can be freed"
    #[test]
    #[should_panic(expected="Only consumed and not locked elem slots can be freed")]
    fn free_unconsumed() {
        test(|stack|{
            stack.provide_new();
            stack.free(ValueRef(0)).unwrap();
            (vec![stack.consumed(0)], vec![])
        });
    }

    #[test]
    #[should_panic(expected="Only consumed and not locked elem slots can be freed")]
    fn free_unconsumed2() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            stack.end_block(block, ExprResult::Return(&[])).unwrap();
            (vec![], vec![])
        });
    }


    //ensure_return (Can be achieved over sub ock)
    //"A consumed slots can not be returned"
    #[test]
    #[should_panic(expected="A consumed slots can not be returned")]
    fn return_consumed() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Consume).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(1)])).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="A consumed slots can not be returned")]
    fn return_consumed2() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="A consumed slots can not be returned")]
    fn return_consumed3() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0), &[t], FetchMode::Consume).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(1)])).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    // absolute_index
    // "Targeted element lies outside of stack"

    #[test]
    #[should_panic(expected="Targeted element lies outside of stack")]
    fn reach_outside() {
        test(|stack|{
            stack.drop(ValueRef(20)).unwrap();
            (vec![], vec![])
        });
    }

    #[test]
    #[should_panic(expected="Targeted element lies outside of stack")]
    fn reach_outside2() {
        test(|stack|{
            let t = stack.introduce_new();
            stack.transform(ValueRef(20), t, FetchMode::Consume).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Targeted element lies outside of stack")]
    fn reach_outside3() {
        test(|stack|{
            let t = stack.introduce_new();
            stack.transform(ValueRef(20), t, FetchMode::Borrow).unwrap();
            (vec![], vec![stack.borrowed(0, vec![ValueRef(21)])])

        });
    }

    #[test]
    #[should_panic(expected="Targeted element lies outside of stack")]
    fn reach_outside4() {
        test(|stack|{
            let t = stack.introduce_new();
            stack.unpack(ValueRef(20), &[t], FetchMode::Consume).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Targeted element lies outside of stack")]
    fn reach_outside5() {
        test(|stack|{
            let t = stack.introduce_new();
            stack.unpack(ValueRef(20), &[t], FetchMode::Borrow).unwrap();
            (vec![], vec![stack.borrowed(0, vec![ValueRef(21)])])
        });
    }

    #[test]
    #[should_panic(expected="Targeted element lies outside of stack")]
    fn reach_outside6() {
        test(|stack|{
            stack.fetch(ValueRef(20), FetchMode::Copy).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Targeted element lies outside of stack")]
    fn reach_outside7() {
        test(|stack|{
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(20)], t, FetchMode::Consume).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Targeted element lies outside of stack")]
    fn reach_outside8() {
        test(|stack|{
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(20)], t, FetchMode::Borrow).unwrap();
            (vec![], vec![stack.borrowed(0, vec![ValueRef(21)])])
        });
    }

    //hide
    //"Can not access already moved slot"
    #[test]
    #[should_panic(expected="Can not access already moved slot")]
    fn hide_test() {
        test(|stack|{
            stack.provide_new();
            stack.apply(&[(ValueRef(0), false), (ValueRef(0), false)], &[]).unwrap();
            (vec![], vec![])
        });
    }

    //ret
    //"can not steal borrows"
    #[test]
    #[should_panic(expected="Only consumed and not locked elem slots can be freed")]
    fn steal_dropped_test() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0), &[t], FetchMode::Borrow).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="Only consumed and not locked elem slots can be freed")]
    fn steal_dropped_test2() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0)], t, FetchMode::Borrow).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="Only consumed and not locked elem slots can be freed")]
    fn steal_dropped_test3() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="Only consumed and not locked elem slots can be freed")]
    fn steal_dropped_test4() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            let t1 = stack.introduce_new();
            stack.transform(ValueRef(0), t1, FetchMode::Borrow).unwrap();
            let t2 = stack.introduce_new();
            stack.transform(ValueRef(0), t2, FetchMode::Borrow).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.borrowed(1, vec![ValueRef(0)]), stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="Only consumed and not locked elem slots can be freed")]
    fn steal_dropped_test5() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            let t1 = stack.introduce_new();
            stack.transform(ValueRef(0), t1, FetchMode::Borrow).unwrap();
            let t2 = stack.introduce_new();
            stack.transform(ValueRef(0), t2, FetchMode::Borrow).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    //"Can not borrow from a later element"

    #[test]
    #[should_panic(expected="Can not borrow from a later element")]
    fn steal_previous_test() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0), &[t], FetchMode::Borrow).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0),ValueRef(1)])).unwrap();
            (vec![], vec![stack.borrowed(0, vec![ValueRef(0)]),stack.owned(1)])
        });
    }

    #[test]
    #[should_panic(expected="Can not borrow from a later element")]
    fn steal_previous_test2() {
        test(|stack| {
            let block = stack.start_block();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0), ValueRef(1)])).unwrap();
            (vec![], vec![stack.borrowed(0, vec![ValueRef(0)]), stack.owned(1)])
        });
    }

    #[test]
    #[should_panic(expected="Can not borrow from a later element")]
    fn steal_previous_test3() {
        test(|stack| {
            let block = stack.start_block();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0)], t, FetchMode::Borrow).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0), ValueRef(1)])).unwrap();
            (vec![], vec![stack.borrowed(0, vec![ValueRef(0)]), stack.owned(1)])
        });
    }

    // ret
    // "Can not handle element from outside of the active frame"

    #[test]
    #[should_panic(expected="Can not handle element from outside of the active frame")]
    fn out_of_frame_return_test() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            stack.end_block(block, ExprResult::Return(&[ValueRef(1)])).unwrap();
            (vec![], vec![stack.owned(1)])
        });
    }

    #[test]
    #[should_panic(expected="Can not handle element from outside of the active frame")]
    fn out_of_frame_return_test2() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            stack.end_block(block, ExprResult::Return(&[ValueRef(1),ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(1)])
        });
    }

    #[test]
    #[should_panic(expected="Can not handle element from outside of the active frame")]
    fn out_of_frame_return_test3() {
        test(|stack|{
            let block = stack.start_block();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    //"Consuming moved slot is forbidden"
    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn double_frame_return_test() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0),ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(0),stack.owned(0)])
        });
    }

    //ret_or_exit_branch
    //"branches must induce the same post state"

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn mismatching_branch_return_test() {
        test(|stack|{
            let mut branching = stack.start_branching();
            stack.provide_new();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.provide_new();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn mismatching_branch_return_test2() {
        test(|stack|{
            stack.provide_new();
            let mut branching = stack.start_branching();
            let t1 = stack.introduce_new();
            stack.transform(ValueRef(0), t1, FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            let t2 = stack.introduce_new();
            stack.transform(ValueRef(0), t2, FetchMode::Consume).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn mismatching_branch_return_test3() {
        test(|stack|{
            stack.provide_new();
            let mut branching = stack.start_branching();
            let t1 = stack.introduce_new();
            stack.transform(ValueRef(0), t1, FetchMode::Borrow).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            let t2 = stack.introduce_new();
            stack.transform(ValueRef(0), t2, FetchMode::Borrow).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn mismatching_branch_return_test4() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Borrow).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(2),ValueRef(1),ValueRef(0)])).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.transform(ValueRef(1), t1.clone(), FetchMode::Borrow).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(2),ValueRef(1),ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(1),stack.owned(1),stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn mismatching_branch_return_test5() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.transform(ValueRef(1), t1.clone(), FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(1),ValueRef(0)])).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(1),stack.owned(1)])
        });
    }

    //"branches must induce the same post state"
    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn mismatching_branch_capture_test() {
        test(|stack|{
            let t0 = stack.introduce_new();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t0.clone()).unwrap();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.transform(ValueRef(1), t1.clone(), FetchMode::Consume).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.consumed(1),stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn mismatching_branch_capture_test2() {
        test(|stack|{
            let t0 = stack.introduce_new();
            stack.provide(t0.clone()).unwrap();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    // check_function_signature
    // "Number of params is wrong"


    #[test]
    #[should_panic(expected="Number of params is wrong")]
    fn signature_size_missmatch() {
        test(|stack|{
            stack.provide_new();
            (vec![], vec![])
        });
    }


    #[test]
    #[should_panic(expected="Number of params is wrong")]
    fn signature_size_missmatch2() {
        test(|stack|{
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Number of params is wrong")]
    fn signature_size_missmatch3() {
        test(|stack|{
            (vec![stack.untouched(0)], vec![])
        });
    }

    //"Function signature mismatch"
    #[test]
    #[should_panic(expected="Function signature mismatch")]
    fn signature_result_mode_missmatch() {
        test(|stack|{
            stack.provide_new();
            stack.drop(ValueRef(0)).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    #[should_panic(expected="Function signature mismatch")]
    fn signature_result_mode_missmatch2() {
        test(|stack|{
            stack.provide_new();
            stack.drop(ValueRef(0)).unwrap();
            (vec![], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    //"Function signature mismatch"

    #[test]
    #[should_panic(expected="Function signature mismatch")]
    fn signature_result_borrow_missmatch() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(1)])])
        });
    }

    #[test]
    #[should_panic(expected="Function signature mismatch")]
    fn signature_result_borrow_missmatch2() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0),ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="Function signature mismatch")]
    fn signature_result_borrow_missmatch3() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![])])
        });
    }

    //"Function signature mismatch"
    #[test]
    #[should_panic(expected="Function signature mismatch")]
    fn signature_param_mode_missmatch() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            (vec![stack.consumed(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="Function signature mismatch")]
    fn signature_param_mode_missmatch2() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Consume).unwrap();
            (vec![stack.untouched(1)], vec![stack.owned(0)])
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
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Consume).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    fn consume_slot2() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0)], t, FetchMode::Consume).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    fn consume_slot3() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(1),ValueRef(0)], t, FetchMode::Consume).unwrap();
            (vec![stack.consumed(2), stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    fn consume_slot4() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0), &[t], FetchMode::Consume).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }


    #[test]
    fn consume_slot5() {
        test(|stack|{
            stack.provide_new();
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.unpack(ValueRef(0), &[t0, t1], FetchMode::Consume).unwrap();
            (vec![stack.consumed(2)], vec![stack.owned(1), stack.owned(0)])
        });
    }

    #[test]
    fn consume_slot6() {
        test(|stack|{
            stack.provide_new();
            stack.drop(ValueRef(0)).unwrap();
            (vec![stack.consumed(0)], vec![])
        });
    }

    //lock
    //"Locking moved slot is forbidden"

    #[test]
    fn lock_slot() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    fn lock_slot2() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0)], t, FetchMode::Borrow).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    fn lock_slot3() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(1), ValueRef(0)], t, FetchMode::Borrow).unwrap();
            (vec![stack.untouched(2), stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(1),ValueRef(0)])])
        });
    }

    #[test]
    fn lock_slot4() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0), &[t], FetchMode::Borrow).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }


    #[test]
    fn lock_slot5() {
        test(|stack|{
            stack.provide_new();
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.unpack(ValueRef(0), &[t0, t1], FetchMode::Borrow).unwrap();
            (vec![stack.untouched(2)], vec![stack.borrowed(1, vec![ValueRef(0)]),stack.borrowed(0, vec![ValueRef(1)])])
        });
    }


    //ensure_freed
    //[not] "Only consumed and not locked elem slots can be freed"
    //[not] "Only consumed and not locked elem slots can be freed"

    #[test]
    fn free_consumed() {
        test(|stack|{
            stack.provide_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.free(ValueRef(0)).unwrap();
            (vec![stack.consumed(0)], vec![])
        });
    }

    #[test]
    fn free_consumed2() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Consume).unwrap();
            stack.free(ValueRef(1)).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    fn free_consumed3() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0)], t, FetchMode::Consume).unwrap();
            stack.free(ValueRef(1)).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    fn free_consumed4() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0), &[t], FetchMode::Consume).unwrap();
            stack.free(ValueRef(1)).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    fn free_consumed5() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.end_block(block, ExprResult::Return(&[])).unwrap();
            (vec![], vec![])
        });
    }

    #[test]
    fn free_consumed7() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Consume).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    fn free_consumed8() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0)], t, FetchMode::Consume).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    fn free_consumed9() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0), &[t], FetchMode::Consume).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    fn free_consumed10() {
        test(|stack|{
            let mut branching = stack.start_branching();
            stack.provide_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[])).unwrap();
            stack.provide_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[])).unwrap();
            (vec![], vec![])
        });
    }

    #[test]
    fn free_consumed11() {
        test(|stack|{
            let mut branching = stack.start_branching();
            stack.provide_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[])).unwrap();
            stack.provide_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[])).unwrap();
            (vec![], vec![])
        });
    }

    #[test]
    fn free_consumed12() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching();
            stack.provide(t0.clone()).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.provide(t0).unwrap();
            stack.transform(ValueRef(0), t1, FetchMode::Consume).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    fn free_consumed13() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching();
            stack.provide(t0.clone()).unwrap();
            stack.pack(&[ValueRef(0)], t1.clone(), FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.provide(t0).unwrap();
            stack.pack(&[ValueRef(0)], t1, FetchMode::Consume).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    fn free_consumed14() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching();
            stack.provide(t0.clone()).unwrap();
            stack.unpack(ValueRef(0), &[t1.clone()], FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.provide(t0).unwrap();
            stack.unpack(ValueRef(0), &[t1], FetchMode::Consume).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    //ensure_return (Can be achieved over sub ock)
    //"A consumed slots can not be returned"

    #[test]
    fn return_unconsumed_test() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    fn return_unconsumed_test2() {
        test(|stack|{
            let t = stack.introduce_new();
            let mut branching = stack.start_branching();
            stack.provide(t.clone()).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.provide(t).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    // non_consuming_fetch
    // [Not] "Can not access already moved slot"
    #[test]
    fn access_elem() {
        test(|stack|{
            stack.provide_new();
            assert_eq!(stack.value_of(ValueRef(0)).unwrap(), stack.get_type(0));
            (vec![stack.untouched(0)], vec![])
        });
    }


    #[test]
    fn access_elem2() {
        test(|stack|{
            stack.provide_new();
            assert!(!stack.is_borrowed(ValueRef(0)).unwrap());
            (vec![stack.untouched(0)], vec![])
        });
    }

    #[test]
    fn access_elem3() {
        test(|stack|{
            stack.provide_new();
            stack.fetch(ValueRef(0), FetchMode::Copy).unwrap();
            (vec![stack.untouched(0)], vec![stack.owned(0)])
        });
    }

    //hide
    //[Not] "Can not access already moved slot"

    #[test]
    fn apply_once() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            stack.apply(&[(ValueRef(1), true),(ValueRef(0), false)], &[]).unwrap();
            (vec![stack.consumed(1), stack.untouched(0)], vec![])
        });
    }

    #[test]
    fn apply_once2() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            stack.apply(&[(ValueRef(1), true),(ValueRef(0), true)], &[]).unwrap();
            (vec![stack.consumed(1), stack.consumed(0)], vec![])
        });
    }

    #[test]
    fn apply_once3() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            stack.apply(&[(ValueRef(1), false),(ValueRef(0), false)], &[]).unwrap();
            (vec![stack.untouched(1), stack.untouched(0)], vec![])
        });
    }

    //steal_ret
    //[not] "can not steal borrows"
    //[not] "can not steal borrows"

    #[test]
    fn steal_borrows() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t.clone(), FetchMode::Borrow).unwrap();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            stack.steal(ValueRef(0),ValueRef(1)).unwrap();
            stack.end_block(block,ExprResult::Return(&[ValueRef(2), ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(1), stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    fn steal_borrows2() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t.clone(), FetchMode::Borrow).unwrap();
            stack.transform(ValueRef(0), t.clone(), FetchMode::Borrow).unwrap();
            stack.transform(ValueRef(0), t.clone(), FetchMode::Borrow).unwrap();
            stack.transform(ValueRef(0), t.clone(), FetchMode::Borrow).unwrap();
            stack.transform(ValueRef(0), t.clone(), FetchMode::Borrow).unwrap();
            stack.steal(ValueRef(0),ValueRef(1)).unwrap();
            stack.steal(ValueRef(0),ValueRef(2)).unwrap();
            stack.steal(ValueRef(0),ValueRef(3)).unwrap();
            stack.steal(ValueRef(0),ValueRef(4)).unwrap();
            stack.end_block(block,ExprResult::Return(&[ValueRef(5), ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(1), stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    fn steal_borrows3() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t.clone(), FetchMode::Borrow).unwrap();
            stack.transform(ValueRef(0), t.clone(), FetchMode::Borrow).unwrap();
            stack.transform(ValueRef(0), t.clone(), FetchMode::Borrow).unwrap();
            stack.transform(ValueRef(0), t.clone(), FetchMode::Borrow).unwrap();
            stack.transform(ValueRef(0), t.clone(), FetchMode::Borrow).unwrap();
            stack.transform(ValueRef(0), t.clone(), FetchMode::Borrow).unwrap();
            stack.steal(ValueRef(0),ValueRef(1)).unwrap();
            stack.steal(ValueRef(2),ValueRef(3)).unwrap();
            stack.steal(ValueRef(4),ValueRef(5)).unwrap();
            stack.end_block(block,ExprResult::Return(&[ValueRef(6), ValueRef(4), ValueRef(2), ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(1), stack.borrowed(0, vec![ValueRef(0)]), stack.borrowed(0, vec![ValueRef(0)]), stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    fn steal_borrows4() {
        test(|stack|{
            stack.provide_new();
            let block = stack.start_block();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t.clone(), FetchMode::Borrow).unwrap();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            stack.steal(ValueRef(0),ValueRef(1)).unwrap();
            stack.end_block(block,ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    fn steal_borrows5() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            let block = stack.start_block();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(2),ValueRef(1),ValueRef(0)], t.clone(), FetchMode::Borrow).unwrap();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            stack.steal(ValueRef(0),ValueRef(1)).unwrap();
            stack.end_block(block,ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.untouched(3),stack.untouched(2),stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(2),ValueRef(1),ValueRef(0)])])
        });
    }

    #[test]
    fn steal_borrows6() {
        test(|stack|{
            stack.provide_new();
            let block = stack.start_block();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0),&[t.clone(),t.clone(),t.clone()],FetchMode::Borrow).unwrap();
            stack.transform(ValueRef(2), t.clone(), FetchMode::Borrow).unwrap();
            stack.transform(ValueRef(2), t.clone(), FetchMode::Borrow).unwrap();
            stack.transform(ValueRef(2), t.clone(), FetchMode::Borrow).unwrap();
            stack.steal(ValueRef(0),ValueRef(3)).unwrap();
            stack.steal(ValueRef(1),ValueRef(4)).unwrap();
            stack.steal(ValueRef(2),ValueRef(5)).unwrap();
            stack.end_block(block,ExprResult::Return(&[ValueRef(2),ValueRef(1),ValueRef(0)])).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)]),stack.borrowed(0, vec![ValueRef(1)]),stack.borrowed(0, vec![ValueRef(2)])])
        });
    }

    // ret
    // [not] "Can not handle element from outside of the active frame"
    // [not] "Consuming moved slot is forbidden"

    #[test]
    fn return_elems() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            stack.end_block(block,ExprResult::Return(&[ValueRef(2),ValueRef(1),ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(2),stack.owned(1),stack.owned(0)])
        });
    }

    #[test]
    fn return_elems2() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let t2 = stack.introduce_new();
            let mut branching = stack.start_branching();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t1.clone()).unwrap();
            stack.provide(t2.clone()).unwrap();
            stack.next_branch(&mut branching,ExprResult::Return(&[ValueRef(2),ValueRef(1),ValueRef(0)])).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t1.clone()).unwrap();
            stack.provide(t2.clone()).unwrap();
            stack.end_branching(branching,ExprResult::Return(&[ValueRef(2),ValueRef(1),ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(2),stack.owned(1),stack.owned(0)])
        });
    }

    //ret_or_exit_branch
    //[not] "branches must induce the same post state"
    //[not] "branches must induce the same post state"

    #[test]
    fn return_elems3() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t1.clone()).unwrap();
            stack.next_branch(&mut branching,ExprResult::Return(&[ValueRef(1),ValueRef(0)])).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t1.clone()).unwrap();
            stack.end_branching(branching,ExprResult::Return(&[ValueRef(1),ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(1),stack.owned(0)])
        });
    }

    #[test]
    fn return_elems4() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            let t = stack.introduce_new();
            let mut branching = stack.start_branching();
            stack.transform(ValueRef(1), t.clone(), FetchMode::Consume).unwrap();
            stack.transform(ValueRef(1), t.clone(), FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching,ExprResult::Return(&[ValueRef(1),ValueRef(0)])).unwrap();
            stack.transform(ValueRef(1), t.clone(), FetchMode::Consume).unwrap();
            stack.transform(ValueRef(1), t.clone(), FetchMode::Consume).unwrap();
            stack.end_branching(branching,ExprResult::Return(&[ValueRef(1),ValueRef(0)])).unwrap();
            (vec![stack.consumed(2),stack.consumed(1)], vec![stack.owned(0),stack.owned(0)])
        });
    }

    #[test]
    fn return_elems5() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            let t = stack.introduce_new();
            let mut branching = stack.start_branching();
            stack.transform(ValueRef(1), t.clone(), FetchMode::Consume).unwrap();
            stack.transform(ValueRef(1), t.clone(), FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching,ExprResult::Return(&[ValueRef(1),ValueRef(0)])).unwrap();
            stack.transform(ValueRef(0), t.clone(), FetchMode::Consume).unwrap();
            stack.transform(ValueRef(2), t.clone(), FetchMode::Consume).unwrap();
            stack.end_branching(branching,ExprResult::Return(&[ValueRef(1),ValueRef(0)])).unwrap();
            (vec![stack.consumed(2),stack.consumed(1)], vec![stack.owned(0),stack.owned(0)])
        });
    }

    #[test]
    fn return_elems6() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            let mut branching = stack.start_branching();
            stack.transform(ValueRef(0), t.clone(), FetchMode::Borrow).unwrap();
            stack.next_branch(&mut branching,ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.transform(ValueRef(0), t.clone(), FetchMode::Borrow).unwrap();
            stack.end_branching(branching,ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    //test the base functions deeply
    //value_of
    #[test]
    fn value_of_test() {
        test(|stack|{
            let t = stack.introduce_new();
            stack.provide(t.clone()).unwrap();
            assert_eq!(stack.value_of(ValueRef(0)).unwrap(),t);
            (vec![stack.untouched(0)], vec![])
        });
    }

    #[test]
    fn value_of_test2() {
        test(|stack|{
            let t = stack.introduce_new();
            stack.provide(t.clone()).unwrap();
            stack.fetch(ValueRef(0), FetchMode::Copy).unwrap();
            assert_eq!(stack.value_of(ValueRef(1)).unwrap(),t.clone());
            assert_eq!(stack.value_of(ValueRef(0)).unwrap(),t.clone());
            (vec![stack.untouched(0),stack.untouched(0)], vec![])
        });
    }

    //is_borrowed
    #[test]
    fn is_borrowed_test() {
        test(|stack|{
            stack.provide_new();
            assert!(!stack.is_borrowed(ValueRef(0)).unwrap());
            (vec![stack.untouched(0)], vec![])
        });
    }

    #[test]
    fn is_borrowed_test2() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            assert!(stack.is_borrowed(ValueRef(0)).unwrap());
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    fn is_borrowed_test3() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0)], t, FetchMode::Borrow).unwrap();
            assert!(stack.is_borrowed(ValueRef(0)).unwrap());
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    fn is_borrowed_test4() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0),&[t], FetchMode::Borrow).unwrap();
            assert!(stack.is_borrowed(ValueRef(0)).unwrap());
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }


    //copy
    #[test]
    fn copy_test() {
        test(|stack|{
            stack.provide_new();
            stack.fetch(ValueRef(0), FetchMode::Copy).unwrap();
            (vec![stack.untouched(0)], vec![stack.owned(0)])
        });
    }

    #[test]
    fn copy_test2() {
        test(|stack|{
            stack.provide_new();
            stack.fetch(ValueRef(0), FetchMode::Copy).unwrap();
            stack.fetch(ValueRef(0), FetchMode::Copy).unwrap();
            stack.fetch(ValueRef(2), FetchMode::Copy).unwrap();
            stack.fetch(ValueRef(0), FetchMode::Copy).unwrap();
            (vec![stack.untouched(0)], vec![stack.owned(0),stack.owned(0),stack.owned(0),stack.owned(0)])
        });
    }

    //provide
    #[test]
    fn provide_test() {
        test(|stack|{
            let t = stack.introduce_new();
            stack.provide(t).unwrap();
            (vec![stack.untouched(0)], vec![])
        });
    }

    #[test]
    fn provide_test2() {
        test(|stack|{
            let t = stack.introduce_new();
            stack.provide(t).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    fn provide_test3() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t1.clone()).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t1.clone()).unwrap();
            (vec![stack.untouched(1),stack.untouched(0),stack.untouched(1),stack.untouched(0)], vec![])
        });
    }

    #[test]
    fn provide_test4() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t1.clone()).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t1.clone()).unwrap();
            (vec![stack.untouched(1),stack.untouched(0)], vec![stack.owned(1),stack.owned(0)])
        });
    }

    //free
    #[test]
    fn free_test() {
        test(|stack|{
            stack.provide_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.free(ValueRef(0)).unwrap();
            (vec![stack.consumed(0)], vec![])
        });
    }

    #[test]
    fn free_test2() {
        test(|stack|{
            stack.provide_new();
            let t0 = stack.introduce_new();
            stack.transform(ValueRef(0), t0, FetchMode::Borrow).unwrap();
            stack.free(ValueRef(0)).unwrap();
            (vec![stack.untouched(1), stack.consumed(0)], vec![])
        });
    }

    //drop
    #[test]
    fn drop_test() {
        test(|stack|{
            stack.provide_new();
            stack.drop(ValueRef(0)).unwrap();
            (vec![stack.consumed(0)], vec![])
        });
    }

    #[test]
    fn drop_test2() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            stack.drop(ValueRef(3)).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            (vec![stack.untouched(4),stack.consumed(3),stack.untouched(2),stack.consumed(1),stack.untouched(0)], vec![])
        });
    }


    // transform
    #[test]
    fn transform_test() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Consume).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }


    #[test]
    fn transform_test2() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    fn transform_test3() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.transform(ValueRef(3), t0, FetchMode::Consume).unwrap();
            stack.transform(ValueRef(2), t1, FetchMode::Borrow).unwrap();
            (vec![stack.untouched(6),stack.consumed(5),stack.untouched(4),stack.untouched(3),stack.untouched(2)],
             vec![stack.owned(1),stack.borrowed(0, vec![ValueRef(2)])])
        });
    }

    //fetch
    #[test]
    fn fetch_test() {
        test(|stack|{
            stack.provide_new();
            stack.fetch(ValueRef(0),FetchMode::Consume).unwrap();
            (vec![stack.consumed(0)], vec![stack.owned(0)])
        });
    }

    #[test]
    fn fetch_test2() {
        test(|stack|{
            stack.provide_new();
            stack.fetch(ValueRef(0),FetchMode::Borrow).unwrap();
            (vec![stack.untouched(0)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    fn fetch_test3() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            stack.fetch(ValueRef(3),FetchMode::Consume).unwrap();
            stack.fetch(ValueRef(2),FetchMode::Borrow).unwrap();
            (vec![stack.untouched(4),stack.consumed(3),stack.untouched(2),stack.untouched(1),stack.untouched(0)],
             vec![stack.owned(3),stack.borrowed(1, vec![ValueRef(2)])])
        });
    }

    //pack
    #[test]
    fn pack_test() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0)], t, FetchMode::Consume).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }


    #[test]
    fn pack_test2() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0)], t, FetchMode::Borrow).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    fn pack_test3() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.pack(&[ValueRef(3)], t0, FetchMode::Consume).unwrap();
            stack.pack(&[ValueRef(2)], t1, FetchMode::Borrow).unwrap();
            (vec![stack.untouched(6),stack.consumed(5),stack.untouched(4),stack.untouched(3),stack.untouched(2)],
             vec![stack.owned(1),stack.borrowed(0, vec![ValueRef(2)])])
        });
    }

    //pack
    #[test]
    fn pack_test4() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0),ValueRef(1),ValueRef(2)], t, FetchMode::Consume).unwrap();
            (vec![stack.consumed(3),stack.consumed(2),stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    fn pack_test5() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.pack(&[ValueRef(0),ValueRef(1),ValueRef(2)], t, FetchMode::Borrow).unwrap();
            (vec![stack.untouched(3),stack.untouched(2),stack.untouched(1)],
             vec![stack.borrowed(0, vec![ValueRef(2),ValueRef(1),ValueRef(0)])])
        });
    }

    //unpack
    #[test]
    fn unpack_test() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0),&[t],FetchMode::Consume).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }


    #[test]
    fn unpack_test2() {
        test(|stack|{
            stack.provide_new();
            let t = stack.introduce_new();
            stack.unpack(ValueRef(0),&[t],FetchMode::Borrow).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    fn unpack_test3() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.unpack(ValueRef(3),&[t0],FetchMode::Consume).unwrap();
            stack.unpack(ValueRef(2),&[t1],FetchMode::Borrow).unwrap();
            (vec![stack.untouched(6),stack.consumed(5),stack.untouched(4),stack.untouched(3),stack.untouched(2)],
             vec![stack.owned(1),stack.borrowed(0, vec![ValueRef(2)])])
        });
    }

    #[test]
    fn unpack_test4() {
        test(|stack|{
            stack.provide_new();
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let t2 = stack.introduce_new();
            stack.unpack(ValueRef(0),&[t0,t1,t2],FetchMode::Consume).unwrap();
            (vec![stack.consumed(3)], vec![stack.owned(2),stack.owned(1),stack.owned(0)])
        });
    }

    #[test]
    fn unpack_test5() {
        test(|stack|{
            stack.provide_new();
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let t2 = stack.introduce_new();
            stack.unpack(ValueRef(0),&[t0,t1,t2],FetchMode::Borrow).unwrap();
            (vec![stack.untouched(3)],
             vec![stack.borrowed(2, vec![ValueRef(0)]),stack.borrowed(1, vec![ValueRef(1)]),stack.borrowed(0, vec![ValueRef(2)])])
        });
    }

    //blocks
    #[test]
    fn block_test() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(0)])
        });
    }

    #[test]
    fn block_test2() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            stack.end_block(block, ExprResult::Throw).unwrap();
            //Note: after a Throw Block Stack state is no really defined -- This is ok as nothing can happen afterwards
            (vec![], vec![])
        });
    }

    #[test]
    fn block_test3() {
        test(|stack|{
            stack.provide_new();
            let block = stack.start_block();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Consume).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.consumed(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    fn block_test4() {
        test(|stack|{
            stack.provide_new();
            let block = stack.start_block();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Consume).unwrap();
            stack.end_block(block, ExprResult::Throw).unwrap();
            //Note: after a Throw Block Stack state is no really defined -- This is ok as nothing can happen afterwards
            (vec![stack.consumed(1)], vec![])
        });
    }

    #[test]
    fn block_test5() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(1),ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(1), stack.borrowed(0,vec![ValueRef(0)])])
        });
    }

    #[test]
    fn block_test5_extra() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            stack.provide_new();
            stack.drop(ValueRef(0)).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(2),ValueRef(1)])).unwrap();
            (vec![], vec![stack.owned(2), stack.borrowed(1,vec![ValueRef(0)])])
        });
    }

    #[test]
    fn block_test6() {
        test(|stack|{
            stack.provide_new();
            let block = stack.start_block();
            let t = stack.introduce_new();
            stack.transform(ValueRef(0), t, FetchMode::Borrow).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0,vec![ValueRef(0)])])
        });
    }

    #[test]
    fn block_test7() {
        test(|stack|{
            let block = stack.start_block();
            stack.provide_new();
            stack.provide_new();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0),ValueRef(1)])).unwrap();
            (vec![], vec![stack.owned(0), stack.owned(1)])
        });
    }

    //branching
    #[test]
    fn branching_test() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t1.clone()).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(1)])).unwrap();
            stack.provide(t1.clone()).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(1)])
        });
    }

    #[test]
    fn branching_test2() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.provide(t1.clone()).unwrap();
            let mut branching = stack.start_branching();
            stack.provide(t0.clone()).unwrap();
            stack.transform(ValueRef(1), t1.clone(), FetchMode::Consume).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(1)])).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.consumed(0)], vec![stack.owned(1)])
        });
    }

    #[test]
    fn branching_test3() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.provide(t1.clone()).unwrap();
            let mut branching = stack.start_branching();
            stack.next_branch(&mut branching, ExprResult::Throw).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.consumed(0)], vec![stack.owned(1)])
        });
    }

    #[test]
    fn branching_test4() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.provide(t1.clone()).unwrap();
            let mut branching = stack.start_branching();
            stack.provide(t0.clone()).unwrap();
            stack.transform(ValueRef(1), t1.clone(), FetchMode::Consume).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(1)])).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.next_branch(&mut branching, ExprResult::Throw).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.consumed(0)], vec![stack.owned(1)])
        });
    }

    #[test]
    fn branching_test5() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.provide(t1.clone()).unwrap();
            let mut branching = stack.start_branching();
            stack.provide(t0.clone()).unwrap();
            stack.transform(ValueRef(1), t1.clone(), FetchMode::Consume).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(1)])).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.end_branching(branching, ExprResult::Throw).unwrap();
            (vec![stack.consumed(0)], vec![stack.owned(1)])
        });
    }

    #[test]
    fn branching_test6() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.provide(t1.clone()).unwrap();
            let mut branching = stack.start_branching();
            stack.transform(ValueRef(1), t1.clone(), FetchMode::Consume).unwrap();
            stack.next_branch(&mut branching, ExprResult::Throw).unwrap();
            stack.next_branch(&mut branching, ExprResult::Throw).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Consume).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.end_branching(branching, ExprResult::Throw).unwrap();
            (vec![stack.untouched(0)], vec![])
        });
    }

    #[test]
    fn branching_test7() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let mut branching = stack.start_branching();
            stack.provide(t0.clone()).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Borrow).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(1),ValueRef(0)])).unwrap();
            stack.provide(t1.clone()).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Borrow).unwrap();
            stack.drop(ValueRef(2)).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(1),ValueRef(0)])).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t1.clone()).unwrap();
            stack.transform(ValueRef(1), t1.clone(), FetchMode::Borrow).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(2),ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(1), stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    fn branching_test8() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.provide(t0.clone()).unwrap();
            let mut branching = stack.start_branching();
            stack.provide(t0.clone()).unwrap();
            stack.transform(ValueRef(1), t1.clone(), FetchMode::Borrow).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Borrow).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Borrow).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(1)])).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0,vec![ValueRef(0)])])
        });
    }

    #[test]
    fn branching_test9() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.provide(t0.clone()).unwrap();
            let mut branching = stack.start_branching();
            stack.provide(t0.clone()).unwrap();
            stack.transform(ValueRef(1), t1.clone(), FetchMode::Borrow).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            stack.next_branch(&mut branching, ExprResult::Throw).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Borrow).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Borrow).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(1)])).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0,vec![ValueRef(0)])])
        });
    }

    #[test]
    fn branching_test10() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.provide(t0.clone()).unwrap();
            let mut branching = stack.start_branching();
            stack.provide(t0.clone()).unwrap();
            stack.transform(ValueRef(1), t1.clone(), FetchMode::Borrow).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.next_branch(&mut branching, ExprResult::Throw).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Borrow).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            stack.end_branching(branching, ExprResult::Return(&[ValueRef(1)])).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0,vec![ValueRef(0)])])
        });
    }

    #[test]
    fn branching_test11() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.provide(t0.clone()).unwrap();
            let mut branching = stack.start_branching();
            stack.provide(t0.clone()).unwrap();
            stack.transform(ValueRef(1), t1.clone(), FetchMode::Borrow).unwrap();
            stack.drop(ValueRef(1)).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Borrow).unwrap();
            stack.next_branch(&mut branching, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.transform(ValueRef(0), t1.clone(), FetchMode::Borrow).unwrap();
            stack.provide(t0.clone()).unwrap();
            stack.end_branching(branching, ExprResult::Throw).unwrap();
            (vec![stack.untouched(1)], vec![stack.borrowed(0,vec![ValueRef(0)])])
        });
    }

    //apply
    #[test]
    fn apply_test() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            let typ = stack.introduce_new();
            stack.apply(
                &[(ValueRef(0), false), (ValueRef(2), true)],
                &[( typ, &[])],
            ).unwrap();
            (vec![stack.consumed(3),stack.untouched(2),stack.untouched(1)], vec![stack.owned(0)])
        });
    }

    #[test]
    fn apply_test2() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            let typ = stack.introduce_new();
            stack.apply(
                &[(ValueRef(0), false), (ValueRef(2), true)],
                &[(typ, &[ValueRef(1)])],
            ).unwrap();
            (vec![stack.consumed(3),stack.untouched(2),stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0)])])
        });
    }

    #[test]
    fn apply_test3() {
        test(|stack|{
            stack.provide_new();
            stack.provide_new();
            stack.provide_new();
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            stack.apply(
                &[(ValueRef(0), false), (ValueRef(2), true), (ValueRef(1), true)],
                &[(t0, &[ValueRef(2)]), (t1, &[])]
            ).unwrap();
            (vec![stack.consumed(4),stack.consumed(3),stack.untouched(2)], vec![stack.borrowed(1, vec![ValueRef(0)]), stack.owned(0)])
        });
    }

    //Make some nested Bigger tests touching on multiple aspects
    #[test]
    fn different_deps() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let t2 = stack.introduce_new();
            let mut br0 = stack.start_branching();
                let bl1 = stack.start_block();
                    stack.provide(t0.clone()).unwrap();
                stack.end_block(bl1, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.next_branch(&mut br0, ExprResult::Return(&[ValueRef(0)])).unwrap();
                let mut br1 = stack.start_branching();
                    stack.provide(t1.clone()).unwrap();
                    stack.provide(t0.clone()).unwrap();
                stack.next_branch(&mut br1, ExprResult::Return(&[ValueRef(0),ValueRef(1)])).unwrap();
                    stack.provide(t0.clone()).unwrap();
                    stack.provide(t1.clone()).unwrap();
                    stack.provide(t2.clone()).unwrap();
                    stack.drop(ValueRef(0)).unwrap();
            stack.end_branching(br1, ExprResult::Return(&[ValueRef(2),ValueRef(1)])).unwrap();
                stack.drop(ValueRef(0)).unwrap();
            stack.next_branch(&mut br0,ExprResult::Return(&[ValueRef(1)])).unwrap();
                stack.provide(t0.clone()).unwrap();
            stack.end_branching(br0,ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![], vec![stack.owned(2)])
        });
    }

    #[test]
    fn different_deps2() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let t2 = stack.introduce_new();
            stack.provide_new();
            let mut br0 = stack.start_branching();
                let bl1 = stack.start_block();
                    stack.provide(t0.clone()).unwrap();
                    stack.drop(ValueRef(1)).unwrap();
                stack.end_block(bl1, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.next_branch(&mut br0, ExprResult::Return(&[ValueRef(0)])).unwrap();
                let mut br1 = stack.start_branching();
                    stack.provide(t1.clone()).unwrap();
                    stack.provide(t0.clone()).unwrap();
                    stack.drop(ValueRef(2)).unwrap();
                stack.next_branch(&mut br1, ExprResult::Return(&[ValueRef(0),ValueRef(1)])).unwrap();
                    stack.provide(t0.clone()).unwrap();
                    stack.drop(ValueRef(1)).unwrap();
                    stack.provide(t1.clone()).unwrap();
                    stack.provide(t2.clone()).unwrap();
                    stack.drop(ValueRef(0)).unwrap();
            stack.end_branching(br1, ExprResult::Return(&[ValueRef(2),ValueRef(1)])).unwrap();
                stack.drop(ValueRef(0)).unwrap();
            stack.next_branch(&mut br0,ExprResult::Return(&[ValueRef(1)])).unwrap();
                stack.drop(ValueRef(0)).unwrap();
                stack.provide(t0.clone()).unwrap();
            stack.end_branching(br0,ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.consumed(0)], vec![stack.owned(3)])
        });
    }

    #[test]
    fn different_deps3() {
        test(|stack|{
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let t2 = stack.introduce_new();
            stack.provide(t2.clone()).unwrap();
            let mut br0 = stack.start_branching();
            let bl1 = stack.start_block();
            stack.transform(ValueRef(0), t0.clone(), FetchMode::Borrow).unwrap();
            stack.end_block(bl1, ExprResult::Return(&[ValueRef(0)])).unwrap();
            stack.next_branch(&mut br0, ExprResult::Return(&[ValueRef(0)])).unwrap();
            let mut br1 = stack.start_branching();
            stack.provide(t1.clone()).unwrap();
            stack.transform(ValueRef(1), t0.clone(), FetchMode::Borrow).unwrap();
            stack.next_branch(&mut br1, ExprResult::Return(&[ValueRef(0),ValueRef(1)])).unwrap();
            stack.transform(ValueRef(0), t0.clone(), FetchMode::Borrow).unwrap();
            stack.provide(t1.clone()).unwrap();
            stack.provide(t2.clone()).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            stack.end_branching(br1, ExprResult::Return(&[ValueRef(2),ValueRef(1)])).unwrap();
            stack.drop(ValueRef(0)).unwrap();
            stack.next_branch(&mut br0,ExprResult::Return(&[ValueRef(1)])).unwrap();
            stack.transform(ValueRef(0), t0.clone(), FetchMode::Borrow).unwrap();
            stack.end_branching(br0,ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.untouched(0)], vec![stack.borrowed(2, vec![ValueRef(0)])])
        });
    }


    #[test]
    fn diamond_borrow_test() {
        test(|stack|{
            stack.provide_new();
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let block = stack.start_block();
                stack.unpack(ValueRef(0), &[t1.clone(), t1.clone()], FetchMode::Borrow).unwrap();
                stack.pack(&[ValueRef(0), ValueRef(1)], t0.clone(), FetchMode::Borrow).unwrap();
                stack.steal(ValueRef(0),ValueRef(1)).unwrap();
                stack.steal(ValueRef(0),ValueRef(2)).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.untouched(2)], vec![stack.borrowed(1, vec![ValueRef(0)])])
        });
    }

    #[test]
    fn diamond_borrow_test2() {
        test(|stack|{
            stack.provide_new();
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let block = stack.start_block();
                stack.unpack(ValueRef(0), &[t1.clone(), t1.clone()], FetchMode::Borrow).unwrap();
                stack.pack(&[ValueRef(1), ValueRef(0)], t0.clone(), FetchMode::Borrow).unwrap();
                stack.steal(ValueRef(0),ValueRef(2)).unwrap();
                stack.steal(ValueRef(0),ValueRef(1)).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.untouched(2)], vec![stack.borrowed(1, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn diamond_borrow_test_fail() {
        test(|stack|{
            stack.provide_new();
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let block = stack.start_block();
                stack.unpack(ValueRef(0), &[t1.clone(), t1.clone()], FetchMode::Borrow).unwrap();
                stack.pack(&[ValueRef(0), ValueRef(0)], t0.clone(), FetchMode::Borrow).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.untouched(2)], vec![stack.borrowed(1, vec![ValueRef(0)])])
        });
    }

    #[test]
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn diamond_borrow_test_fail2() {
        test(|stack|{
            stack.provide_new();
            let t0 = stack.introduce_new();
            let t1 = stack.introduce_new();
            let block = stack.start_block();
            stack.unpack(ValueRef(0), &[t1.clone(), t1.clone()], FetchMode::Borrow).unwrap();
            stack.pack(&[ValueRef(1), ValueRef(1)], t0.clone(), FetchMode::Borrow).unwrap();
            stack.end_block(block, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.untouched(2)], vec![stack.borrowed(1, vec![ValueRef(0)])])
        });
    }

    #[test]
    fn cross_borrow_test() {
        test(|stack|{
            let t0 = stack.introduce_new();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t0).unwrap();
            let t1 = stack.introduce_new();
            let mut branch = stack.start_branching();
                stack.pack(&[ValueRef(0), ValueRef(1)], t1.clone(), FetchMode::Borrow).unwrap();
            stack.next_branch(&mut branch, ExprResult::Return(&[ValueRef(0)])).unwrap();
                stack.pack(&[ValueRef(1), ValueRef(0)], t1.clone(), FetchMode::Borrow).unwrap();
            stack.end_branching(branch, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.untouched(1),stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(0),ValueRef(1)])])
        });
    }

    #[test]
    fn cross_borrow_test2() {
        test(|stack|{
            let t0 = stack.introduce_new();
            stack.provide(t0.clone()).unwrap();
            stack.provide(t0).unwrap();
            let t1 = stack.introduce_new();
            let mut branch = stack.start_branching();
                stack.pack(&[ValueRef(0), ValueRef(1)], t1.clone(), FetchMode::Borrow).unwrap();
            stack.next_branch(&mut branch, ExprResult::Return(&[ValueRef(0)])).unwrap();
                stack.pack(&[ValueRef(1), ValueRef(0)], t1.clone(), FetchMode::Borrow).unwrap();
            stack.end_branching(branch, ExprResult::Return(&[ValueRef(0)])).unwrap();
            (vec![stack.untouched(1),stack.untouched(1)], vec![stack.borrowed(0, vec![ValueRef(1),ValueRef(0)])])
        });
    }
}