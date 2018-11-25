use super::function::BranchTarget;
use super::{ContorlContextType, ControlContext, FunctionCodeGen};
use std::rc::Rc;
use wasm::{BlockType, FunctionType, ValueType};

trait ControlInstrEmit {
    declare_control_instrs!();
}

impl<'a> ControlInstrEmit for FunctionCodeGen<'a> {
    fn block(&mut self, ty: BlockType) {
        let res_ty = ValueType::from(ty);

        let end_block = self.ctx.create_basic_block("blockEnd", self.ll_func);
        let end_PHIs = self.create_PHIs(end_block, &vec![res_ty]);

        self.control_stack.push(ControlContext::new(
            ContorlContextType::Block,
            vec![res_ty],
            end_block,
            end_PHIs.clone(),
            None,
            self.stack.len(),
            self.branch_target_stack.len(),
        ));

        self.branch_target_stack.push(BranchTarget {
            block: end_block,
            type_PHIs: vec![(res_ty, end_PHIs.first().unwrap())],
        })
    }

    fn loop_(&mut self, ty: BlockType) {
        let res_ty = vec![ValueType::from(ty)];
        // let enrty_block = self.builder.get_insert_block();

        let loop_body_block = self.ctx.create_basic_block("loopBody", self.ll_func);
        let end_block = self.ctx.create_basic_block("loopEnd", self.ll_func);
        let end_PHIs = self.create_PHIs(end_block, &res_ty);

        self.builder.create_br_instr(loop_body_block);
        self.builder.set_insert_block(loop_body_block);

        self.control_stack.push(ControlContext::new(
            ContorlContextType::Loop,
            res_ty,
            end_block,
            end_PHIs,
            None,
            self.stack.len(),
            self.branch_target_stack.len(),
        ));
    }

    fn if_(&mut self, ty: BlockType) {
        let res_ty = ValueType::from(ty);

        let then_block = self.ctx.create_basic_block("ifThen", self.ll_func);
        let else_block = self.ctx.create_basic_block("ifElse", self.ll_func);
        let end_block = self.ctx.create_basic_block("ifElseEnd", self.ll_func);
        let end_PHIs = self.create_PHIs(end_block, &vec![res_ty]);

        let cond = self.pop();
        let cond_bool = self.ctx.coerce_i32_to_bool(self.builder, cond);
        self.builder
            .create_cond_br_instr(cond_bool, then_block, else_block);

        self.builder.set_insert_block(then_block);

        self.control_stack.push(ControlContext::new(
            ContorlContextType::IfElse,
            vec![res_ty],
            end_block,
            end_PHIs.clone(),
            Some(else_block),
            self.stack.len(),
            self.branch_target_stack.len(),
        ));

        self.branch_target_stack.push(BranchTarget {
            // param_types: res_ty,
            block: end_block,
            type_PHIs: vec![(res_ty, end_PHIs.first().unwrap())],
        });
    }

    fn else_(&mut self) {
        assert!(self.control_stack.len() != 0);

        self.branch_to_end_of_control_context();

        let cur_ctx = self.control_stack.last_mut().unwrap();

        assert!(cur_ctx.else_block.is_some());
        assert!(cur_ctx.ty == ContorlContextType::IfThen);

        let else_block = cur_ctx.else_block.unwrap().clone();
        else_block.move_after(self.builder.get_insert_block().unwrap());

        // TODO: check whether need else arguments.
        // cur_ctx.else_args.clone().into_iter().for_each(|t| {});

        cur_ctx.ty = ContorlContextType::IfElse;
        cur_ctx.is_reachable = true;
        cur_ctx.else_block = None;
    }

    fn end(&mut self) {
        assert!(self.control_stack.len() != 0);

        self.branch_to_end_of_control_context();

        let cur_ctx = self.control_stack.last().unwrap();
        if let Some(else_block) = cur_ctx.else_block {
            else_block.move_after(self.builder.get_insert_block().unwrap());
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
            .move_after(self.builder.get_insert_block().unwrap());
        self.builder.set_insert_block(cur_ctx.end_block);

        assert!(cur_ctx.end_PHIs.len() == cur_ctx.res_types.len());
        (0..cur_ctx.end_PHIs.len()).for_each(|t| {
            if cur_ctx.end_PHIs[t].count_incoming() == 0 {

            } else {
                //self.push(cur_ctx.end_PHIs[t]);
            }
        });
    }

    fn br(&mut self, depth: u32) {
        let len = self.get_branch_target(depth).type_PHIs.len();

        (0..len).for_each(|t| {
            let res = self.pop();
            let (ty, PHI) = self.get_branch_target(depth).type_PHIs[t];
            PHI.add_incoming(
                self.ctx.coerce_to_canonical_type(self.builder, res),
                self.builder.get_insert_block().unwrap(),
            )
        });

        self.builder
            .create_br_instr(self.get_branch_target(depth).block);
    }

    fn br_if(&mut self, depth: u32) {
        let cond = self.pop();

        let target = self.get_branch_target(depth);
        let len = target.type_PHIs.len();
        (0..target.type_PHIs.len()).for_each(|t| {
            let arg = self.get_value_from_stack(len - t - 1);
            target.type_PHIs[t].1.add_incoming(
                self.ctx.coerce_to_canonical_type(self.builder, arg),
                self.builder.get_insert_block().unwrap(),
            );
        });

        let false_block = self.ctx.create_basic_block("br_ifElse", self.ll_func);
        self.builder.create_cond_br_instr(
            self.ctx.coerce_i32_to_bool(self.builder, cond),
            target.block,
            false_block,
        );
        self.builder.set_insert_block(false_block);
    }

    fn return_(&mut self) {
        match self.func_ty.return_type() {
            Some(_) => {
                let v = self.pop();
                let cur_ctx = self.control_stack.first().unwrap();
                cur_ctx.end_PHIs[0].add_incoming(
                    self.ctx.coerce_to_canonical_type(self.builder, v),
                    self.builder.get_insert_block().unwrap(),
                );
            }
            None => {}
        }

        self.builder
            .create_br_instr(self.control_stack.first().unwrap().end_block);
        self.enter_unreachable();
    }

    fn br_table(&mut self, depth: u32) {}

    fn call(&mut self, index: u32) {}

    fn unreachable(&mut self) {
        self.emit_runtime_intrinsic("unreachableTrap", FunctionType::default(), Vec::new());
        self.builder.create_unreachable();
        self.enter_unreachable();
    }

    fn call_indirect(&mut self, ty_index: u32, table_index: u8) {}

    fn nop(&mut self) {}

    fn drop(&mut self) {
        self.pop();
    }

    fn select(&mut self) {
        let cond = self.pop();
        let cond_bool = self.ctx.coerce_i32_to_bool(self.builder, cond);
        let false_value = self.pop();
        let true_value = self.pop();
        self.push(
            self.builder
                .create_select(cond_bool, true_value, false_value),
        );
    }
}
