use test::inputgen::import::type_body_import_builder::*;
use test::inputgen::import::function_import_builder::*;
use test::inputgen::import::type_import_builder::*;
use compiler::common::types::*;
use compiler::common::parsing::*;
use compiler::common::macros::view::*;
use compiler::typ::view::*;
use compiler::function::view::*;
use compiler::module::view::*;

use compiler::errors::general::*;


#[cfg(test)]
mod type_tests {

    use super::*;
    use test::inputgen::type_builder::*;

    const PSEUDO_HASH_BYTES:[u8;HASH_SIZE] = [127;HASH_SIZE];
    const PSEUDO_HASH_BYTES2:[u8;HASH_SIZE] = [97;HASH_SIZE];
    const PSEUDO_HASH_BYTES3:[u8;HASH_SIZE] = [225;HASH_SIZE];


    //Helper for methods where the header is not tested
    fn header_without_type_imports() -> TypeBuilder{
        let mut builder = TypeBuilder::new();
        builder.add_fix_header_data(HeaderData{
            module_hash:  Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap(),
            type_index: MemberIndex(9),
            max_privileges: Privileges::no_privileges().add_copy_privilege(),
            visibility: Visibility::Public, //todo: is checked???))
            kind: TypeKind::Cell,
            optimisation_declaration: OptimizationDeclaration::Empty,
        });

        builder.add_init_code( Hash::from_bytes(&PSEUDO_HASH_BYTES3).unwrap());
        builder
    }

    //Helper for methods where the header and imports are not tested
    fn header_with_imports() -> TypeBuilder {
        let mut builder = header_without_type_imports();

        builder.add_init_code(Hash::from_bytes(&PSEUDO_HASH_BYTES3).unwrap());

        builder.add_module_import( Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap());
        builder.add_module_import( Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap());

        let mut import = TypeImportBuilder::new();
        import.add_fix_header_data(TypeImportData{
            module_id: ModuleId(1),
            type_index: MemberIndex(5),
            optimisation_declaration: OptimizationDeclaration::Wrapper,
            kind_declaration: TypeKind::Normal,
            privileges_declaration: Privileges::no_privileges().add_native_privilege(),
        });

        import.add_type_apply(TypeId(0));
        import.add_type_apply(TypeId(3));

        builder.add_type_import(import);

        import = TypeImportBuilder::new();
        import.add_fix_header_data(TypeImportData{
            module_id: ModuleId(2),
            type_index: MemberIndex(1),
            optimisation_declaration: OptimizationDeclaration::Normal,
            kind_declaration: TypeKind::View,
            privileges_declaration: Privileges::no_privileges()
                .add_create_privilege()
                .add_access_privilege()
                .add_discard_privilege(),
        });

        import.add_type_apply(TypeId(7));
        builder.add_type_import(import);


        let mut case1 = ConstructorCaseBuilder::new();
        case1.add_field(Field(Control::Ref,TypeId(1)));
        case1.add_field(Field(Control::UnusedOwned,TypeId(3)));

        builder.add_case( case1);
        builder.finish_header(true);
        builder
    }

    fn header_and_body_with_imports() -> TypeBuilder  {
        let mut builder = header_with_imports();


        builder.add_body_module_import(Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap());
        builder.add_body_module_import(Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap());

        let mut import = TypeImportBuilder::new();
        import.add_fix_header_data(TypeImportData{
            module_id: ModuleId(3),
            type_index: MemberIndex(4),
            optimisation_declaration: OptimizationDeclaration::Empty,
            kind_declaration: TypeKind::View,
            privileges_declaration: Privileges::no_privileges()
                .add_copy_privilege()
                .add_access_privilege()
                .add_persist_privilege()
                .add_discard_privilege(),
        });

        import.add_type_apply(TypeId(0));
        import.add_type_apply(TypeId(3));

        builder.add_body_type_import(import);

        import = TypeImportBuilder::new();
        import.add_fix_header_data(TypeImportData{
            module_id: ModuleId(1),
            type_index: MemberIndex(9),
            optimisation_declaration: OptimizationDeclaration::Wrapper,
            kind_declaration: TypeKind::Normal,
            privileges_declaration: Privileges::no_privileges()
                .add_copy_privilege()
                .add_write_privilege()
                .add_persist_privilege()
                .add_discard_privilege(),
        });

        import.add_type_apply(TypeId(7));
        builder.add_body_type_import(import);

        builder
    }


    fn header_and_body_with_imports_inc_functions() -> TypeBuilder  {
        let mut builder = header_and_body_with_imports();

        let mut import = FunctionImportBuilder::new();
        import.add_fix_header_data(FunctionImportData{
            module_id: ModuleId(5),
            fun_index: MemberIndex(3),
            optimisation_declaration: OptimizationDeclaration::Wrapper,
            return_type: TypeId(3),
            return_control: Control::UnusedOwned,
            code_hash: Hash::from_bytes(&PSEUDO_HASH_BYTES3).unwrap(),
        });

        import.add_type_apply(TypeId(1));
        import.add_type_apply(TypeId(4));

        import.add_value_apply(Field(Control::Owned, TypeId(7)));
        import.add_value_apply(Field(Control::UnusedOwned, TypeId(2)));

        builder.add_function_import(import);

        builder
    }

    fn header_and_body_with_imports_inc_functions_and_constructors() -> TypeBuilder  {
        let mut builder = header_and_body_with_imports_inc_functions();

        let mut import = ConstructorsImportBuilder::new();
        import.corresponding_type(TypeId(9));
        let mut case1 = ConstructorCaseBuilder::new();
        case1.add_field(Field(Control::Owned,TypeId(2)));
        case1.add_field(Field(Control::Ref,TypeId(4)));

        import.add_constructor_case(case1);
        let mut case2 = ConstructorCaseBuilder::new();
        case2.add_field(Field(Control::UnusedBorrowed,TypeId(0)));

        import.add_constructor_case(case2);
        import.finish();

        builder.add_constructors_import(import);

        builder
    }

    #[test] fn without_body_and_imports(){
        let mut builder = TypeBuilder::new();
        builder.add_fix_header_data(HeaderData{
            module_hash:  Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap(),
            type_index: MemberIndex(1),
            visibility: Visibility::Public,
            kind: TypeKind::Normal,
            optimisation_declaration: OptimizationDeclaration::Normal,
            max_privileges: Privileges::no_privileges()
                .add_copy_privilege()
                .add_write_privilege()
                .add_persist_privilege()
                .add_load_privilege(),
        });

        builder.add_generic(Bound::Phantom);
        builder.add_generic(Bound::Dynamic);

        let mut case1 = ConstructorCaseBuilder::new();
        case1.add_field(Field(Control::Owned, TypeId(0)));
        case1.add_field(Field(Control::UnusedOwned, TypeId(1)));

        builder.add_case(case1);

        let mut case2 = ConstructorCaseBuilder::new();
        case2.add_field(Field(Control::Owned, TypeId(2)));

        builder.add_case(case2);
        builder.finish_header(false);

        let data = builder.extract();

        let view = TypeView::parse(&data,true).unwrap();

        fn my_catch(view:&TypeView) -> Result<(),CompilationError>{
            assert_eq!(view.header.module_hash()?, Hash::from_bytes(&PSEUDO_HASH_BYTES)?);
            assert_eq!(view.header.type_index()?,MemberIndex(1));
            assert_eq!(view.header.max_supported_privileges()?,Privileges::no_privileges()
                .add_copy_privilege()
                .add_write_privilege()
                .add_persist_privilege()
                .add_load_privilege(),
            );
            assert_eq!(view.header.visibility()?,Visibility::Public);
            assert_eq!(view.header.kind()?,TypeKind::Normal);
            assert_eq!(view.header.declared_optimisation()?,OptimizationDeclaration::Normal);

            assert_eq!(view.header.generics.len(),2);
            assert_eq!(view.header.generics.get(0)?,Bound::Phantom);
            assert_eq!(view.header.generics.get(1)?,Bound::Dynamic);

            assert_eq!(view.header.constructors.len(),2);
            assert_eq!(view.header.constructors.get(0)?.params.len(),2);
            assert_eq!(view.header.constructors.get(0)?.params.get(0)?,Field(Control::Owned, TypeId(0)));
            assert_eq!(view.header.constructors.get(0)?.params.get(1)?,Field(Control::UnusedOwned, TypeId(1)));
            assert_eq!(view.header.constructors.get(1)?.params.len(),1);
            assert_eq!(view.header.constructors.get(1)?.params.get(0)?,Field(Control::Owned, TypeId(2)));
            Ok(())
        }

        //let res:Result<(),CompilationError> = do catch { }; //works but no syntax for inteliJ
        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok());
    }

    #[test] fn imports_without_body(){
        let mut builder = header_without_type_imports();


        builder.add_module_import( Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap());
        builder.add_module_import( Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap());

        let mut import = TypeImportBuilder::new();
        import.add_fix_header_data(TypeImportData{
            module_id: ModuleId(0),
            type_index: MemberIndex(2),
            optimisation_declaration: OptimizationDeclaration::Empty,
            kind_declaration: TypeKind::Cell,
            privileges_declaration: Privileges::no_privileges()
                .add_copy_privilege()
                .add_write_privilege()
                .add_unwrap_privilege()
                .add_wrap_privilege(),
        });


        import.add_type_apply(TypeId(0));
        import.add_type_apply(TypeId(3));
        import.add_type_apply(TypeId(7));

        builder.add_type_import(import);
        builder.add_case( ConstructorCaseBuilder::new());
        builder.finish_header(false);

        let data = builder.extract();

        let view = TypeView::parse(&data,false).unwrap();

        fn my_catch(view:&TypeView) -> Result<(),CompilationError>{
            assert_eq!(view.header.code_hash()?,Hash::from_bytes(&PSEUDO_HASH_BYTES3)?);

            assert_eq!(view.header.module_imports.len(),2);
            assert_eq!(view.header.module_imports.get(0)?,Hash::from_bytes(&PSEUDO_HASH_BYTES)?);
            assert_eq!(view.header.module_imports.get(1)?,Hash::from_bytes(&PSEUDO_HASH_BYTES2)?);

            assert_eq!(view.header.type_imports.len(),1);
            let t1 = view.header.type_imports.get(0)?;

            assert_eq!(t1.declaring_module()?,ModuleId(0));
            assert_eq!(t1.module_version()?,Version(0));
            assert_eq!(t1.identifying_index()?,MemberIndex(2));

            assert_eq!(t1.declared_optimisation()?,OptimizationDeclaration::Empty);
            assert_eq!(t1.declared_privileges()?,Privileges::no_privileges()
                .add_copy_privilege()
                .add_write_privilege()
                .add_unwrap_privilege()
                .add_wrap_privilege(),
            );
            assert_eq!(t1.declared_kind()?, TypeKind::Cell);

            assert_eq!(t1.params.len(),3);
            assert_eq!(t1.params.get(0)?,TypeId(0));
            assert_eq!(t1.params.get(1)?,TypeId(3));
            assert_eq!(t1.params.get(2)?,TypeId(7));

            assert_eq!(view.header.generics.len(),0);
            assert_eq!(view.header.constructors.len(),1);
            assert_eq!(view.header.constructors.get(0)?.params.len(),0);

            Ok(())
        }

        //let res:Result<(),CompilationError> = do catch { }; //works but no syntax for inteliJ
        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok());

    }

    #[test] fn body_module_imports(){
        let mut builder = header_with_imports();

        builder.add_body_module_import(Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap());
        builder.add_body_module_import(Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap());

        builder.finish_body();

        let data = builder.extract();

        let view = TypeView::parse(&data,true).unwrap();

        fn my_catch(view:&TypeView) -> Result<(),CompilationError>{

            assert_eq!(view.header.module_imports.len(),2);
            assert_eq!(view.header.module_imports.get(0)?,Hash::from_bytes(&PSEUDO_HASH_BYTES)?);
            assert_eq!(view.header.module_imports.get(1)?,Hash::from_bytes(&PSEUDO_HASH_BYTES2)?);

            assert_eq!(view.header.code_hash()?,Hash::from_bytes(&PSEUDO_HASH_BYTES3)?);

            let body = match view.body {
                Some(ref body) => body,
                None => {
                    assert!(false);
                    panic!()
                }
            };

            assert_eq!(body.module_imports.len(),2);
            assert_eq!(body.module_imports.get(0)?,Hash::from_bytes(&PSEUDO_HASH_BYTES)?);
            assert_eq!(body.module_imports.get(1)?,Hash::from_bytes(&PSEUDO_HASH_BYTES2)?);

            Ok(())

        }

        //let res:Result<(),CompilationError> = do catch { }; //works but no syntax for inteliJ
        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok(),res);

    }

    #[test] fn body_module_type_imports(){
        let mut builder = header_with_imports();

        builder.add_body_module_import(Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap());
        builder.add_body_module_import(Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap());

        let mut import = TypeImportBuilder::new();
        import.add_fix_header_data(TypeImportData{
            module_id: ModuleId(3),
            type_index: MemberIndex(4),
            optimisation_declaration: OptimizationDeclaration::Empty,
            kind_declaration: TypeKind::View,
            privileges_declaration: Privileges::no_privileges()
                .add_write_privilege()
                .add_copy_privilege()
                .add_unwrap_privilege()
                .add_access_privilege(),
        });

        import.add_type_apply(TypeId(0));
        import.add_type_apply(TypeId(3));

        builder.add_body_type_import(import);

        import = TypeImportBuilder::new();
        import.add_fix_header_data(TypeImportData{
            module_id: ModuleId(1),
            type_index: MemberIndex(9),
            optimisation_declaration: OptimizationDeclaration::Wrapper,
            kind_declaration: TypeKind::Normal,
            privileges_declaration: Privileges::no_privileges()
                .add_write_privilege()
                .add_copy_privilege()
                .add_unwrap_privilege()
                .add_load_privilege(),
        });

        import.add_type_apply(TypeId(7));
        builder.add_body_type_import(import);

        builder.finish_body();

        let data = builder.extract();

        let view = TypeView::parse(&data,true).unwrap();

        fn my_catch(view:&TypeView) -> Result<(),CompilationError>{

            assert_eq!(view.header.module_imports.len(),2);
            assert_eq!(view.header.module_imports.get(0)?,Hash::from_bytes(&PSEUDO_HASH_BYTES)?);
            assert_eq!(view.header.module_imports.get(1)?,Hash::from_bytes(&PSEUDO_HASH_BYTES2)?);

            assert_eq!(view.header.code_hash()?,Hash::from_bytes(&PSEUDO_HASH_BYTES3)?);

            let body = match view.body {
                Some(ref body) => body,
                None => {
                    assert!(false);
                    panic!()
                }
            };

            assert_eq!(body.module_imports.len(),2);
            assert_eq!(body.module_imports.get(0)?,Hash::from_bytes(&PSEUDO_HASH_BYTES)?);
            assert_eq!(body.module_imports.get(1)?,Hash::from_bytes(&PSEUDO_HASH_BYTES2)?);

            assert_eq!(body.type_imports.len(),2);
            let tb1 = body.type_imports.get(0)?;

            assert_eq!(tb1.declaring_module()?,ModuleId(3));
            assert_eq!(tb1.module_version()?,Version(0));
            assert_eq!(tb1.identifying_index()?,MemberIndex(4));
            assert_eq!(tb1.declared_optimisation()?,OptimizationDeclaration::Empty);
            assert_eq!(tb1.declared_privileges()?,Privileges::no_privileges()
                .add_write_privilege()
                .add_copy_privilege()
                .add_unwrap_privilege()
                .add_access_privilege(),
            );
            assert_eq!(tb1.declared_kind()?, TypeKind::View);

            assert_eq!(tb1.params.len(),2);
            assert_eq!(tb1.params.get(0)?,TypeId(0));
            assert_eq!(tb1.params.get(1)?,TypeId(3));

            let tb2 = body.type_imports.get(1)?;

            assert_eq!(tb2.declaring_module()?,ModuleId(1));
            assert_eq!(tb2.module_version()?,Version(0));
            assert_eq!(tb2.identifying_index()?,MemberIndex(9));
            assert_eq!(tb2.declared_optimisation()?,OptimizationDeclaration::Wrapper);
            assert_eq!(tb2.declared_privileges()?,Privileges::no_privileges()
                .add_write_privilege()
                .add_copy_privilege()
                .add_unwrap_privilege()
                .add_load_privilege(),
            );

            assert_eq!(tb2.declared_kind()?, TypeKind::Normal);

            assert_eq!(tb2.params.len(),1);
            assert_eq!(tb2.params.get(0)?,TypeId(7));

            Ok(())

        }

        //let res:Result<(),CompilationError> = do catch { }; //works but no syntax for inteliJ
        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok());

    }



    #[test] fn body_functions_imports(){
        let mut builder = header_and_body_with_imports();

        let mut import = FunctionImportBuilder::new();
        import.add_fix_header_data(FunctionImportData{
            module_id: ModuleId(5),
            fun_index: MemberIndex(3),
            optimisation_declaration: OptimizationDeclaration::Wrapper,
            return_type: TypeId(3),
            return_control: Control::UnusedOwned,
            code_hash: Hash::from_bytes(&PSEUDO_HASH_BYTES3).unwrap(),
        });

        import.add_type_apply(TypeId(1));
        import.add_type_apply(TypeId(4));

        import.add_value_apply(Field(Control::Owned, TypeId(7)));
        import.add_value_apply(Field(Control::UnusedOwned, TypeId(2)));

        builder.add_function_import(import);

        builder.finish_body();

        let data = builder.extract();

        let view = TypeView::parse(&data,true).unwrap();

        fn my_catch(view:&TypeView) -> Result<(),CompilationError>{

            let body = match view.body {
                Some(ref body) => body,
                None => {
                    assert!(false);
                    panic!()
                }
            };

            assert_eq!(body.function_imports.len(),1);
            let fb1 = body.function_imports.get(0)?;

            assert_eq!(fb1.declaring_module()?,ModuleId(5));
            assert_eq!(fb1.version()?,Version(0));
            assert_eq!(fb1.identifying_index()?,MemberIndex(3));
            assert_eq!(fb1.declared_optimisation()?,OptimizationDeclaration::Wrapper);
            assert_eq!(fb1.return_type()?,TypeId(3));
            assert_eq!(fb1.return_control()?,Control::UnusedOwned);
            assert_eq!(fb1.code_hash()?,Hash::from_bytes(&PSEUDO_HASH_BYTES3)?);

            assert_eq!(fb1.generic_params.len(),2);
            assert_eq!(fb1.generic_params.get(0)?,TypeId(1));
            assert_eq!(fb1.generic_params.get(1)?,TypeId(4));

            assert_eq!(fb1.params.len(),2);
            assert_eq!(fb1.params.get(0)?,Field(Control::Owned, TypeId(7)));
            assert_eq!(fb1.params.get(1)?,Field(Control::UnusedOwned, TypeId(2)));

            Ok(())

        }

        //let res:Result<(),CompilationError> = do catch { }; //works but no syntax for inteliJ
        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok());

    }

    #[test] fn body_constructor_imports(){
        let mut builder = header_and_body_with_imports_inc_functions();

        let mut import = ConstructorsImportBuilder::new();
        import.corresponding_type(TypeId(9));
        let mut case1 = ConstructorCaseBuilder::new();
        case1.add_field(Field(Control::Owned,TypeId(2)));
        case1.add_field(Field(Control::Ref,TypeId(4)));

        import.add_constructor_case(case1);
        let mut case2 = ConstructorCaseBuilder::new();
        case2.add_field(Field(Control::UnusedBorrowed,TypeId(0)));

        import.add_constructor_case(case2);
        import.finish();

        builder.add_constructors_import(import);


        builder.finish_body();

        let data = builder.extract();

        let view = TypeView::parse(&data,true).unwrap();

        fn my_catch(view:&TypeView) -> Result<(),CompilationError>{

            let body = match view.body {
                Some(ref body) => body,
                None => {
                    assert!(false);
                    panic!()
                }
            };

            assert_eq!(body.constructor_imports.len(),1);
            let c1 = body.constructor_imports.get(0)?;

            assert_eq!(c1.coresponding_type()?,TypeId(9));
            assert_eq!(c1.constructors.len(),2);
            let ca1 = c1.constructors.get(0)?;
            assert_eq!(ca1.params.len(),2);
            assert_eq!(ca1.params.get(0)?,Field(Control::Owned,TypeId(2)));
            assert_eq!(ca1.params.get(1)?,Field(Control::Ref,TypeId(4)));

            let ca2 = c1.constructors.get(1)?;
            assert_eq!(ca2.params.len(),1);
            assert_eq!(ca2.params.get(0)?,Field(Control::UnusedBorrowed,TypeId(0)));
            Ok(())

        }

        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok(), res);

    }

    #[test] fn body_init_imports(){
        let mut builder = header_and_body_with_imports_inc_functions_and_constructors();

        let mut import = InitImportBuilder::new();
        import.corresponding_type(TypeId(3));
        import.add_init_code( Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap());
        import.return_type(TypeId(2));
        builder.add_init_import(import);

        import = InitImportBuilder::new();
        import.corresponding_type(TypeId(8));
        import.return_type(TypeId(6));
        import.add_init_code( Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap());
        builder.add_init_import(import);

        builder.finish_body();

        let data = builder.extract();

        let view = TypeView::parse(&data,true).unwrap();

        fn my_catch(view:&TypeView) -> Result<(),CompilationError>{

            let body = match view.body {
                Some(ref body) => body,
                None => {
                    assert!(false);
                    panic!()
                }
            };

            assert_eq!(body.init_imports.len(),2);
            assert_eq!(body.init_imports.get(0)?.coresponding_type()?,TypeId(3));
            assert_eq!(body.init_imports.get(0)?.code_hash()?,Hash::from_bytes(&PSEUDO_HASH_BYTES2)?);
            assert_eq!(body.init_imports.get(0)?.init_return_type()?,TypeId(2));

            assert_eq!(body.init_imports.get(1)?.coresponding_type()?,TypeId(8));
            assert_eq!(body.init_imports.get(1)?.code_hash()?,Hash::from_bytes(&PSEUDO_HASH_BYTES)?);
            assert_eq!(body.init_imports.get(1)?.init_return_type()?,TypeId(6));

            Ok(())

        }

        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok(), res);

    }


}

#[cfg(test)]
mod function_tests {

    use super::*;
    use test::inputgen::function_builder::*;
    use test::inputgen::type_builder as type_builder;

    const PSEUDO_HASH_BYTES:[u8;HASH_SIZE] = [127;HASH_SIZE];
    const PSEUDO_HASH_BYTES2:[u8;HASH_SIZE] = [97;HASH_SIZE];
    const PSEUDO_HASH_BYTES3:[u8;HASH_SIZE] = [225;HASH_SIZE];


    //Helper for methods where the header is not tested
    fn header_without_type_imports() -> FunctionBuilder{
        let mut builder = FunctionBuilder::new();
        builder.add_fix_header_data(HeaderData{
            module_hash:  Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap(),
            visibility: Visibility::Public, //todo: is checked???))
            optimisation_declaration: OptimizationDeclaration::Empty,
            return_type: TypeId(4),
            return_control: Control::UnusedOwned,
            fun_index: MemberIndex(9),
            execution_mode: ExecutionMode::Dependent,
            code_hash: Hash::from_bytes(&PSEUDO_HASH_BYTES3).unwrap(),
        });


        builder
    }

    //Helper for methods where the header and imports are not tested
    fn header_with_imports() -> FunctionBuilder {
        let mut builder = header_without_type_imports();

        builder.add_module_import( Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap());
        builder.add_module_import( Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap());

        let mut import = TypeImportBuilder::new();
        import.add_fix_header_data(TypeImportData{
            module_id: ModuleId(1),
            type_index: MemberIndex(5),
            optimisation_declaration: OptimizationDeclaration::Wrapper,
            kind_declaration: TypeKind::Normal,
            privileges_declaration: Privileges::no_privileges().add_native_privilege(),
        });

        import.add_type_apply(TypeId(0));
        import.add_type_apply(TypeId(3));

        builder.add_type_import(import);

        import = TypeImportBuilder::new();
        import.add_fix_header_data(TypeImportData{
            module_id: ModuleId(2),
            type_index: MemberIndex(1),
            optimisation_declaration: OptimizationDeclaration::Normal,
            kind_declaration: TypeKind::View,
            privileges_declaration: Privileges::no_privileges()
                .add_create_privilege()
                .add_access_privilege()
                .add_discard_privilege(),
        });

        import.add_type_apply(TypeId(7));
        builder.add_type_import(import);

        builder.finish_header(true);
        builder
    }

    fn header_and_body_with_imports() -> FunctionBuilder  {
        let mut builder = header_with_imports();


        builder.add_body_module_import(Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap());
        builder.add_body_module_import(Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap());

        let mut import = TypeImportBuilder::new();
        import.add_fix_header_data(TypeImportData{
            module_id: ModuleId(3),
            type_index: MemberIndex(4),
            optimisation_declaration: OptimizationDeclaration::Empty,
            kind_declaration: TypeKind::View,
            privileges_declaration: Privileges::no_privileges()
                .add_copy_privilege()
                .add_access_privilege()
                .add_persist_privilege()
                .add_discard_privilege(),
        });

        import.add_type_apply(TypeId(0));
        import.add_type_apply(TypeId(3));

        builder.add_body_type_import(import);

        import = TypeImportBuilder::new();
        import.add_fix_header_data(TypeImportData{
            module_id: ModuleId(1),
            type_index: MemberIndex(9),
            optimisation_declaration: OptimizationDeclaration::Wrapper,
            kind_declaration: TypeKind::Normal,
            privileges_declaration: Privileges::no_privileges()
                .add_copy_privilege()
                .add_write_privilege()
                .add_persist_privilege()
                .add_discard_privilege(),
        });

        import.add_type_apply(TypeId(7));
        builder.add_body_type_import(import);

        builder
    }


    fn header_and_body_with_imports_inc_functions() -> FunctionBuilder  {
        let mut builder = header_and_body_with_imports();

        let mut import = FunctionImportBuilder::new();
        import.add_fix_header_data(FunctionImportData{
            module_id: ModuleId(5),
            fun_index: MemberIndex(3),
            optimisation_declaration: OptimizationDeclaration::Wrapper,
            return_type: TypeId(3),
            return_control: Control::UnusedOwned,
            code_hash: Hash::from_bytes(&PSEUDO_HASH_BYTES3).unwrap(),
        });

        import.add_type_apply(TypeId(1));
        import.add_type_apply(TypeId(4));

        import.add_value_apply(Field(Control::Owned, TypeId(7)));
        import.add_value_apply(Field(Control::UnusedOwned, TypeId(2)));

        builder.add_function_import(import);

        builder
    }

    fn header_and_body_with_imports_inc_functions_and_constructors() -> FunctionBuilder  {
        let mut builder = header_and_body_with_imports_inc_functions();

        let mut import = ConstructorsImportBuilder::new();
        import.corresponding_type(TypeId(9));
        let mut case1 = type_builder::ConstructorCaseBuilder::new();
        case1.add_field(Field(Control::Owned,TypeId(2)));
        case1.add_field(Field(Control::Ref,TypeId(4)));

        import.add_constructor_case(case1);
        let mut case2 = type_builder::ConstructorCaseBuilder::new();
        case2.add_field(Field(Control::UnusedBorrowed,TypeId(0)));

        import.add_constructor_case(case2);
        import.finish();

        builder.add_constructors_import(import);

        builder
    }

    #[test] fn without_body_and_imports(){
        let mut builder = FunctionBuilder::new();
        builder.add_fix_header_data(HeaderData{
            module_hash:  Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap(),
            visibility: Visibility::Public,
            optimisation_declaration: OptimizationDeclaration::Normal,
            return_type: TypeId(3),
            return_control: Control::Owned,
            fun_index: MemberIndex(1),
            execution_mode: ExecutionMode::Pure,
            code_hash:  Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap()
        });

        builder.add_generic(Privileges::no_privileges().add_discard_privilege());
        builder.add_generic(Privileges::no_privileges().add_load_privilege().add_write_privilege());

        builder.finish_header(false);

        let data = builder.extract();

        let view = FunctionView::parse(&data,false).unwrap();

        fn my_catch(view:&FunctionView) -> Result<(),CompilationError>{

            assert_eq!(view.header.module_hash()?, Hash::from_bytes(&PSEUDO_HASH_BYTES)?);
            assert_eq!(view.header.visibility()?,Visibility::Public);
            assert_eq!(view.header.return_type()?, TypeId(3));
            assert_eq!(view.header.return_control()?,Control::Owned);
            assert_eq!(view.header.fun_index()?,MemberIndex(1));
            assert_eq!(view.header.code_hash()?,Hash::from_bytes(&PSEUDO_HASH_BYTES2)?);
            assert_eq!(view.header.execution_mode()?,ExecutionMode::Pure);
            assert_eq!(view.header.declared_optimisation()?,OptimizationDeclaration::Normal);

            assert_eq!(view.header.generics.len(),2);
            assert_eq!(view.header.generics.get(0)?,Privileges::no_privileges().add_discard_privilege());
            assert_eq!(view.header.generics.get(1)?,Privileges::no_privileges().add_load_privilege().add_write_privilege());

            Ok(())
        }

        //let res:Result<(),CompilationError> = do catch { }; //works but no syntax for inteliJ
        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok());
    }

    #[test] fn imports_without_body(){
        let mut builder = header_without_type_imports();

        builder.add_module_import( Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap());
        builder.add_module_import( Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap());

        let mut import = TypeImportBuilder::new();
        import.add_fix_header_data(TypeImportData{
            module_id: ModuleId(0),
            type_index: MemberIndex(2),
            optimisation_declaration: OptimizationDeclaration::Empty,
            kind_declaration: TypeKind::Cell,
            privileges_declaration: Privileges::no_privileges()
                .add_copy_privilege()
                .add_write_privilege()
                .add_unwrap_privilege()
                .add_wrap_privilege(),
        });

        import.add_type_apply(TypeId(0));
        import.add_type_apply(TypeId(3));
        import.add_type_apply(TypeId(7));

        builder.add_type_import(import);
        builder.finish_header(false);

        let data = builder.extract();

        let view = FunctionView::parse(&data,false).unwrap();

        fn my_catch(view:&FunctionView) -> Result<(),CompilationError>{

            assert_eq!(view.header.module_imports.len(),2);
            assert_eq!(view.header.module_imports.get(0)?,Hash::from_bytes(&PSEUDO_HASH_BYTES)?);
            assert_eq!(view.header.module_imports.get(1)?,Hash::from_bytes(&PSEUDO_HASH_BYTES2)?);

            assert_eq!(view.header.type_imports.len(),1);
            let t1 = view.header.type_imports.get(0)?;

            assert_eq!(t1.declaring_module()?,ModuleId(0));
            assert_eq!(t1.module_version()?,Version(0));
            assert_eq!(t1.identifying_index()?,MemberIndex(2));

            assert_eq!(t1.declared_optimisation()?,OptimizationDeclaration::Empty);
            assert_eq!(t1.declared_privileges()?,Privileges::no_privileges()
                .add_copy_privilege()
                .add_write_privilege()
                .add_unwrap_privilege()
                .add_wrap_privilege(),
            );
            assert_eq!(t1.declared_kind()?, TypeKind::Cell);

            assert_eq!(t1.params.len(),3);
            assert_eq!(t1.params.get(0)?,TypeId(0));
            assert_eq!(t1.params.get(1)?,TypeId(3));
            assert_eq!(t1.params.get(2)?,TypeId(7));

            Ok(())
        }

        //let res:Result<(),CompilationError> = do catch { }; //works but no syntax for inteliJ
        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok());

    }

    #[test] fn body_module_imports(){
        let mut builder = header_with_imports();

        builder.add_body_module_import(Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap());
        builder.add_body_module_import(Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap());

        builder.finish_body();

        let data = builder.extract();

        let view = FunctionView::parse(&data,true).unwrap();

        fn my_catch(view:&FunctionView) -> Result<(),CompilationError>{

            assert_eq!(view.header.module_imports.len(),2);
            assert_eq!(view.header.module_imports.get(0)?,Hash::from_bytes(&PSEUDO_HASH_BYTES)?);
            assert_eq!(view.header.module_imports.get(1)?,Hash::from_bytes(&PSEUDO_HASH_BYTES2)?);

            assert_eq!(view.header.code_hash()?,Hash::from_bytes(&PSEUDO_HASH_BYTES3)?);

            let body = match view.body {
                Some(ref body) => body,
                None => {
                    assert!(false);
                    panic!()
                }
            };

            assert_eq!(body.module_imports.len(),2);
            assert_eq!(body.module_imports.get(0)?,Hash::from_bytes(&PSEUDO_HASH_BYTES)?);
            assert_eq!(body.module_imports.get(1)?,Hash::from_bytes(&PSEUDO_HASH_BYTES2)?);

            Ok(())

        }

        //let res:Result<(),CompilationError> = do catch { }; //works but no syntax for inteliJ
        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok(),res);

    }

    #[test] fn body_module_type_imports(){
        let mut builder = header_with_imports();

        builder.add_body_module_import(Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap());
        builder.add_body_module_import(Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap());

        let mut import = TypeImportBuilder::new();
        import.add_fix_header_data(TypeImportData{
            module_id: ModuleId(3),
            type_index: MemberIndex(4),
            optimisation_declaration: OptimizationDeclaration::Empty,
            kind_declaration: TypeKind::View,
            privileges_declaration: Privileges::no_privileges()
                .add_write_privilege()
                .add_copy_privilege()
                .add_unwrap_privilege()
                .add_access_privilege(),
        });

        import.add_type_apply(TypeId(0));
        import.add_type_apply(TypeId(3));

        builder.add_body_type_import(import);

        import = TypeImportBuilder::new();
        import.add_fix_header_data(TypeImportData{
            module_id: ModuleId(1),
            type_index: MemberIndex(9),
            optimisation_declaration: OptimizationDeclaration::Wrapper,
            kind_declaration: TypeKind::Normal,
            privileges_declaration: Privileges::no_privileges()
                .add_write_privilege()
                .add_copy_privilege()
                .add_unwrap_privilege()
                .add_load_privilege(),
        });

        import.add_type_apply(TypeId(7));
        builder.add_body_type_import(import);

        builder.finish_body();

        let data = builder.extract();

        let view = FunctionView::parse(&data,true).unwrap();

        fn my_catch(view:&FunctionView) -> Result<(),CompilationError>{

            assert_eq!(view.header.module_imports.len(),2);
            assert_eq!(view.header.module_imports.get(0)?,Hash::from_bytes(&PSEUDO_HASH_BYTES)?);
            assert_eq!(view.header.module_imports.get(1)?,Hash::from_bytes(&PSEUDO_HASH_BYTES2)?);

            assert_eq!(view.header.code_hash()?,Hash::from_bytes(&PSEUDO_HASH_BYTES3)?);

            let body = match view.body {
                Some(ref body) => body,
                None => {
                    assert!(false);
                    panic!()
                }
            };

            assert_eq!(body.module_imports.len(),2);
            assert_eq!(body.module_imports.get(0)?,Hash::from_bytes(&PSEUDO_HASH_BYTES)?);
            assert_eq!(body.module_imports.get(1)?,Hash::from_bytes(&PSEUDO_HASH_BYTES2)?);

            assert_eq!(body.type_imports.len(),2);
            let tb1 = body.type_imports.get(0)?;

            assert_eq!(tb1.declaring_module()?,ModuleId(3));
            assert_eq!(tb1.module_version()?,Version(0));
            assert_eq!(tb1.identifying_index()?,MemberIndex(4));
            assert_eq!(tb1.declared_optimisation()?,OptimizationDeclaration::Empty);
            assert_eq!(tb1.declared_privileges()?,Privileges::no_privileges()
                .add_write_privilege()
                .add_copy_privilege()
                .add_unwrap_privilege()
                .add_access_privilege(),
            );
            assert_eq!(tb1.declared_kind()?, TypeKind::View);

            assert_eq!(tb1.params.len(),2);
            assert_eq!(tb1.params.get(0)?,TypeId(0));
            assert_eq!(tb1.params.get(1)?,TypeId(3));

            let tb2 = body.type_imports.get(1)?;

            assert_eq!(tb2.declaring_module()?,ModuleId(1));
            assert_eq!(tb2.module_version()?,Version(0));
            assert_eq!(tb2.identifying_index()?,MemberIndex(9));
            assert_eq!(tb2.declared_optimisation()?,OptimizationDeclaration::Wrapper);
            assert_eq!(tb2.declared_privileges()?,Privileges::no_privileges()
                .add_write_privilege()
                .add_copy_privilege()
                .add_unwrap_privilege()
                .add_load_privilege(),
            );

            assert_eq!(tb2.declared_kind()?, TypeKind::Normal);

            assert_eq!(tb2.params.len(),1);
            assert_eq!(tb2.params.get(0)?,TypeId(7));

            Ok(())

        }

        //let res:Result<(),CompilationError> = do catch { }; //works but no syntax for inteliJ
        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok());

    }



    #[test] fn body_functions_imports(){
        let mut builder = header_and_body_with_imports();

        let mut import = FunctionImportBuilder::new();
        import.add_fix_header_data(FunctionImportData{
            module_id: ModuleId(5),
            fun_index: MemberIndex(3),
            optimisation_declaration: OptimizationDeclaration::Wrapper,
            return_type: TypeId(3),
            return_control: Control::UnusedOwned,
            code_hash: Hash::from_bytes(&PSEUDO_HASH_BYTES3).unwrap(),
        });

        import.add_type_apply(TypeId(1));
        import.add_type_apply(TypeId(4));

        import.add_value_apply(Field(Control::Owned, TypeId(7)));
        import.add_value_apply(Field(Control::UnusedOwned, TypeId(2)));

        builder.add_function_import(import);

        builder.finish_body();

        let data = builder.extract();

        let view = FunctionView::parse(&data,true).unwrap();

        fn my_catch(view:&FunctionView) -> Result<(),CompilationError>{

            let body = match view.body {
                Some(ref body) => body,
                None => {
                    assert!(false);
                    panic!()
                }
            };

            assert_eq!(body.function_imports.len(),1);
            let fb1 = body.function_imports.get(0)?;

            assert_eq!(fb1.declaring_module()?,ModuleId(5));
            assert_eq!(fb1.version()?,Version(0));
            assert_eq!(fb1.identifying_index()?,MemberIndex(3));
            assert_eq!(fb1.declared_optimisation()?,OptimizationDeclaration::Wrapper);
            assert_eq!(fb1.return_type()?,TypeId(3));
            assert_eq!(fb1.return_control()?,Control::UnusedOwned);
            assert_eq!(fb1.code_hash()?,Hash::from_bytes(&PSEUDO_HASH_BYTES3)?);

            assert_eq!(fb1.generic_params.len(),2);
            assert_eq!(fb1.generic_params.get(0)?,TypeId(1));
            assert_eq!(fb1.generic_params.get(1)?,TypeId(4));

            assert_eq!(fb1.params.len(),2);
            assert_eq!(fb1.params.get(0)?,Field(Control::Owned, TypeId(7)));
            assert_eq!(fb1.params.get(1)?,Field(Control::UnusedOwned, TypeId(2)));

            Ok(())

        }

        //let res:Result<(),CompilationError> = do catch { }; //works but no syntax for inteliJ
        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok());

    }

    #[test] fn body_constructor_imports(){
        let mut builder = header_and_body_with_imports_inc_functions();

        let mut import = ConstructorsImportBuilder::new();
        import.corresponding_type(TypeId(9));
        let mut case1 = type_builder::ConstructorCaseBuilder::new();
        case1.add_field(Field(Control::Owned,TypeId(2)));
        case1.add_field(Field(Control::Ref,TypeId(4)));

        import.add_constructor_case(case1);
        let mut case2 = type_builder::ConstructorCaseBuilder::new();
        case2.add_field(Field(Control::UnusedBorrowed,TypeId(0)));

        import.add_constructor_case(case2);
        import.finish();

        builder.add_constructors_import(import);


        builder.finish_body();

        let data = builder.extract();

        let view = FunctionView::parse(&data,true).unwrap();

        fn my_catch(view:&FunctionView) -> Result<(),CompilationError>{

            let body = match view.body {
                Some(ref body) => body,
                None => {
                    assert!(false);
                    panic!()
                }
            };

            assert_eq!(body.constructor_imports.len(),1);
            let c1 = body.constructor_imports.get(0)?;

            assert_eq!(c1.coresponding_type()?,TypeId(9));
            assert_eq!(c1.constructors.len(),2);
            let ca1 = c1.constructors.get(0)?;
            assert_eq!(ca1.params.len(),2);
            assert_eq!(ca1.params.get(0)?,Field(Control::Owned,TypeId(2)));
            assert_eq!(ca1.params.get(1)?,Field(Control::Ref,TypeId(4)));

            let ca2 = c1.constructors.get(1)?;
            assert_eq!(ca2.params.len(),1);
            assert_eq!(ca2.params.get(0)?,Field(Control::UnusedBorrowed,TypeId(0)));
            Ok(())

        }

        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok(), res);

    }

    #[test] fn body_init_imports(){
        let mut builder = header_and_body_with_imports_inc_functions_and_constructors();

        let mut import = InitImportBuilder::new();
        import.corresponding_type(TypeId(3));
        import.add_init_code( Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap());
        import.return_type(TypeId(2));
        builder.add_init_import(import);

        import = InitImportBuilder::new();
        import.corresponding_type(TypeId(8));
        import.return_type(TypeId(6));
        import.add_init_code( Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap());
        builder.add_init_import(import);

        builder.finish_body();

        let data = builder.extract();

        let view = FunctionView::parse(&data,true).unwrap();

        fn my_catch(view:&FunctionView) -> Result<(),CompilationError>{

            let body = match view.body {
                Some(ref body) => body,
                None => {
                    assert!(false);
                    panic!()
                }
            };

            assert_eq!(body.init_imports.len(),2);
            assert_eq!(body.init_imports.get(0)?.coresponding_type()?,TypeId(3));
            assert_eq!(body.init_imports.get(0)?.code_hash()?,Hash::from_bytes(&PSEUDO_HASH_BYTES2)?);
            assert_eq!(body.init_imports.get(0)?.init_return_type()?,TypeId(2));

            assert_eq!(body.init_imports.get(1)?.coresponding_type()?,TypeId(8));
            assert_eq!(body.init_imports.get(1)?.code_hash()?,Hash::from_bytes(&PSEUDO_HASH_BYTES)?);
            assert_eq!(body.init_imports.get(1)?.init_return_type()?,TypeId(6));

            Ok(())

        }

        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok(), res);

    }


}

#[cfg(test)]
mod module_tests {
    use super::*;
    use test::inputgen::module_builder::*;

    const PSEUDO_HASH_BYTES: [u8; HASH_SIZE] = [127; HASH_SIZE];
    const PSEUDO_HASH_BYTES2: [u8; HASH_SIZE] = [97; HASH_SIZE];
    const PSEUDO_HASH_BYTES3: [u8; HASH_SIZE] = [225; HASH_SIZE];

    #[test]
    fn type_imports() {
        let mut builder = ModuleBuilder::new();
        builder.add_type_import(Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap());
        builder.add_type_import(Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap());

        let data = builder.extract();

        let view = ModuleView::parse(&data).unwrap();

        fn my_catch(view:&ModuleView) -> Result<(),CompilationError>{

            assert_eq!(view.types.len(),2);
            assert_eq!(view.types.get(0)?,Hash::from_bytes(&PSEUDO_HASH_BYTES2)?);
            assert_eq!(view.types.get(1)?,Hash::from_bytes(&PSEUDO_HASH_BYTES)?);
            Ok(())

        }

        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok(), res);
    }


    #[test]
    fn function_imports() {
        let mut builder = ModuleBuilder::new();
        builder.add_function_import(Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap());
        builder.add_function_import(Hash::from_bytes(&PSEUDO_HASH_BYTES3).unwrap());

        let data = builder.extract();

        let view = ModuleView::parse(&data).unwrap();

        fn my_catch(view:&ModuleView) -> Result<(),CompilationError>{

            assert_eq!(view.functions.len(),2);
            assert_eq!(view.functions.get(0)?,Hash::from_bytes(&PSEUDO_HASH_BYTES)?);
            assert_eq!(view.functions.get(1)?,Hash::from_bytes(&PSEUDO_HASH_BYTES3)?);
            Ok(())

        }

        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok(), res);
    }

    #[test]
    fn constants_imports() {
        let mut builder = ModuleBuilder::new();
        builder.add_constant_import(Hash::from_bytes(&PSEUDO_HASH_BYTES3).unwrap());
        builder.add_constant_import(Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap());

        let data = builder.extract();

        let view = ModuleView::parse(&data).unwrap();

        fn my_catch(view:&ModuleView) -> Result<(),CompilationError>{

            assert_eq!(view.constants.len(),2);
            assert_eq!(view.constants.get(0)?,Hash::from_bytes(&PSEUDO_HASH_BYTES3)?);
            assert_eq!(view.constants.get(1)?,Hash::from_bytes(&PSEUDO_HASH_BYTES2)?);
            Ok(())

        }

        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok(), res);
    }

    #[test]
    fn all_imports() {
        let mut builder = ModuleBuilder::new();

        builder.add_type_import(Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap());
        builder.add_type_import(Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap());

        builder.add_function_import(Hash::from_bytes(&PSEUDO_HASH_BYTES3).unwrap());
        builder.add_function_import(Hash::from_bytes(&PSEUDO_HASH_BYTES).unwrap());

        builder.add_constant_import(Hash::from_bytes(&PSEUDO_HASH_BYTES2).unwrap());
        builder.add_constant_import(Hash::from_bytes(&PSEUDO_HASH_BYTES3).unwrap());

        let data = builder.extract();

        let view = ModuleView::parse(&data).unwrap();

        fn my_catch(view:&ModuleView) -> Result<(),CompilationError>{

            assert_eq!(view.types.len(),2);
            assert_eq!(view.types.get(0)?,Hash::from_bytes(&PSEUDO_HASH_BYTES)?);
            assert_eq!(view.types.get(1)?,Hash::from_bytes(&PSEUDO_HASH_BYTES2)?);

            assert_eq!(view.functions.len(),2);
            assert_eq!(view.functions.get(0)?,Hash::from_bytes(&PSEUDO_HASH_BYTES3)?);
            assert_eq!(view.functions.get(1)?,Hash::from_bytes(&PSEUDO_HASH_BYTES)?);

            assert_eq!(view.constants.len(),2);
            assert_eq!(view.constants.get(0)?,Hash::from_bytes(&PSEUDO_HASH_BYTES2)?);
            assert_eq!(view.constants.get(1)?,Hash::from_bytes(&PSEUDO_HASH_BYTES3)?);
            Ok(())

        }

        let res:Result<(),CompilationError> = my_catch(&view);
        assert!(res.is_ok(), res);
    }
}