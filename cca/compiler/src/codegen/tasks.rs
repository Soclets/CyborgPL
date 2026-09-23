//! `concur` (`design/concurrency.md`): starting a task, and the end of the
//! block that waits for it.
//!
//! A task's body is already its own MIR function whose parameters are its
//! captures. Starting one is a call handed to the scheduler instead of made:
//! the captures are packed into a small block on the heap — each taken with
//! the verb the writer wrote — and a generated *start* function unpacks
//! them on the task's own stack, calls the body, and leaves its outcome
//! where the block that waits for it will look.
//!
//! The scheduler is the runtime's (`runtime/src/tasks.rs`): green tasks on
//! one worker thread per core, as the design says, so a thousand waiting
//! tasks do not cost a thousand threads.

use super::*;

/// Room for one group in a frame; the runtime asserts its own `Group`
/// fits (`GROUP_BYTES` in `runtime/src/tasks.rs`).
const GROUP_BYTES: u32 = 64;

impl<'a, 'ctx> Cg<'a, 'ctx> {
    /// Does anything in this program start a task? Only then does MAIN run
    /// on the scheduler: a program with no `concur` starts no threads.
    pub(super) fn uses_tasks(&self) -> bool {
        // A channel too: a `take` waits, and only a task can wait.
        self.p.functions.iter().any(|f| {
            f.blocks.iter().any(|b| b.stmts.iter().any(|s| matches!(s.kind, StmtKind::Task { .. }) || matches!(&s.kind, StmtKind::Assign { rvalue: Rvalue::MakeChannel { .. }, .. })))
        })
    }

    fn rt_task(&mut self, name: &str) -> FunctionValue<'ctx> {
        if let Some(f) = self.module.get_function(name) {
            return f;
        }
        let p = self.ctx.ptr_type(AddressSpace::default());
        let v = self.ctx.void_type();
        let t = match name {
            "cyborg_spawn" => v.fn_type(&[p.into(), p.into(), p.into()], false),
            "cyborg_group_init" | "cyborg_group_wait" | "cyborg_run" | "cyborg_free" => v.fn_type(&[p.into()], false),
            other => unreachable!("no runtime function named {}", other),
        };
        self.module.add_function(name, t, None)
    }

    /// In the entry block of a function that starts tasks: a group for
    /// each block that starts any, and a place for each task that can fail
    /// to leave its outcome.
    pub(super) fn task_frame(&mut self, fi: usize) {
        self.cur_groups.clear();
        self.cur_outcomes.clear();
        let f = &self.p.functions[fi];
        let mut groups = 0u32;
        let mut failing: Vec<(u32, usize)> = Vec::new();
        for b in &f.blocks {
            for s in &b.stmts {
                if let StmtKind::Task { detached, body, group, id, .. } = &s.kind {
                    if !*detached {
                        groups = groups.max(group + 1);
                    }
                    if !self.p.functions[body.0 as usize].fails.is_empty() {
                        failing.push((*id, body.0 as usize));
                    }
                }
            }
        }
        if groups == 0 && failing.is_empty() {
            return;
        }
        let room = self.ctx.i8_type().array_type(GROUP_BYTES);
        let init = self.rt_task("cyborg_group_init");
        for _ in 0..groups {
            let g = self.builder.build_alloca(room, "group").unwrap();
            if let Some(i) = g.as_instruction() {
                let _ = i.set_alignment(16);
            }
            let _ = self.builder.build_call(init, &[g.into()], "");
            self.cur_groups.push(g);
        }
        for (id, body) in failing {
            if let Some(shape) = self.fail_shape(body) {
                let slot = self.builder.build_alloca(shape, "outcome").unwrap();
                self.cur_outcomes.insert(id, slot);
            }
        }
    }

    /// `concur`: pack the captures and hand the task to the scheduler.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_task(&mut self, fi: usize, detached: bool, body: FnIdx, captures: &[(crate::ast::Verb, Place)], group: u32, id: u32, slots: &[Option<PointerValue<'ctx>>], span: Option<crate::source::Span>) -> Option<()> {
        let bi = body.0 as usize;
        let body_fn = *self.fns.get(&bi)?;
        if detached && !self.p.functions[bi].fails.is_empty() {
            // A detached task has no block waiting for it, so its failure
            // has nowhere to land. The design does not say where it goes.
            self.cannot("a `concur.detached` task that can fail: nothing waits for it, so its failure has nowhere to land", span);
            return None;
        }
        let pack = self.task_pack_shape(bi)?;
        let start = self.task_start_fn(fi, id, bi, body_fn, pack)?;
        let w = self.ctx.i64_type();
        let size = self.target.get_store_size(&pack);
        let block = self.alloc(w.const_int(size, false));
        let outcome = match self.cur_outcomes.get(&id) {
            Some(p) => *p,
            None => self.ctx.ptr_type(AddressSpace::default()).const_null(),
        };
        let at = self.builder.build_struct_gep(pack, block, 0, "outcome").ok()?;
        let _ = self.builder.build_store(at, outcome);
        let params = self.p.functions[bi].params.clone();
        for (k, ((verb, place), param)) in captures.iter().zip(params.iter()).enumerate() {
            let lent = matches!(self.p.functions[bi].locals[param.0 as usize].kind, LocalKind::Lent { .. });
            let v: BasicValueEnum<'ctx> = match verb {
                // A lend is an address; the block that started the task
                // outlives it, which is what makes the address safe.
                crate::ast::Verb::Lend | crate::ast::Verb::LendMut => self.slot_of(fi, place, slots)?.0.into(),
                crate::ast::Verb::Give => self.operand(fi, &Operand::Move(place.clone()), slots)?,
                crate::ast::Verb::Copy => {
                    let (src, ty) = self.slot_of(fi, place, slots)?;
                    match self.copy_fn(&ty) {
                        Some(copier) => {
                            let shape = self.ty(&ty)?;
                            let tmp = self.builder.build_alloca(shape, "copy").ok()?;
                            let _ = self.builder.build_call(copier, &[tmp.into(), src.into()], "");
                            self.builder.build_load(shape, tmp, "copied").ok()?
                        }
                        None => self.operand(fi, &Operand::Read(place.clone()), slots)?,
                    }
                }
            };
            let _ = lent;
            let field = self.builder.build_struct_gep(pack, block, k as u32 + 1, "cap").ok()?;
            let want = pack.get_field_type_at_index(k as u32 + 1)?;
            let v = self.fit(v, want);
            let _ = self.builder.build_store(field, v);
        }
        // Each channel lent to the task: the task can reach it until it
        // ends, which is what `closed` is counted from.
        for (verb, place) in captures {
            if matches!(verb, crate::ast::Verb::Lend | crate::ast::Verb::LendMut) {
                if let Some((ch, _)) = self.chan_of(fi, &Operand::Read(place.clone()), slots) {
                    let hold = self.rt_chan("cyborg_chan_hold");
                    let _ = self.builder.build_call(hold, &[ch.into()], "");
                }
            }
        }
        let group_ptr = if detached {
            self.ctx.ptr_type(AddressSpace::default()).const_null()
        } else {
            *self.cur_groups.get(group as usize)?
        };
        let spawn = self.rt_task("cyborg_spawn");
        let _ = self.builder.build_call(spawn, &[group_ptr.into(), start.as_global_value().as_pointer_value().into(), block.into()], "");
        Some(())
    }

    /// What travels with a task: where its outcome goes, then each capture
    /// exactly as the body takes it.
    fn task_pack_shape(&mut self, bi: usize) -> Option<inkwell::types::StructType<'ctx>> {
        let body_fn = *self.fns.get(&bi)?;
        let mut fields: Vec<BasicTypeEnum<'ctx>> = vec![self.ctx.ptr_type(AddressSpace::default()).into()];
        for t in body_fn.get_type().get_param_types() {
            fields.push(BasicTypeEnum::try_from(t).ok()?);
        }
        Some(self.ctx.struct_type(&fields, false))
    }

    /// `cy_start_F_N(pack)`: runs on the task's own stack. Unpacks the
    /// captures, calls the body, leaves the outcome where the waiting block
    /// will read it, and gives the pack back.
    fn task_start_fn(&mut self, fi: usize, id: u32, bi: usize, body_fn: FunctionValue<'ctx>, pack: inkwell::types::StructType<'ctx>) -> Option<FunctionValue<'ctx>> {
        let name = format!("cy_start_{}_{}", fi, id);
        if let Some(f) = self.module.get_function(&name) {
            return Some(f);
        }
        let p = self.ctx.ptr_type(AddressSpace::default());
        let func = self.module.add_function(&name, self.ctx.void_type().fn_type(&[p.into()], false), Some(Linkage::Internal));
        let here = self.builder.get_insert_block();
        let entry = self.ctx.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);
        let block = func.get_nth_param(0)?.into_pointer_value();
        let mut args: Vec<BasicMetadataValueEnum<'ctx>> = Vec::new();
        for k in 0..body_fn.count_params() {
            let t = pack.get_field_type_at_index(k + 1)?;
            let at = self.builder.build_struct_gep(pack, block, k + 1, "cap").ok()?;
            args.push(self.builder.build_load(t, at, "c").ok()?.into());
        }
        let call = self.builder.build_call(body_fn, &args, "ran").ok()?;
        // The task has ended: it can reach its lent channels no longer.
        let params = self.p.functions[bi].params.clone();
        for (k, prm) in params.iter().enumerate() {
            let local = &self.p.functions[bi].locals[prm.0 as usize];
            if matches!(local.kind, LocalKind::Lent { .. }) && Self::chan_elem(&local.ty).is_some() {
                if let Some(BasicMetadataValueEnum::PointerValue(at)) = args.get(k).copied() {
                    let ch = self.builder.build_load(p, at, "ch").ok()?.into_pointer_value();
                    let release = self.rt_chan("cyborg_chan_release");
                    let _ = self.builder.build_call(release, &[ch.into()], "");
                }
            }
        }
        if !self.p.functions[bi].fails.is_empty() {
            if let Some(v) = basic_result(call) {
                let at = self.builder.build_struct_gep(pack, block, 0, "o").ok()?;
                let out = self.builder.build_load(p, at, "outcome").ok()?.into_pointer_value();
                let _ = self.builder.build_store(out, v);
            }
        }
        let free = self.rt_task("cyborg_free");
        let _ = self.builder.build_call(free, &[block.into()], "");
        let _ = self.builder.build_return(None);
        if let Some(b) = here {
            self.builder.position_at_end(b);
        }
        Some(func)
    }

    /// The end of a block that started tasks: wait for all of them.
    pub(super) fn emit_join(&mut self, group: u32) -> Option<()> {
        let g = *self.cur_groups.get(group as usize)?;
        let wait = self.rt_task("cyborg_group_wait");
        let _ = self.builder.build_call(wait, &[g.into()], "");
        Some(())
    }

    /// After the wait: how did task `id` end? Its answer is nothing; its
    /// reasons go to the arms its `otherwise` wrote.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_task_outcome(&mut self, fi: usize, id: u32, body: FnIdx, ok: BlockId, fail: &[FailArm], blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        let bi = body.0 as usize;
        let slot = *self.cur_outcomes.get(&id)?;
        let shape = self.fail_shape(bi)?;
        let cases = self.p.functions[bi].fails.len() + 1;
        let tag_ty = self.enum_tag_type(cases);
        let p = self.builder.build_struct_gep(shape, slot, 0, "which").ok()?;
        let tag = self.builder.build_load(tag_ty, p, "tag").ok()?.into_int_value();
        let mut arms: Vec<(IntValue<'ctx>, LlBlock<'ctx>)> = vec![(tag_ty.const_zero(), blocks[ok.0 as usize])];
        let mut landings = Vec::new();
        for arm in fail {
            let land = self.ctx.append_basic_block(func, "task_failed");
            arms.push((tag_ty.const_int(arm.index as u64 + 1, false), land));
            landings.push((arm.clone(), land));
        }
        let unreached = self.ctx.append_basic_block(func, "no_such_outcome");
        let _ = self.builder.build_switch(tag, unreached, &arms);
        self.builder.position_at_end(unreached);
        let _ = self.builder.build_unreachable();
        for (arm, land) in landings {
            self.builder.position_at_end(land);
            if let Some(place) = &arm.payload {
                let carried = self.p.functions[bi].fails.get(arm.index).and_then(|(_, t)| t.clone());
                if let Some(ct) = carried.and_then(|t| self.ty(&t)) {
                    if let Ok(at) = self.builder.build_struct_gep(shape, slot, 1, "carried") {
                        if let Ok(v) = self.builder.build_load(ct, at, "why") {
                            self.store_place(fi, place, v, slots);
                        }
                    }
                }
            }
            let _ = self.builder.build_unconditional_branch(blocks[arm.block.0 as usize]);
        }
        Some(())
    }
}

// ------------------------------------------------------------------ channels
//
// A channel is one block the runtime owns — bookkeeping, then its slots
// inline (owner, 2026-09-23) — and the value a program holds is a pointer
// to it. What the compiler adds is the element's size, and what the tasks
// that can reach it are: each task a channel is lent to holds it for as
// long as the task runs, which is what `closed` is counted from.

impl<'a, 'ctx> Cg<'a, 'ctx> {
    fn rt_chan(&mut self, name: &str) -> FunctionValue<'ctx> {
        if let Some(f) = self.module.get_function(name) {
            return f;
        }
        let p = self.ctx.ptr_type(AddressSpace::default());
        let w = self.ctx.i64_type();
        let v = self.ctx.void_type();
        let b = self.ctx.bool_type();
        let t = match name {
            "cyborg_chan_new" => p.fn_type(&[w.into(), w.into()], false),
            "cyborg_chan_hold" | "cyborg_chan_release" => v.fn_type(&[p.into()], false),
            "cyborg_chan_drop" | "cyborg_chan_send" => v.fn_type(&[p.into(), p.into()], false),
            "cyborg_chan_take" => b.fn_type(&[p.into(), p.into()], false),
            "cyborg_whichever" => w.fn_type(&[w.into(), p.into(), p.into()], false),
            other => unreachable!("no runtime function named {}", other),
        };
        self.module.add_function(name, t, None)
    }

    pub(super) fn chan_elem(t: &Ty) -> Option<Ty> {
        match t.base() {
            Ty::Channel(e) => Some((**e).clone()),
            _ => None,
        }
    }

    /// `channel[T]` — a hand-off, no slots — or `channel[T][0, N]`.
    pub(super) fn make_channel(&mut self, elem: &Ty, capacity: Option<(u64, u64)>) -> Option<BasicValueEnum<'ctx>> {
        let size = self.size_of_ty(elem)?;
        let cap = capacity.map(|(_, hi)| hi).unwrap_or(0);
        let w = self.ctx.i64_type();
        let f = self.rt_chan("cyborg_chan_new");
        basic_result(self.builder.build_call(f, &[w.const_int(cap, false).into(), w.const_int(size, false).into()], "chan").ok()?)
    }

    /// The channel a place holds.
    fn chan_of(&mut self, fi: usize, o: &Operand, slots: &[Option<PointerValue<'ctx>>]) -> Option<(PointerValue<'ctx>, Ty)> {
        let place = o.place()?.clone();
        let (at, ty) = self.slot_of(fi, &place, slots)?;
        let elem = Self::chan_elem(&ty)?;
        let ch = self.builder.build_load(self.ctx.ptr_type(AddressSpace::default()), at, "ch").ok()?.into_pointer_value();
        Some((ch, elem))
    }

    /// `channel.send[lend 'c', v]` — the value is given: it is the
    /// channel's (and then a taker's) from here on.
    pub(super) fn chan_send(&mut self, fi: usize, args: &[Operand], slots: &[Option<PointerValue<'ctx>>]) -> Option<()> {
        let (ch, elem) = self.chan_of(fi, args.first()?, slots)?;
        let et = self.ty(&elem)?;
        let v = self.operand(fi, args.get(1)?, slots)?;
        let signed = self.operand_signed(fi, args.get(1)?).unwrap_or(true);
        let v = self.fit_as(v, et, signed);
        let tmp = self.builder.build_alloca(et, "sent").ok()?;
        let _ = self.builder.build_store(tmp, v);
        let f = self.rt_chan("cyborg_chan_send");
        let _ = self.builder.build_call(f, &[ch.into(), tmp.into()], "");
        Some(())
    }

    /// `channel.take[lend 'c'] otherwise …` — the value, or `closed`: the
    /// only reason a take gives.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn chan_take(&mut self, fi: usize, args: &[Arg], dest: &Option<Place>, ok: BlockId, fail: &[FailArm], blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        let place = match args.first()? {
            Arg::Lend { place, .. } | Arg::Copy(place) | Arg::Give(place) => place.clone(),
            Arg::Value(o) => o.place()?.clone(),
            _ => return None,
        };
        let (ch, elem) = self.chan_of(fi, &Operand::Read(place), slots)?;
        let et = self.ty(&elem)?;
        let tmp = self.builder.build_alloca(et, "taken").ok()?;
        let f = self.rt_chan("cyborg_chan_take");
        let got = basic_result(self.builder.build_call(f, &[ch.into(), tmp.into()], "got").ok()?)?.into_int_value();
        let yes = self.ctx.append_basic_block(func, "took");
        let closed = match fail.first() {
            Some(arm) => blocks[arm.block.0 as usize],
            None => return None,
        };
        let _ = self.builder.build_conditional_branch(got, yes, closed);
        self.builder.position_at_end(yes);
        if let Some(d) = dest {
            let v = self.builder.build_load(et, tmp, "value").ok()?;
            self.store_place(fi, d, v, slots);
            if d.proj.is_empty() && self.owns_memory(&elem) {
                self.mark_alive(d.local, true);
            }
        }
        let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
        Some(())
    }

    /// `whichever` — wait on every arm's channel at once; the runtime says
    /// which one gave a value, or that all of them are closed.
    pub(super) fn emit_whichever(&mut self, fi: usize, arms: &[WhicheverArm], closed: Option<BlockId>, blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        let p = self.ctx.ptr_type(AddressSpace::default());
        let w = self.ctx.i64_type();
        let n = arms.len() as u32;
        let chans = self.builder.build_alloca(p.array_type(n), "chans").ok()?;
        let dests = self.builder.build_alloca(p.array_type(n), "dests").ok()?;
        let mut bufs = Vec::new();
        for (i, arm) in arms.iter().enumerate() {
            let (ch, elem) = self.chan_of(fi, &arm.channel, slots)?;
            let et = self.ty(&elem)?;
            let buf = self.builder.build_alloca(et, "arm_value").ok()?;
            let at = unsafe { self.builder.build_gep(p.array_type(n), chans, &[w.const_zero(), w.const_int(i as u64, false)], "c").ok()? };
            let _ = self.builder.build_store(at, ch);
            let at = unsafe { self.builder.build_gep(p.array_type(n), dests, &[w.const_zero(), w.const_int(i as u64, false)], "d").ok()? };
            let _ = self.builder.build_store(at, buf);
            bufs.push((buf, et, elem));
        }
        let f = self.rt_chan("cyborg_whichever");
        let which = basic_result(self.builder.build_call(f, &[w.const_int(n as u64, false).into(), chans.into(), dests.into()], "which").ok()?)?.into_int_value();
        let none = match closed {
            Some(b) => blocks[b.0 as usize],
            None => {
                // No `otherwise`: every arm closed at once cannot happen
                // here, the checker having asked for one where it can.
                let u = self.ctx.append_basic_block(func, "all_closed");
                let here = self.builder.get_insert_block();
                self.builder.position_at_end(u);
                let _ = self.builder.build_unreachable();
                if let Some(h) = here {
                    self.builder.position_at_end(h);
                }
                u
            }
        };
        let mut cases = Vec::new();
        for (i, arm) in arms.iter().enumerate() {
            let land = self.ctx.append_basic_block(func, "arm");
            cases.push((w.const_int(i as u64, false), land));
            let here = self.builder.get_insert_block();
            self.builder.position_at_end(land);
            let (buf, et, elem) = bufs[i].clone();
            let v = self.builder.build_load(et, buf, "v").ok()?;
            self.store_place(fi, &arm.dest, v, slots);
            if arm.dest.proj.is_empty() && self.owns_memory(&elem) {
                self.mark_alive(arm.dest.local, true);
            }
            let _ = self.builder.build_unconditional_branch(blocks[arm.block.0 as usize]);
            if let Some(h) = here {
                self.builder.position_at_end(h);
            }
        }
        let _ = self.builder.build_switch(which, none, &cases);
        Some(())
    }
}

impl<'a, 'ctx> Cg<'a, 'ctx> {
    pub(super) fn rt_chan_drop(&mut self) -> FunctionValue<'ctx> {
        self.rt_chan("cyborg_chan_drop")
    }
}

