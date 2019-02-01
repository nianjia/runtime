use super::common;
use super::function::BranchTarget;
use super::{
    value::Value, ContextCodeGen, ContorlContextType, ControlContext, FunctionCodeGen,
    ModuleCodeGen,
};
use std::rc::Rc;
use wasm::{
    call_conv::CallConv as WASMCallConv, BlockType, BrTableData, FunctionType,
    Module as WASMModule, ValueType,
};

pub trait ControlInstrEmit<'ll> {
    declare_control_instrs!(declear_op);
}

impl<'ll> ControlInstrEmit<'ll> for FunctionCodeGen<'ll> {
    fn block(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
        ty: BlockType,
    ) {
        let res_type = match ty {
            BlockType::Value(v) => Some(ValueType::from(v)),
            _ => None,
        };

        let end_block = ctx.create_basic_block("blockEnd", self);
        let end_PHIs = res_type.map(|ty| self.create_PHIs(ctx, end_block, ty));

        self.push_control_stack(
            ContorlContextType::Block,
            res_type,
            end_block,
            end_PHIs,
            None,
        );

        self.branch_target_stack.push(BranchTarget {
            block: end_block,
            type_PHIs: res_type.map(|ty| (ty, end_PHIs.unwrap())),
        })
    }

    fn loop_(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
        ty: BlockType,
    ) {
        let res_type = match ty {
            BlockType::Value(v) => Some(ValueType::from(v)),
            _ => None,
        };
        // let enrty_block = self.builder.get_insert_block();

        let loop_body_block = ctx.create_basic_block("loopBody", self);
        let end_block = ctx.create_basic_block("loopEnd", self);
        let end_PHIs = res_type.map(|ty| self.create_PHIs(ctx, end_block, ty));

        self.builder.create_br_instr(loop_body_block);
        self.builder.set_insert_block(loop_body_block);

        self.push_control_stack(
            ContorlContextType::Loop,
            res_type,
            end_block,
            end_PHIs,
            None,
        );
    }

    fn if_(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
        ty: BlockType,
    ) {
        let res_type = match ty {
            BlockType::Value(v) => Some(ValueType::from(v)),
            _ => None,
        };

        let then_block = ctx.create_basic_block("ifThen", self);
        let else_block = ctx.create_basic_block("ifElse", self);
        let end_block = ctx.create_basic_block("ifElseEnd", self);
        let end_PHIs = res_type.map(|ty| self.create_PHIs(ctx, end_block, ty));

        let cond = self.pop();
        let cond_bool = ctx.coerce_i32_to_bool(self.builder, cond);
        self.builder
            .create_cond_br_instr(cond_bool, then_block, else_block);

        self.builder.set_insert_block(then_block);

        self.push_control_stack(
            ContorlContextType::IfElse,
            res_type,
            end_block,
            end_PHIs,
            Some(else_block),
        );

        self.branch_target_stack.push(BranchTarget {
            // param_types: res_ty,
            block: end_block,
            type_PHIs: res_type.map(|ty| (ty, end_PHIs.unwrap())),
        });
    }

    fn else_(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
    ) {
        assert!(self.control_stack.len() != 0);

        self.branch_to_end_of_control_context(ctx);

        let cur_ctx = self.control_stack.last_mut().unwrap();

        assert!(cur_ctx.else_block.is_some());
        assert!(cur_ctx.ty == ContorlContextType::IfThen);

        let else_block = cur_ctx.else_block.unwrap();
        else_block.move_after(self.builder.get_insert_block());

        // TODO: check whether need else arguments.
        // cur_ctx.else_args.clone().into_iter().for_each(|t| {});

        cur_ctx.ty = ContorlContextType::IfElse;
        cur_ctx.is_reachable = true;
        cur_ctx.else_block = None;
    }

    fn end(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
    ) {
        let (PHI, res_type) = {
            assert!(self.control_stack.len() > 0);

            self.branch_to_end_of_control_context(ctx);

            let cur_ctx = self.control_stack.last().unwrap();
            if let Some(else_block) = cur_ctx.else_block {
                else_block.move_after(self.builder.get_insert_block());
                self.builder.set_insert_block(else_block);
                self.builder.create_br_instr(cur_ctx.end_block);

                // assert!(cur_ctx.else_args.len() == cur_ctx.end_PHIs.len());

                // TODO
                // (0..cur_ctx.else_args.len())
                //     .for_each(|t| cur_ctx.end_PHIs[t].add_incoming(cur_ctx.else_args[t], else_block));
            }

            match cur_ctx.ty {
                ContorlContextType::Try => { /* TODO: Add end_try */ }
                ContorlContextType::Catch => { /* TODO: Add end_catch */ }
                _ => {}
            };

            cur_ctx
                .end_block
                .move_after(self.builder.get_insert_block());
            self.builder.set_insert_block(cur_ctx.end_block);

            assert!(
                (cur_ctx.end_PHIs.is_some() && cur_ctx.res_types.is_some())
                    || (cur_ctx.end_PHIs.is_none() && cur_ctx.res_types.is_none())
            );
            if cur_ctx.end_PHIs.is_none() {
                return;
            }
            (cur_ctx.end_PHIs.unwrap(), cur_ctx.res_types.unwrap())
        };

        // if let Some(PHI) = cur_ctx.end_PHIs {
        if PHI.count_incoming() == 0 {
            PHI.erase_from_parent();
            self.push(ctx.typed_zero_constants[res_type as usize]);
        } else {
            self.push(Value::from(*PHI));
        }
        // }
    }

    fn br(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
        depth: u32,
    ) {
        if let Some((ty, PHI)) = self.get_branch_target(depth).type_PHIs {
            let res = self.pop();
            PHI.add_incoming(
                ctx.coerce_to_canonical_type(self.builder, res),
                self.builder.get_insert_block(),
            )
        }

        self.builder
            .create_br_instr(self.get_branch_target(depth).block);
    }

    fn br_if(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
        depth: u32,
    ) {
        let cond = self.pop();
        let target = self.get_branch_target(depth);
        if let Some((ty, PHI)) = target.type_PHIs {
            let arg = self.get_value_from_stack(0);
            PHI.add_incoming(
                ctx.coerce_to_canonical_type(self.builder, arg),
                self.builder.get_insert_block(),
            );
        }

        let false_block = ctx.create_basic_block("br_ifElse", self);
        self.builder.create_cond_br_instr(
            ctx.coerce_i32_to_bool(self.builder, cond),
            target.block,
            false_block,
        );
        self.builder.set_insert_block(false_block);
    }

    fn return_(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
    ) {
        match self.func_ty.res() {
            None => {}
            _ => {
                let v = self.pop();
                let cur_ctx = self.control_stack.first().unwrap();
                cur_ctx.end_PHIs.unwrap().add_incoming(
                    ctx.coerce_to_canonical_type(self.builder, v),
                    self.builder.get_insert_block(),
                );
            }
        }

        self.builder
            .create_br_instr(self.control_stack.first().unwrap().end_block);
        self.enter_unreachable();
    }

    fn br_table(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
        data: Box<BrTableData>,
    ) {
        let index = self.pop();
        // let num_args = self.get_branch_target(data.default).type_PHIs.len();
        // let args = (0..num_args).map(|_| self.pop()).rev().collect::<Vec<_>>();

        let ll_switch = {
            let default_target = self.get_branch_target(data.default);
            // TODO
            // args.iter().enumerate().for_each(|(t, v)| {
            //     default_target.type_PHIs[t].1.add_incoming(
            //         ctx.coerce_to_canonical_type(self.builder, *v),
            //         self.builder.get_insert_block(),
            //     );
            // });

            self.builder
                .create_switch(index, default_target.block, data.table.len())
        };

        data.table.iter().enumerate().for_each(|(i, item)| {
            let target = self.get_branch_target(*item);
            assert!(i < std::u32::MAX as usize);

            ll_switch.add_case(
                common::const_u32(ctx.get_llvm_wrapper(), i as u32),
                target.block,
            );

            // TODO

            // assert!(target.type_PHIs.len() == num_args);
            // args.iter().enumerate().for_each(|(t, v)| {
            //     target.type_PHIs[t].1.add_incoming(
            //         ctx.coerce_to_canonical_type(self.builder, *v),
            //         self.builder.get_insert_block(),
            //     );
            // });
        });

        self.enter_unreachable();
    }

    fn call(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
        index: u32,
    ) {
        // unimplemented!()
        let callee = module.functions()[index as usize];
        let callee_type = wasm_module.functions().get_type(index as usize);
        let mut args = vec![self.builder.create_load(self.ctx_ptr.unwrap())];
        args.extend(
            self.pop_multi(callee_type.params().len())
                .iter()
                .map(|t| ctx.coerce_to_canonical_type(self.builder, *t)),
        );

        let res = ctx.emit_call_or_invoke(callee, args, WASMCallConv::Wasm, self.builder);

        self.push(res);
    }

    fn unreachable_(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
    ) {
        self.emit_runtime_intrinsic("unreachableTrap", FunctionType::default(), Vec::new());
        self.builder.create_unreachable();
        self.enter_unreachable();
    }

    fn call_indirect(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
        ty_index: u32,
        table_index: u8,
    ) {
        // let index = self.pop();
        // let func_ty = match self.module.get_wasm_module().type_section() {
        //     Some(section) => section.types()[ty_index as usize].clone(),
        //     None => panic!(),
        // };

        // TODO
    }

    fn nop(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
    ) {
    }

    fn drop(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
    ) {
        self.pop();
    }

    fn select_(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
    ) {
        let cond = self.pop();
        let cond_bool = ctx.coerce_i32_to_bool(self.builder, cond);
        let false_value = self.pop();
        let true_value = self.pop();
        let val = self
            .builder
            .create_select(cond_bool, true_value, false_value);
        self.push(val);
    }
}
