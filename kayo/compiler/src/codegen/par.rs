//! Parallel `for` in the generated code (`parallelism.md`, `crate::par`).
//!
//! A planned loop keeps its ordinary code, and where it is entered a test
//! is added: when its trips times the work of one (`par::WORTH`) say the
//! threads pay, the runtime runs it in slices instead (`cyborg_par_for`).
//! A slice is a function of its own, made of the loop's very blocks — the
//! same statements, the same sites and what Precog proved of them — with
//! its own counter, stop and private locals, and every other local read in
//! the parent's frame through an address it is handed. A fold sums its own
//! trips in 128 bits from 0; the parent adds the partials to what the
//! variable held and checks, once, that the total fits.

use std::collections::{HashMap, HashSet};

use inkwell::basic_block::BasicBlock as LlBlock;
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::{AddressSpace, IntPredicate};

use super::Cg;
use crate::mir::*;
use crate::ast::BinOp;
use crate::par::{Fold, FoldKind, ParLoop, MAX_SLICES, WORTH};

/// What the slice being written leaves out, and where its folds go.
pub(super) struct ParChild<'ctx> {
    pub skip: HashSet<(usize, usize)>,
    /// (block, statement) of a fold's add: which fold, and the term added.
    pub adds: HashMap<(usize, usize), (usize, Operand)>,
    /// Each fold's partial, in the slice's frame, and for a product of
    /// whole numbers whether it went past 128 bits.
    pub partials: Vec<PointerValue<'ctx>>,
    pub flags: Vec<Option<PointerValue<'ctx>>>,
    pub folds: Vec<Fold>,
    /// A loop under `no_cheat`: each round's term goes into the slice's sink
    /// (`super::keep`), or, with none, the partial through a barrier.
    pub nocheat: bool,
}

impl<'a, 'ctx> Cg<'a, 'ctx> {
    /// The locals a slice reads in the parent's frame, in the order the
    /// parent hands their addresses: every one the loop names that the
    /// slice does not keep its own, then the counter's and the step's
    /// (from which a slice works out its first value).
    fn par_shared(&self, fi: usize, pl: &ParLoop) -> Vec<u32> {
        let f = &self.p.functions[fi];
        let mut named: HashSet<u32> = HashSet::new();
        let note = |o: &Operand, n: &mut HashSet<u32>| {
            if let Some(p) = o.place() {
                n.insert(p.local.0);
                for pr in &p.proj {
                    if let Proj::Index(Operand::Read(ix) | Operand::Move(ix)) = pr {
                        n.insert(ix.local.0);
                    }
                }
            }
        };
        for &b in &pl.blocks {
            let blk = &f.blocks[b];
            for s in &blk.stmts {
                let mut ops: Vec<Operand> = Vec::new();
                match &s.kind {
                    StmtKind::Assign { place, rvalue } => {
                        ops.push(Operand::Read(place.clone()));
                        collect_rvalue(rvalue, &mut ops);
                    }
                    StmtKind::Op { dest, args, .. } => {
                        ops.extend(args.iter().cloned());
                        if let Some(d) = dest {
                            ops.push(Operand::Read(d.clone()));
                        }
                    }
                    StmtKind::Drop(p) | StmtKind::EndLend(p) => ops.push(Operand::Read(p.clone())),
                    StmtKind::Site(id) => ops.extend(f.site(*id).operands.iter().cloned()),
                    StmtKind::Asm { ins, outs, lends, .. } => {
                        ops.extend(ins.iter().map(|(p, _)| Operand::Read(p.clone())));
                        ops.extend(outs.iter().map(|(p, _)| Operand::Read(p.clone())));
                        ops.extend(lends.iter().map(|(p, _, _)| Operand::Read(p.clone())));
                    }
                    _ => {}
                }
                for o in &ops {
                    note(o, &mut named);
                }
            }
            match &blk.term {
                Terminator::If { cond, .. } => note(cond, &mut named),
                Terminator::Switch { value, .. } => note(value, &mut named),
                Terminator::Call { callee, args, dest, fail, .. } => {
                    if let Callee::Value(o) = callee {
                        note(o, &mut named);
                    }
                    for a in args {
                        let o = match a {
                            Arg::Value(o) => o.clone(),
                            Arg::Lend { place, .. } | Arg::Give(place) | Arg::Copy(place) | Arg::Spread { place, .. } => Operand::Read(place.clone()),
                        };
                        note(&o, &mut named);
                    }
                    if let Some(d) = dest {
                        note(&Operand::Read(d.clone()), &mut named);
                    }
                    for arm in fail {
                        if let Some(p) = &arm.payload {
                            note(&Operand::Read(p.clone()), &mut named);
                        }
                    }
                }
                _ => {}
            }
        }
        let mut shared: Vec<u32> = named.into_iter().filter(|l| !pl.private.contains(l)).collect();
        shared.sort();
        shared.retain(|l| *l != pl.fl.counter.0);
        shared.push(pl.fl.counter.0);
        if let Some(s) = pl.fl.step {
            shared.retain(|l| *l != s.0);
            shared.push(s.0);
        }
        shared
    }

    /// Where a planned loop is entered: slices when they pay, the ordinary
    /// loop otherwise. Ends the block.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn par_dispatch(&mut self, fi: usize, pl: &ParLoop, blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        let f = &self.p.functions[fi];
        let i64t = self.ctx.i64_type();
        let w = self.ctx.i128_type();
        let head = blocks[pl.fl.head.0 as usize];
        let exit = blocks[pl.fl.exit.0 as usize];
        let (cbits, csigned) = crate::precog::smt::int_shape(&f.locals[pl.fl.counter.0 as usize].ty)?;
        let _ = cbits;
        let load_wide = |this: &mut Self, l: LocalId, signed: bool| -> Option<IntValue<'ctx>> {
            let slot = slots[l.0 as usize]?;
            let ty = this.ty(&this.p.functions[fi].locals[l.0 as usize].ty.clone())?.into_int_type();
            let v = this.builder.build_load(ty, slot, "v").ok()?.into_int_value();
            this.builder.build_int_cast_sign_flag(v, w, signed, "wide").ok()
        };
        // Whether slices pay: trips times the work of one at least `WORTH`,
        // and at least two trips. A range's trips are its span over its step
        // plus one, so that is a span of at least `least` steps, asked with a
        // multiply: the division waits until the loop is sliced. It was asked
        // on every entry, once per prime in bench/sieve, mostly for loops of
        // no trips at all, and was the sieve's gap to C (2026-10-15).
        let least = if pl.work == 0 { None } else { Some(WORTH.div_ceil(pl.work).saturating_sub(1).max(1)) };
        let (go, range) = match pl.fl.step {
            Some(step_l) => {
                let c0 = load_wide(self, pl.fl.counter, csigned)?;
                let (_, ssigned) = crate::precog::smt::int_shape(&f.locals[pl.fl.stop.0 as usize].ty)?;
                let stop = load_wide(self, pl.fl.stop, ssigned)?;
                let step = load_wide(self, step_l, true)?;
                let zero = w.const_zero();
                let up = self.builder.build_int_compare(IntPredicate::SGT, step, zero, "up").ok()?;
                let span_up = self.builder.build_int_sub(stop, c0, "span").ok()?;
                let span_down = self.builder.build_int_sub(c0, stop, "span").ok()?;
                let mag = self.builder.build_select(up, step, self.builder.build_int_neg(step, "neg").ok()?, "mag").ok()?.into_int_value();
                let span = self.builder.build_select(up, span_up, span_down, "span").ok()?.into_int_value();
                let go = match least {
                    // A step's size is below 2^64 and `least` below `WORTH`,
                    // so the product holds in 128 bits.
                    Some(k) => {
                        let need = self.builder.build_int_mul(mag, w.const_int(k, false), "need").ok()?;
                        let enough = self.builder.build_int_compare(IntPredicate::SGE, span, need, "pays").ok()?;
                        let moving = self.builder.build_int_compare(IntPredicate::NE, step, zero, "moving").ok()?;
                        self.builder.build_and(enough, moving, "go").ok()?
                    }
                    None => self.ctx.bool_type().const_zero(),
                };
                (go, Some((span, mag)))
            }
            None => {
                let trips = load_wide(self, pl.fl.stop, false)?;
                let work = self.builder.build_int_mul(trips, w.const_int(pl.work, false), "work").ok()?;
                let pays = self.builder.build_int_compare(IntPredicate::UGE, work, w.const_int(WORTH, false), "pays").ok()?;
                let several = self.builder.build_int_compare(IntPredicate::SGE, trips, w.const_int(2, false), "several").ok()?;
                (self.builder.build_and(pays, several, "go").ok()?, None)
            }
        };
        let par_b = self.ctx.append_basic_block(func, "in_slices");
        let _ = self.builder.build_conditional_branch(go, par_b, head);
        self.builder.position_at_end(par_b);
        // How many trips, in 128 bits: a range's from its first value to its
        // stop by its step (a span of at least one step, here), a list's its
        // length.
        let trips = match range {
            Some((span, mag)) => {
                let q = self.builder.build_int_signed_div(span, mag, "q").ok()?;
                self.builder.build_int_add(q, w.const_int(1, false), "trips").ok()?
            }
            None => load_wide(self, pl.fl.stop, false)?,
        };

        let shared = self.par_shared(fi, pl);
        let child = self.par_child_fn(fi, pl, &shared)?;
        let p = self.ctx.ptr_type(AddressSpace::default());
        // The addresses: every shared local's, then each fold's partials
        // and overflow flags (one per slice).
        let nfolds = pl.folds.len();
        let env_ty = p.array_type((shared.len() + 2 * nfolds) as u32);
        let env = self.stack(env_ty, "slices_env").ok()?;
        let env_at = |this: &mut Self, k: usize| unsafe { this.builder.build_gep(env_ty, env, &[i64t.const_zero(), i64t.const_int(k as u64, false)], "env_at").ok() };
        // A copy of each, and of what a borrow among them points at: a loop
        // in slices never changes a shared local itself (only elements, through
        // the list's own memory), and handing over the originals' addresses
        // kept LLVM from holding them in registers anywhere in the function —
        // the list built just before stored its length on every append.
        for (k, l) in shared.iter().enumerate() {
            let at = env_at(self, k)?;
            let addr = match slots[*l as usize] {
                Some(slot) => self.par_snapshot(fi, *l, slot).unwrap_or(slot),
                None => p.const_null(),
            };
            let _ = self.builder.build_store(at, addr);
        }
        let flags_ty = self.ctx.i8_type().array_type(MAX_SLICES as u32);
        let mut partials = Vec::new();
        let mut flag_arrays = Vec::new();
        for (r, fd) in pl.folds.iter().enumerate() {
            let pt = self.fold_ty(fi, fd)?;
            let arr = self.stack(pt.array_type(MAX_SLICES as u32), "partials").ok()?;
            let at = env_at(self, shared.len() + 2 * r)?;
            let _ = self.builder.build_store(at, arr);
            let flags = self.stack(flags_ty, "overflowed").ok()?;
            let _ = self.builder.build_store(flags, flags_ty.const_zero());
            let at = env_at(self, shared.len() + 2 * r + 1)?;
            let _ = self.builder.build_store(at, flags);
            partials.push((arr, pt));
            flag_arrays.push(flags);
        }
        let run = self.rt("cyborg_par_for");
        let trips64 = self.builder.build_int_truncate(trips, i64t, "trips").ok()?;
        let used = super::basic_result(self.builder.build_call(run, &[child.as_global_value().as_pointer_value().into(), env.into(), trips64.into()], "slices").ok()?)?.into_int_value();
        // Each fold: what the variable held, folded with every slice's
        // partial in slice order, then checked once.
        for (r, fd) in pl.folds.iter().enumerate() {
            self.par_combine(fi, fd, partials[r], flag_arrays[r], used, slots, func)?;
        }
        let _ = self.builder.build_unconditional_branch(exit);
        Some(())
    }

    /// The slice: `void(env, first, end, slice)`, the loop's blocks with
    /// the iterations `first..end` of it.
    fn par_child_fn(&mut self, fi: usize, pl: &ParLoop, shared: &[u32]) -> Option<FunctionValue<'ctx>> {
        let f = &self.p.functions[fi];
        let p = self.ctx.ptr_type(AddressSpace::default());
        let i64t = self.ctx.i64_type();
        let w = self.ctx.i128_type();
        let fnty = self.ctx.void_type().fn_type(&[p.into(), i64t.into(), i64t.into(), i64t.into()], false);
        let name = format!("cy_slice_{}_{}", fi, pl.fl.pre.0);
        let child = self.module.add_function(&name, fnty, Some(inkwell::module::Linkage::Internal));
        // Where the parent was.
        let back = self.builder.get_insert_block();
        let saved_flags = std::mem::take(&mut self.cur_flags);
        let saved_here = self.here;
        let saved_sites = std::mem::take(&mut self.op_sites);
        let saved_child = self.par_child.take();
        let saved_sink = self.sink.take();

        let entry = self.ctx.append_basic_block(child, "entry");
        self.builder.position_at_end(entry);
        // A loop under `no_cheat` (`super::keep`): the slice's own sink.
        let keep = self.keep.get(&fi).is_some_and(|k| k.blocks.contains(&(pl.fl.head.0 as usize)));
        if keep {
            if let Ok(sink) = self.stack(i64t, "no_cheat_sink") {
                let _ = self.builder.build_store(sink, i64t.const_zero());
                self.sink = Some(sink);
            }
        }
        let env = child.get_nth_param(0)?.into_pointer_value();
        let first = child.get_nth_param(1)?.into_int_value();
        let end = child.get_nth_param(2)?.into_int_value();
        let slice = child.get_nth_param(3)?.into_int_value();
        let env_ty = p.array_type((shared.len() + 2 * pl.folds.len()) as u32);
        let mut slots: Vec<Option<PointerValue<'ctx>>> = vec![None; f.locals.len()];
        let mut flags: Vec<Option<PointerValue<'ctx>>> = vec![None; f.locals.len()];
        for (li, l) in f.locals.iter().enumerate() {
            if !pl.private.contains(&(li as u32)) {
                continue;
            }
            let lent = matches!(l.kind, LocalKind::Lent { .. });
            let t: BasicTypeEnum = if lent { p.into() } else { self.ty(&l.ty.clone())? };
            slots[li] = Some(self.stack(t, "own").ok()?);
            if !lent && self.owns_memory(&l.ty.clone()) {
                let flag = self.stack(self.ctx.bool_type(), "alive").ok()?;
                let _ = self.builder.build_store(flag, self.ctx.bool_type().const_zero());
                flags[li] = Some(flag);
            }
        }
        let mut parent_counter = None;
        let mut parent_step = None;
        for (k, l) in shared.iter().enumerate() {
            let at = unsafe { self.builder.build_gep(env_ty, env, &[i64t.const_zero(), i64t.const_int(k as u64, false)], "env_at").ok()? };
            let addr = self.builder.build_load(p, at, "addr").ok()?.into_pointer_value();
            // The slice's own copy: read through the parent's addresses, a
            // store into an element might be one of them, so the list's
            // memory and the step were read again every round (the sieve
            // at half speed). A loop in slices never changes a shared local.
            let own = self.par_snapshot(fi, *l, addr).unwrap_or(addr);
            if *l == pl.fl.counter.0 {
                parent_counter = Some(addr);
            } else if Some(LocalId(*l)) == pl.fl.step {
                parent_step = Some(own);
            } else {
                slots[*l as usize] = Some(own);
            }
        }
        // The step, read from the parent's frame when the loop never
        // writes it (it never does).
        if let (Some(s), Some(addr)) = (pl.fl.step, parent_step) {
            slots[s.0 as usize] = Some(addr);
            // A step written as a number is that number here too: read from
            // the parent, `by 1` was a step LLVM could not know, and no
            // slice of any loop was vectorised.
            if let Some(v) = super::step_const(f, s) {
                let st = self.ty(&f.locals[s.0 as usize].ty.clone())?.into_int_type();
                let own = self.stack(st, "step").ok()?;
                let _ = self.builder.build_store(own, st.const_int(v as u64, true));
                slots[s.0 as usize] = Some(own);
            }
        }
        // The slice's counter and stop: its first and last iteration.
        let cty_c = f.locals[pl.fl.counter.0 as usize].ty.clone();
        let (_, csigned) = crate::precog::smt::int_shape(&cty_c)?;
        let ct = self.ty(&cty_c)?.into_int_type();
        let first_w = self.builder.build_int_s_extend(first, w, "first").ok()?;
        let last_w = self.builder.build_int_sub(self.builder.build_int_s_extend(end, w, "end").ok()?, w.const_int(1, false), "last").ok()?;
        let (c_first, c_last) = match pl.fl.step {
            Some(step_l) => {
                let c0 = self.builder.build_load(ct, parent_counter?, "c0").ok()?.into_int_value();
                let c0 = self.builder.build_int_cast_sign_flag(c0, w, csigned, "c0").ok()?;
                let st = self.ty(&f.locals[step_l.0 as usize].ty.clone())?.into_int_type();
                let step = self.builder.build_load(st, slots[step_l.0 as usize]?, "step").ok()?.into_int_value();
                let step = self.builder.build_int_s_extend(step, w, "step").ok()?;
                let a = self.builder.build_int_add(c0, self.builder.build_int_mul(first_w, step, "k").ok()?, "c_first").ok()?;
                let b = self.builder.build_int_add(c0, self.builder.build_int_mul(last_w, step, "k").ok()?, "c_last").ok()?;
                (a, b)
            }
            None => (self.builder.build_int_add(first_w, w.const_int(1, false), "i_first").ok()?, self.builder.build_int_add(last_w, w.const_int(1, false), "i_last").ok()?),
        };
        let _ = self.builder.build_store(slots[pl.fl.counter.0 as usize]?, self.builder.build_int_truncate(c_first, ct, "c").ok()?);
        let stop_ty = self.ty(&f.locals[pl.fl.stop.0 as usize].ty.clone())?.into_int_type();
        let _ = self.builder.build_store(slots[pl.fl.stop.0 as usize]?, self.builder.build_int_truncate(c_last, stop_ty, "stop").ok()?);
        // The folds' partials, from nothing: 0 for a sum, 1 for a product.
        let mut partials: Vec<PointerValue<'ctx>> = Vec::new();
        let mut flags_own: Vec<Option<PointerValue<'ctx>>> = Vec::new();
        for fd in &pl.folds {
            let t = self.fold_ty(fi, fd)?;
            let part = self.stack(t, "partial").ok()?;
            let start = self.fold_identity(fi, fd)?;
            let _ = self.builder.build_store(part, start);
            partials.push(part);
            if fd.kind == FoldKind::Int && fd.op == BinOp::Mul && !fd.wrap {
                let flag = self.stack(self.ctx.bool_type(), "overflowed").ok()?;
                let _ = self.builder.build_store(flag, self.ctx.bool_type().const_zero());
                flags_own.push(Some(flag));
            } else {
                flags_own.push(None);
            }
        }
        let mut skip = HashSet::new();
        let mut adds = HashMap::new();
        for (r, fd) in pl.folds.iter().enumerate() {
            skip.extend(fd.skip.iter().copied());
            for (b, si, x) in &fd.adds {
                adds.insert((*b, *si), (r, x.clone()));
            }
        }
        self.par_child = Some(ParChild { skip, adds, partials: partials.clone(), flags: flags_own.clone(), folds: pl.folds.clone(), nocheat: pl.fl.nocheat || keep });
        self.cur_flags = flags;

        let done = self.ctx.append_basic_block(child, "done");
        let nowhere = self.ctx.append_basic_block(child, "not_in_the_loop");
        let inside: HashSet<usize> = pl.blocks.iter().copied().collect();
        let blocks: Vec<LlBlock<'ctx>> = (0..f.blocks.len())
            .map(|b| {
                if b == pl.fl.exit.0 as usize {
                    done
                } else if inside.contains(&b) {
                    self.ctx.append_basic_block(child, &format!("b{}", b))
                } else {
                    nowhere
                }
            })
            .collect();
        let _ = self.builder.build_unconditional_branch(blocks[pl.fl.head.0 as usize]);
        for &b in &pl.blocks {
            self.builder.position_at_end(blocks[b]);
            self.block(fi, b, &blocks, &slots, child);
        }
        self.builder.position_at_end(nowhere);
        let _ = self.builder.build_unreachable();
        self.builder.position_at_end(done);
        if let Some(sink) = self.sink {
            self.barrier_on(sink, i64t.into());
        }
        for (r, part) in partials.iter().enumerate() {
            let t = self.fold_ty(fi, &pl.folds[r])?;
            let parts_ty = t.array_type(MAX_SLICES as u32);
            let at = unsafe { self.builder.build_gep(env_ty, env, &[i64t.const_zero(), i64t.const_int((shared.len() + 2 * r) as u64, false)], "env_at").ok()? };
            let arr = self.builder.build_load(p, at, "partials").ok()?.into_pointer_value();
            let slot = unsafe { self.builder.build_gep(parts_ty, arr, &[i64t.const_zero(), slice], "mine").ok()? };
            let v = self.builder.build_load(t, *part, "partial").ok()?;
            let _ = self.builder.build_store(slot, v);
            if let Some(flag) = flags_own[r] {
                let at = unsafe { self.builder.build_gep(env_ty, env, &[i64t.const_zero(), i64t.const_int((shared.len() + 2 * r + 1) as u64, false)], "env_at").ok()? };
                let arr = self.builder.build_load(p, at, "flags").ok()?.into_pointer_value();
                let ft = self.ctx.i8_type().array_type(MAX_SLICES as u32);
                let slot = unsafe { self.builder.build_gep(ft, arr, &[i64t.const_zero(), slice], "mine").ok()? };
                let b = self.builder.build_load(self.ctx.bool_type(), flag, "overflowed").ok()?.into_int_value();
                let b = self.builder.build_int_z_extend(b, self.ctx.i8_type(), "overflowed").ok()?;
                let _ = self.builder.build_store(slot, b);
            }
        }
        let _ = self.builder.build_return(None);

        self.cur_flags = saved_flags;
        self.here = saved_here;
        self.op_sites = saved_sites;
        self.par_child = saved_child;
        self.sink = saved_sink;
        if let Some(b) = back {
            self.builder.position_at_end(b);
        }
        Some(child)
    }

    /// A shared local's value in a slot of its own; for a borrow, a slot
    /// holding the address of a copy of what it borrows.
    fn par_snapshot(&mut self, fi: usize, l: u32, slot: PointerValue<'ctx>) -> Option<PointerValue<'ctx>> {
        let f = &self.p.functions[fi];
        let local = &f.locals[l as usize];
        let ty = local.ty.clone();
        // A lent parameter's slot is the address it borrows already; a
        // lent local's holds that address.
        let lent = matches!(local.kind, LocalKind::Lent { .. }) && !f.params.contains(&LocalId(l));
        let t = self.ty(&ty)?;
        if lent {
            let p = self.ctx.ptr_type(AddressSpace::default());
            let target = self.builder.build_load(p, slot, "borrowed").ok()?.into_pointer_value();
            let v = self.builder.build_load(t, target, "shared").ok()?;
            let copy = self.stack(t, "shared_copy").ok()?;
            let _ = self.builder.build_store(copy, v);
            let at = self.stack(p, "shared_borrow").ok()?;
            let _ = self.builder.build_store(at, copy);
            Some(at)
        } else {
            let v = self.builder.build_load(t, slot, "shared").ok()?;
            let copy = self.stack(t, "shared_copy").ok()?;
            let _ = self.builder.build_store(copy, v);
            Some(copy)
        }
    }

    /// In a slice: a fold's add (or multiply) goes to the slice's partial.
    pub(super) fn par_fold_add(&mut self, fi: usize, which: usize, x: &Operand, slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        let pc = self.par_child.as_ref()?;
        let part = pc.partials.get(which).copied()?;
        let flag = pc.flags.get(which).copied().flatten();
        let fd = pc.folds.get(which)?.clone();
        let t = self.fold_ty(fi, &fd)?;
        let xv = self.operand(fi, x, slots)?;
        let signed = self.operand_signed(fi, x).unwrap_or(true);
        let cur = self.builder.build_load(t, part, "partial").ok()?;
        let (next, over) = self.fold_step(fi, &fd, cur, xv, signed, func)?;
        let _ = self.builder.build_store(part, next);
        // Under `no_cheat` each round's term goes into the sink: the
        // partial still sums in vector lanes (`super::keep`).
        if self.par_child.as_ref().is_some_and(|pc| pc.nocheat) {
            if self.sink.is_some() {
                self.mix_value(xv);
            } else {
                self.barrier_on(part, t);
            }
        }
        if let (Some(flag), Some(o)) = (flag, over) {
            let was = self.builder.build_load(self.ctx.bool_type(), flag, "overflowed").ok()?.into_int_value();
            let now = self.builder.build_or(was, o, "overflowed").ok()?;
            let _ = self.builder.build_store(flag, now);
        }
        Some(())
    }

    /// What a fold's partial is held in: 128 bits for a whole number, the
    /// variable's own type for a bin or a deci.
    fn fold_ty(&mut self, fi: usize, fd: &Fold) -> Option<BasicTypeEnum<'ctx>> {
        match fd.kind {
            FoldKind::Int if !fd.wrap => Some(self.ctx.i128_type().into()),
            FoldKind::Int | FoldKind::Bin | FoldKind::Deci(_) => {
                let t = self.p.functions[fi].locals[fd.acc.0 as usize].ty.clone();
                self.ty(&t)
            }
        }
    }

    /// Where a partial starts: 0 for a sum, 1 for a product.
    fn fold_identity(&mut self, fi: usize, fd: &Fold) -> Option<BasicValueEnum<'ctx>> {
        let one = fd.op == BinOp::Mul;
        Some(match fd.kind {
            FoldKind::Int => self.fold_ty(fi, fd)?.into_int_type().const_int(one as u64, false).into(),
            FoldKind::Bin => match self.fold_ty(fi, fd)? {
                BasicTypeEnum::FloatType(t) => t.const_float(if one { 1.0 } else { 0.0 }).into(),
                _ => return None,
            },
            FoldKind::Deci(bits) => {
                let v = crate::deci::literal(if one { "1" } else { "0" }, bits).ok()?;
                let t = self.ctx.custom_width_int_type(std::num::NonZeroU32::new(bits as u32)?).ok()?;
                t.const_int_arbitrary_precision(&[v as u64, (v >> 64) as u64]).into()
            }
        })
    }

    /// One step of a fold: `acc op x`, and for a whole-number product
    /// whether it went past 128 bits.
    fn fold_step(&mut self, fi: usize, fd: &Fold, acc: BasicValueEnum<'ctx>, x: BasicValueEnum<'ctx>, x_signed: bool, func: FunctionValue<'ctx>) -> Option<(BasicValueEnum<'ctx>, Option<IntValue<'ctx>>)> {
        let w = self.ctx.i128_type();
        match fd.kind {
            FoldKind::Int if fd.wrap => {
                // Modulo 2^width: the proven total is what comes out.
                let a = acc.into_int_value();
                let b = self.builder.build_int_cast_sign_flag(x.into_int_value(), a.get_type(), x_signed, "term").ok()?;
                let v = if fd.op == BinOp::Mul { self.builder.build_int_mul(a, b, "product").ok()? } else { self.builder.build_int_add(a, b, "sum").ok()? };
                Some((v.into(), None))
            }
            FoldKind::Int => {
                let a = acc.into_int_value();
                let b = self.builder.build_int_cast_sign_flag(x.into_int_value(), w, x_signed, "term").ok()?;
                if fd.op == BinOp::Mul {
                    let f = inkwell::intrinsics::Intrinsic::find("llvm.smul.with.overflow")?.get_declaration(self.module, &[w.into()])?;
                    let r = super::basic_result(self.builder.build_call(f, &[a.into(), b.into()], "product").ok()?)?.into_struct_value();
                    let v = self.builder.build_extract_value(r, 0, "product").ok()?;
                    let o = self.builder.build_extract_value(r, 1, "past").ok()?.into_int_value();
                    Some((v, Some(o)))
                } else {
                    Some((self.builder.build_int_add(a, b, "sum").ok()?.into(), None))
                }
            }
            FoldKind::Bin => {
                let a = acc.into_float_value();
                let b = self.fit(x, a.get_type().into()).into_float_value();
                let v = if fd.op == BinOp::Mul { self.builder.build_float_mul(a, b, "product").ok()? } else { self.builder.build_float_add(a, b, "sum").ok()? };
                Some((v.into(), None))
            }
            FoldKind::Deci(bits) => Some((self.deci_binary(fi, &fd.op, acc, x, bits, false, func)?, None)),
        }
    }

    /// The parent's fold of the partials into the variable, in slice
    /// order, over the `used` slices, then the one check.
    #[allow(clippy::too_many_arguments)]
    fn par_combine(&mut self, fi: usize, fd: &Fold, (arr, pt): (PointerValue<'ctx>, BasicTypeEnum<'ctx>), flags: PointerValue<'ctx>, used: IntValue<'ctx>, slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        let f = &self.p.functions[fi];
        let i64t = self.ctx.i64_type();
        let w = self.ctx.i128_type();
        let boolt = self.ctx.bool_type();
        let acc_ty = f.locals[fd.acc.0 as usize].ty.clone();
        let slot = slots[fd.acc.0 as usize]?;
        let lt = self.ty(&acc_ty)?;
        let held = self.builder.build_load(lt, slot, "held").ok()?;
        let (start, signed) = match fd.kind {
            FoldKind::Int if fd.wrap => (held, true),
            FoldKind::Int => {
                let (_, signed) = crate::precog::smt::int_shape(&acc_ty)?;
                (self.builder.build_int_cast_sign_flag(held.into_int_value(), w, signed, "held").ok()?.into(), signed)
            }
            _ => (held, true),
        };
        let product = fd.kind == FoldKind::Int && fd.op == BinOp::Mul && !fd.wrap;
        let start_zero = match (product, start) {
            (true, BasicValueEnum::IntValue(v)) => self.builder.build_int_compare(IntPredicate::EQ, v, w.const_zero(), "zero").ok()?,
            _ => boolt.const_zero(),
        };
        // for k in 0..used
        let before = self.builder.get_insert_block()?;
        let head = self.ctx.append_basic_block(func, "fold");
        let body = self.ctx.append_basic_block(func, "fold_one");
        let done = self.ctx.append_basic_block(func, "folded");
        let _ = self.builder.build_unconditional_branch(head);
        self.builder.position_at_end(head);
        let k = self.builder.build_phi(i64t, "k").ok()?;
        let total = self.builder.build_phi(pt, "total").ok()?;
        let past = self.builder.build_phi(boolt, "past").ok()?;
        let zero = self.builder.build_phi(boolt, "zero").ok()?;
        k.add_incoming(&[(&i64t.const_zero(), before)]);
        total.add_incoming(&[(&start, before)]);
        past.add_incoming(&[(&boolt.const_zero(), before)]);
        zero.add_incoming(&[(&start_zero, before)]);
        let kv = k.as_basic_value().into_int_value();
        let more = self.builder.build_int_compare(IntPredicate::ULT, kv, used, "more").ok()?;
        let _ = self.builder.build_conditional_branch(more, body, done);
        self.builder.position_at_end(body);
        let at = unsafe { self.builder.build_gep(pt.array_type(MAX_SLICES as u32), arr, &[i64t.const_zero(), kv], "part").ok()? };
        let pv = self.builder.build_load(pt, at, "part").ok()?;
        let fa = unsafe { self.builder.build_gep(self.ctx.i8_type().array_type(MAX_SLICES as u32), flags, &[i64t.const_zero(), kv], "flag").ok()? };
        let fv = self.builder.build_load(self.ctx.i8_type(), fa, "flag").ok()?.into_int_value();
        let fl = self.builder.build_int_compare(IntPredicate::NE, fv, self.ctx.i8_type().const_zero(), "part_past").ok()?;
        let (next, o) = self.fold_step(fi, fd, total.as_basic_value(), pv, true, func)?;
        let body_end = self.builder.get_insert_block()?;
        let mut past_next = self.builder.build_or(past.as_basic_value().into_int_value(), fl, "past").ok()?;
        if let Some(o) = o {
            past_next = self.builder.build_or(past_next, o, "past").ok()?;
        }
        let zero_next = match (product, pv) {
            (true, BasicValueEnum::IntValue(v)) => {
                let z = self.builder.build_int_compare(IntPredicate::EQ, v, w.const_zero(), "is_zero").ok()?;
                let not_past = self.builder.build_not(fl, "exact").ok()?;
                let z = self.builder.build_and(z, not_past, "zero_part").ok()?;
                self.builder.build_or(zero.as_basic_value().into_int_value(), z, "zero").ok()?
            }
            _ => zero.as_basic_value().into_int_value(),
        };
        let k_next = self.builder.build_int_add(kv, i64t.const_int(1, false), "k").ok()?;
        let _ = self.builder.build_unconditional_branch(head);
        k.add_incoming(&[(&k_next, body_end)]);
        total.add_incoming(&[(&next, body_end)]);
        past.add_incoming(&[(&past_next, body_end)]);
        zero.add_incoming(&[(&zero_next, body_end)]);
        self.builder.position_at_end(done);
        let total = total.as_basic_value();
        match fd.kind {
            // Proven to fit: the wrapped total is the total.
            FoldKind::Int if fd.wrap => {
                let _ = self.builder.build_store(slot, total);
            }
            FoldKind::Int => {
                let (bits, _) = crate::precog::smt::int_shape(&acc_ty)?;
                let mut t = total.into_int_value();
                let mut bad_past = past.as_basic_value().into_int_value();
                if product {
                    // A zero anywhere makes the product 0, whatever the
                    // other slices reached.
                    let z = zero.as_basic_value().into_int_value();
                    t = self.builder.build_select(z, w.const_zero(), t, "product").ok()?.into_int_value();
                    let nz = self.builder.build_not(z, "no_zero").ok()?;
                    bad_past = self.builder.build_and(bad_past, nz, "past").ok()?;
                } else {
                    bad_past = boolt.const_zero();
                }
                let (lo, hi) = crate::resolve::int_range(signed, bits as u32);
                // The site the loop would have stopped at: the store into the
                // variable when that is the one left to check, else the op.
                let mut sites: Vec<SiteId> = fd.skip.iter().filter_map(|(b, si)| match &f.blocks[*b].stmts[*si].kind {
                    StmtKind::Site(id) if matches!(f.site(*id).kind, SiteKind::Overflow(_) | SiteKind::Narrow { .. }) => Some(*id),
                    _ => None,
                }).collect();
                sites.sort_by_key(|id| (!self.gets_check(&f.site(*id).verdict), !matches!(f.site(*id).kind, SiteKind::Narrow { .. }), id.0));
                if let Some(id) = sites.first().copied() {
                    if self.gets_check(&f.site(id).verdict) {
                        let below = self.builder.build_int_compare(IntPredicate::SLT, t, super::const_i128(w, lo), "below").ok()?;
                        let above = self.builder.build_int_compare(IntPredicate::SGT, t, super::const_i128(w, hi), "above").ok()?;
                        let bad = self.builder.build_or(below, above, "outside").ok()?;
                        let bad = self.builder.build_or(bad, bad_past, "outside").ok()?;
                        self.guard(fi, id, bad, func)?;
                    }
                }
                let v = self.builder.build_int_truncate(t, lt.into_int_type(), "total").ok()?;
                let _ = self.builder.build_store(slot, v);
            }
            FoldKind::Bin => {
                // In slices a bin total is grouped otherwise than in order,
                // and can reach infinity where the loop did not: it is
                // always checked, whatever was proven of the loop.
                let t = total.into_float_value();
                let diff = self.builder.build_float_sub(t, t, "finite").ok()?;
                let finite = self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, diff, t.get_type().const_zero(), "finite").ok()?;
                let bad = self.builder.build_not(finite, "infinite").ok()?;
                let what = if fd.op == BinOp::Mul { "product" } else { "sum" };
                self.deci_stop(fi, bad, "value_does_not_fit", &format!("the {} this parallel `for`'s slices made went past the largest bin (it would be infinity or not a number)", what), func)?;
                let _ = self.builder.build_store(slot, t);
            }
            FoldKind::Deci(_) => {
                let _ = self.builder.build_store(slot, total);
            }
        }
        Some(())
    }
}

fn collect_rvalue(rv: &Rvalue, ops: &mut Vec<Operand>) {
    match rv {
        Rvalue::Use(o) | Rvalue::Unary { operand: o, .. } | Rvalue::Convert { operand: o, .. } => ops.push(o.clone()),
        Rvalue::Duplicate(p) | Rvalue::Lend { place: p, .. } => ops.push(Operand::Read(p.clone())),
        Rvalue::Binary { lhs, rhs, .. } => {
            ops.push(lhs.clone());
            ops.push(rhs.clone());
        }
        Rvalue::MakeList(os) => ops.extend(os.iter().cloned()),
        Rvalue::MakeMap(kv) => kv.iter().for_each(|(k, v)| {
            ops.push(k.clone());
            ops.push(v.clone());
        }),
        Rvalue::MakeStruct { fields, .. } => ops.extend(fields.iter().cloned()),
        Rvalue::MakeEnum { payload: Some(o), .. } => ops.push(o.clone()),
        Rvalue::Closure { captures, .. } => ops.extend(captures.iter().map(|(_, p)| Operand::Read(p.clone()))),
        _ => {}
    }
}
