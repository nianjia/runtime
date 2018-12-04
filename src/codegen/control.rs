use super::common;
use super::function::BranchTarget;
use super::{ContextCodeGen, ContorlContextType, ControlContext, FunctionCodeGen};
use llvm_sys::LLVMCallConv;
use std::rc::Rc;
use wasm::{BlockType, BrTableData, FunctionType, ValueType};

trait ControlInstrEmit {
    declare_control_instrs!();
}

impl ControlInstrEmit for FunctionCodeGen {
    fn block(&mut self, ctx: &ContextCodeGen, ty: BlockType) {
        let res_ty = ValueType::from(ty);

        let end_block = ctx.create_basic_block("blockEnd", self);
        let end_PHIs = self.create_PHIs(ctx, end_block, &vec![res_ty]);

        self.push_control_stack(
            ContorlContextType::Block,
            vec![res_ty],
            end_block,
            end_PHIs.clone(),
            None,
        );

        self.branch_target_stack.push(BranchTarget {
            block: end_block,
            type_PHIs: vec![(res_ty, *end_PHIs.first().unwrap())],
        })
    }

    fn loop_(&mut self, ctx: &ContextCodeGen, ty: BlockType) {
        let res_ty = vec![ValueType::from(ty)];
        // let enrty_block = self.builder.get_insert_block();

        let loop_body_block = ctx.create_basic_block("loopBody", self);
        let end_block = ctx.create_basic_block("loopEnd", self);
        let end_PHIs = self.create_PHIs(ctx, end_block, &res_ty);

        ctx.get_builder().create_br_instr(loop_body_block);
        ctx.get_builder().set_insert_block(loop_body_block);

        self.push_control_stack(ContorlContextType::Loop, res_ty, end_block, end_PHIs, None);
    }

    fn if_(&mut self, ctx: &ContextCodeGen, ty: BlockType) {
        let res_ty = ValueType::from(ty);

        let then_block = ctx.create_basic_block("ifThen", self);
        let else_block = ctx.create_basic_block("ifElse", self);
        let end_block = ctx.create_basic_block("ifElseEnd", self);
        let end_PHIs = self.create_PHIs(ctx, end_block, &vec![res_ty]);

        let cond = self.pop();
        let cond_bool = ctx.coerce_i32_to_bool(ctx.get_builder(), cond);
        self.builder
            .create_cond_br_instr(cond_bool, then_block, else_block);

        self.builder.set_insert_block(then_block);

        self.push_control_stack(
            ContorlContextType::IfElse,
            vec![res_ty],
            end_block,
            end_PHIs.clone(),
            Some(else_block),
        );

        self.branch_target_stack.push(BranchTarget {
            // param_types: res_ty,
            block: end_block,
            type_PHIs: vec![(res_ty, *end_PHIs.first().unwrap())],
        });
    }

    fn else_(&mut self, ctx: &ContextCodeGen) {
        assert!(self.control_stack.len() != 0);

        self.branch_to_end_of_control_context(ctx);

        let cur_ctx = self.control_stack.last_mut().unwrap();

        assert!(cur_ctx.else_block.is_some());
        assert!(cur_ctx.ty == ContorlContextType::IfThen);

        let else_block = cur_ctx.else_block.unwrap().clone();
        else_block.move_after(self.builder.get_insert_block());

        // TODO: check whether need else arguments.
        // cur_ctx.else_args.clone().into_iter().for_each(|t| {});

        cur_ctx.ty = ContorlContextType::IfElse;
        cur_ctx.is_reachable = true;
        cur_ctx.else_block = None;
    }

    fn end(&mut self, ctx: &ContextCodeGen) {
        assert!(self.control_stack.len() != 0);

        self.branch_to_end_of_control_context(ctx);

        let cur_ctx = self.control_stack.last().unwrap();
        if let Some(else_block) = cur_ctx.else_block {
            else_block.move_after(self.builder.get_insert_block());
            self.builder.set_insert_block(else_block);
            self.builder.create_br_instr(cur_ctx.end_block);

            assert!(cur_ctx.else_args.len() == cur_ctx.end_PHIs.len());

            (0..cur_ctx.else_args.len())
                .for_each(|t| cur_ctx.end_PHIs[t].add_incoming(cur_ctx.else_args[t], else_block));
        }

        match cur_ctx.ty {
            ContorlContextType::Try => { /* TODO: Add end_try */ }
            ContorlContextType::Catch => { /* TODO: Add end_catch */ }
            _ => {}
        }

        cur_ctx
            .end_block
            .move_after(self.builder.get_insert_block());
        self.builder.set_insert_block(cur_ctx.end_block);

        assert!(cur_ctx.end_PHIs.len() == cur_ctx.res_types.len());
        (0..cur_ctx.end_PHIs.len()).for_each(|t| {
            if cur_ctx.end_PHIs[t].count_incoming() == 0 {

            } else {
                //self.push(cur_ctx.end_PHIs[t]);
            }
        });
    }

    fn br(&mut self, ctx: &ContextCodeGen, depth: u32) {
        let len = self.get_branch_target(depth).type_PHIs.len();

        (0..len).for_each(|t| {
            let res = self.pop();
            let (ty, PHI) = self.get_branch_target(depth).type_PHIs[t];
            PHI.add_incoming(
                ctx.coerce_to_canonical_type(self.builder, res),
                self.builder.get_insert_block(),
            )
        });

        self.builder
            .create_br_instr(self.get_branch_target(depth).block);
    }

    fn br_if(&mut self, ctx: &ContextCodeGen, depth: u32) {
        let cond = self.pop();

        let target = self.get_branch_target(depth);
        let len = target.type_PHIs.len();
        (0..target.type_PHIs.len()).for_each(|t| {
            let arg = self.get_value_from_stack(len - t - 1);
            target.type_PHIs[t].1.add_incoming(
                ctx.coerce_to_canonical_type(self.builder, arg),
                self.builder.get_insert_block(),
            );
        });

        let false_block = ctx.create_basic_block("br_ifElse", self);
        self.builder.create_cond_br_instr(
            ctx.coerce_i32_to_bool(self.builder, cond),
            target.block,
            false_block,
        );
        self.builder.set_insert_block(false_block);
    }

    fn return_(&mut self, ctx: &ContextCodeGen) {
        match self.func_ty.return_type() {
            Some(_) => {
                let v = self.pop();
                let cur_ctx = self.control_stack.first().unwrap();
                cur_ctx.end_PHIs[0].add_incoming(
                    ctx.coerce_to_canonical_type(self.builder, v),
                    self.builder.get_insert_block(),
                );
            }
            None => {}
        }

        self.builder
            .create_br_instr(self.control_stack.first().unwrap().end_block);
        self.enter_unreachable();
    }

    fn br_table(&mut self, ctx: &ContextCodeGen, data: BrTableData) {
        let index = self.pop();
        let num_args = self.get_branch_target(data.default).type_PHIs.len();
        let args = (0..num_args).map(|_| self.pop()).rev().collect::<Vec<_>>();

        let ll_switch = {
            let default_target = self.get_branch_target(data.default);

            args.iter().enumerate().for_each(|(t, v)| {
                default_target.type_PHIs[t].1.add_incoming(
                    ctx.coerce_to_canonical_type(self.builder, *v),
                    self.builder.get_insert_block(),
                );
            });

            self.builder
                .create_switch(index, default_target.block, data.table.len())
        };

        data.table.iter().enumerate().for_each(|(i, item)| {
            let target = self.get_branch_target(*item);
            assert!(i < std::u32::MAX as usize);

            ll_switch.add_case(
                common::const_u32(*ctx.get_llvm_wrapper(), i as u32),
                target.block,
            );

            assert!(target.type_PHIs.len() == num_args);
            args.iter().enumerate().for_each(|(t, v)| {
                target.type_PHIs[t].1.add_incoming(
                    ctx.coerce_to_canonical_type(self.builder, *v),
                    self.builder.get_insert_block(),
                );
            });
        });

        self.enter_unreachable();
    }

    fn call(&mut self, ctx: &ContextCodeGen, index: u32) {
        unimplemented!()
        // let callee_type = self.module.get_function(index).get_func_type();
        // let ll_args = (0..callee_type.params().len())
        //     .map(|_| {
        //         let v = self.pop();
        //         self.ctx.coerce_to_canonical_type(self.builder, v)
        //     })
        //     .rev()
        //     .collect::<Vec<_>>();

        // let res = self.ctx.emit_call_or_invoke(
        //     self.module.get_function(index),
        //     ll_args,
        //     LLVMCallConv::LLVMFastCallConv,
        // );

        // res.iter().for_each(|v| self.push(*v));
    }

    fn unreachable(&mut self, ctx: &ContextCodeGen) {
        self.emit_runtime_intrinsic("unreachableTrap", FunctionType::default(), Vec::new());
        self.builder.create_unreachable();
        self.enter_unreachable();
    }

    fn call_indirect(&mut self, ctx: &ContextCodeGen, ty_index: u32, table_index: u8) {
        // let index = self.pop();
        // let func_ty = match self.module.get_wasm_module().type_section() {
        //     Some(section) => section.types()[ty_index as usize].clone(),
        //     None => panic!(),
        // };

        // TODO
    }

    fn nop(&mut self, ctx: &ContextCodeGen) {}

    fn drop(&mut self, ctx: &ContextCodeGen) {
        self.pop();
    }

    fn select(&mut self, ctx: &ContextCodeGen) {
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
