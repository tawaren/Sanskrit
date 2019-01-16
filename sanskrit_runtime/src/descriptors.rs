use interpreter::*;
use script_stack::*;
use sanskrit_common::model::*;
use sanskrit_common::errors::*;
use sanskrit_common::linear_stack::*;
use sanskrit_common::capabilities::CapSet;
use model::*;
use ContextEnvironment;
use sanskrit_common::arena::*;

impl<'b> AdtDescriptor<'b> {

    //builds a Runtime type from the descriptor
    pub fn build_type<'a,'h>(&self, applies:SlicePtr<'a,Ptr<'a,RuntimeType<'a>>>, alloc:&'a VirtualHeapArena<'h>) -> Result<Ptr<'a,RuntimeType<'a>>> {

        //Check applies and infer caps
        let mut caps = self.base_caps;
        for (TypeTypeParam(is_phantom,reqs),appl) in self.generics.iter().zip(applies.iter()) {
            //get the caps of the apply
            let cap_set = appl.get_caps();
            //check if the constraints are full filled
            if !reqs.is_subset_of(cap_set){
                return type_apply_constraint_violation()
            }
            //  phantom types are newType & accountTypes
            if !*is_phantom {
                //if not phantom eliminate non supported recursive caps (non-recursives will be added in again later)
                caps = caps.intersect(cap_set);
                // check that no phantom type is a Applied to non-phantom
                match **appl {
                    RuntimeType::Custom { .. } | RuntimeType::NativeType { .. } => {},
                    RuntimeType::NewType { .. } | RuntimeType::AccountType { .. } => return can_not_apply_phantom_to_physical_error(),
                }
            }
        }
        //add in the non recursives again
        caps = caps.union(self.base_caps.intersect(CapSet::non_recursive()));

        //build the runtime type
        alloc.alloc(match self.id {
            AdtId::Custom(module, offset) => RuntimeType::Custom {
                caps,
                module,
                offset,
                applies
            },
            AdtId::Native(typ) => RuntimeType::NativeType {
                caps,
                typ,
                applies
            }
        })
    }

    //todo: can we have the fetchMode::Copy as well (would need check that can Copy)
    //executes a pack for the descriptor on value and type level
    pub fn pack<'a,'h>(&self, applies:SlicePtr<'a,Ptr<'a,RuntimeType<'a>>>, Tag(t):Tag, params:&[ValueRef], is_borrow:bool, stack:&mut LinearScriptStack<'a,'h>, alloc:&'a VirtualHeapArena<'h> ) -> Result<()> {

        //endure that the type supports the create capability
        if !self.base_caps.contains(NativeCap::Create) {
            return capability_missing_error()

        }

        //check that the number of applied types params is correct
        if applies.len() != self.generics.len() {
            return can_not_apply_phantom_to_physical_error()
        }

        //ensure the requested constructor exists
        if t as usize >=  self.constructors.len() {
            return requested_ctr_missing()
        }

        //fetch the requested constructor
        let ctr = &self.constructors[t as usize];

        //ensure the number of provided parameters is correct
        if params.len() != ctr.len() {
            return num_fields_mismatch()
        }

        //ensure that their is something to borrow if we borrow
        if params.is_empty() && is_borrow {
            return empty_borrow_error()
        }

        //check the field types and calculate the field values
        let mut fields = alloc.slice_builder(params.len())?;
        for (builder, index) in ctr.iter().zip(params.iter()) {
            //Fetch the input
            let StackEntry{ref typ,ref val,..} = stack.value_of(*index)?;

            //check that we do only use Pack on borrowed values if we borrow ourself
            if stack.is_borrowed(*index)? && !is_borrow {
                return borrow_input_error()
            }

            //construct the field type
            let field_type = builder.execute(&applies, alloc)?;

            //check tha the param type is the field type
            if field_type != *typ {
                return type_mismatch()
            }
            //save the value for later
            fields.push(*val);
        }

        //build the type of the resulting adt
        let new_typ = self.build_type(applies, alloc)?;
        //get the slice
        let fields = fields.finish();
        //generate the value
        let new_val = if self.constructors.len() == 1 && fields.len() == 1 {
            //Wrapper Optimisation (Elimination of new type patter)
            fields[0].clone()
        } else {
            alloc.alloc(Object::Adt(t,fields))?
        };

        //generate the result
        let new_entry = StackEntry::new(new_val,new_typ);
        //apply it to the stack
        if is_borrow {
            stack.pack(&params,new_entry, FetchMode::Borrow)
        } else {
            stack.pack(&params,new_entry, FetchMode::Consume)
        }
    }

    //executes an unpack for the descriptor on value and type level
    pub fn unpack<'a,'h>(&self, packed:ValueRef, Tag(expected_tag):Tag, is_borrow:bool, stack:&mut LinearScriptStack<'a,'h>, alloc:&'a VirtualHeapArena<'h>, temporary_values:&HeapArena<'h>) -> Result<()> {
        //get the input
        let StackEntry{ref typ,ref val, ..} = stack.value_of(packed)?;

        //extract the type param from the input type & check the input
        let applies = match **typ {
            RuntimeType::Custom { ref applies, module, offset,.. } => {
                match self.id {
                    //Ensure that the input and the descriptor match
                    AdtId::Custom(self_module, self_offset) => if self_module != module || self_offset != offset {
                        return type_mismatch()
                    },
                    AdtId::Native(_) => return type_mismatch(),
                }
                applies
            },

            RuntimeType::NativeType { ref applies, typ,.. }  => {
                match self.id {
                    AdtId::Custom(_, _) => return type_mismatch(),
                    //Ensure that the input and the descriptor match
                    AdtId::Native(self_typ) =>  if self_typ != typ {
                        return type_mismatch()
                    },
                }
                applies
            },
            _ => unreachable!()
        };

        if is_borrow {
            //if borrowed it must be inspectable
            if !self.base_caps.contains(NativeCap::Inspect) {
                return capability_missing_error()
            }
        } else {
            //if not it must e consumable
            if !self.base_caps.contains(NativeCap::Consume) {
                return  capability_missing_error()
            }

            //ensure that we not consume borrowed
            if stack.is_borrowed(packed)? {
                return borrow_input_error()
            }
        }


        let temp = temporary_values.temp_arena()?;
        //generate the resulting fields
        let elems = if self.constructors.len() == 1 && self.constructors[0].len() == 1 {
            //Wrapper Optimisation Branch
            //ensure that the tag is 0
            if expected_tag != 0 { return constructor_mismatch() }
            //fetch the single field
            let type_b = &self.constructors[0][0];
            //change the type but copy the value (wrapper optim)
            temp.copy_alloc_slice(&[StackEntry::new(
                val.clone(),
                type_b.execute(applies, alloc)?
            )])?
        } else {
            //get the types & value
            match **val {
                Object::Adt(t, ref fields) => {
                    //check that the tag matches
                    if expected_tag != t { return constructor_mismatch() }
                    //fetch the corresponding ctr
                    let ctr_typs = &self.constructors[t as usize];
                    //for each field build a StackEntry
                    temp.iter_result_alloc_slice(fields.iter().zip(ctr_typs.iter()).map(|(obj, type_b)| Ok(StackEntry::new(
                        obj.clone(),
                        type_b.execute(applies, alloc)?
                    ))))?
                },
                _ => return requested_ctr_missing()
            }
        };
        //apply the unpack to the stack
        stack.unpack(packed,&elems, is_borrow)
    }

}

impl<'b> FunctionDescriptor<'b> {

    //Executes a function
    pub fn apply<'a, 'h>(&self, applies:&[(bool, Ptr<'a,RuntimeType<'a>>)], params:&[ValueRef], stack:&mut LinearScriptStack<'a,'h>, env:ContextEnvironment, alloc:&'a VirtualHeapArena<'h>, stack_alloc:&'a HeapArena<'h>, temporary_values:&HeapArena<'h>) -> Result<()> {


        //chekc that the right amount of type parameters are applied
        if applies.len() != self.generics.len() {
            return num_applied_generics_error()
        }

        let tmp = temporary_values.temp_arena()?;
        //check the type parameters and prepare them for application
        let mut plain_applies = tmp.slice_builder(applies.len())?;
        for (FunTypeParam{is_protected, is_phantom, caps},(is_priv,typ)) in self.generics.iter().zip(applies.iter()) {
            //ckeck that the constraints are full filled
            if !caps.is_subset_of(typ.get_caps()) {
                return type_apply_constraint_violation()
            }

            //check that protected parameters are applied with privileged types (nEwType & AccountTypes)
            if is_protected & !is_priv {
                return visibility_violation()
            }

            if !is_phantom {
                // check that no phantom type is a Applied to non-phantom
                match **typ {
                    RuntimeType::Custom { .. } | RuntimeType::NativeType { .. } => {},
                    RuntimeType::NewType { .. } | RuntimeType::AccountType { .. } => return can_not_apply_phantom_to_physical_error(),
                }
            }
            //Extract the type part
            plain_applies.push(typ.clone());
        }
        let plain_applies = plain_applies.finish();

        //check that the right amount of parameters are supplied
        if params.len() != self.params.len() {
            return num_param_mismatch()
        }

        //prepare the stacks needed for interpretation

        let tmp_stack_alloc = stack_alloc.temp_arena()?;
        let mut value_stack = tmp_stack_alloc.alloc_stack(1000)?;
        let mut frame_stack = tmp_stack_alloc.alloc_stack(1000)?;

        //extract the information needed to execute the function on the stack
        assert_eq!(value_stack.len(), 0);
        let mut param_types = tmp.slice_builder(params.len())?;
        for (Param(is_consume,builder), index) in self.params.iter().zip(params.iter()) {
            let StackEntry{ref typ,ref val, ..} = stack.value_of(*index)?;
            //build the expected parameter type
            let param_type = builder.execute(&plain_applies, alloc)?;
            //check he parameter type is the expected one
            if param_type != *typ {
                return type_mismatch()
            }
            //capture values needed
            value_stack.push(*val)?;
            param_types.push((*index,*is_consume))
        }
        //let param_vals = param_vals.finish();
        let param_types = param_types.finish();
        //Execute the function (
        ExecutionContext::interpret(env,self.functions.0, &mut value_stack, &mut frame_stack, alloc, &tmp)?;

        assert_eq!(value_stack.len(), self.returns.len() );

        //Extract the return values & types
        let mut ret_elems = tmp.slice_builder(self.returns.len())?;
        for (Return(builder,borrows), val) in self.returns.iter().zip(value_stack.as_slice().iter()) {
            //build the return type
            let typ = builder.execute(&plain_applies, alloc)?;
            //create the stack entry
            ret_elems.push((StackEntry::new(*val, typ), &borrows[..]));
        }
        value_stack.rewind_to(0)?;
        assert_eq!(value_stack.len(), 0);
        //Make the necessary stack transformation
        stack.apply(&param_types,&ret_elems.finish())
    }
}