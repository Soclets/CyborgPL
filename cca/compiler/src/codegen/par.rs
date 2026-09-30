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
use inkwell::types::BasicTypeEnum;
use inkwell::values::{FunctionValue, IntValue, PointerValue};
use inkwell::{AddressSpace, IntPredicate};

use super::Cg;
use crate::mir::*;
use crate::par::{ParLoop, MAX_SLICES, WORTH};

/// What the slice being written leaves out, and where its folds go.
pub(super) struct ParChild<'ctx> {
    pub skip: HashSet<(usize, usize)>,
    /// (block, statement) of a fold's add: which fold, and the term added.
    pub adds: HashMap<(usize, usize), (usize, Operand)>,
    /// Each fold's 128-bit partial, in the slice's frame.
    pub partials: Vec<PointerValue<'ctx>>,
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
        // How many trips, in 128 bits: a range's from its first value to
        // its stop by its step, a list's its length.
        let (trips, go_ok) = match pl.fl.step {
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
                let nonneg = self.builder.build_int_compare(IntPredicate::SGE, span, zero, "reached").ok()?;
                let safe_mag = self.builder.build_select(self.builder.build_int_compare(IntPredicate::EQ, mag, zero, "zero").ok()?, w.const_int(1, false), mag, "by").ok()?.into_int_value();
                let q = self.builder.build_int_signed_div(span, safe_mag, "q").ok()?;
                let n = self.builder.build_int_add(q, w.const_int(1, false), "n").ok()?;
                let n = self.builder.build_select(nonneg, n, zero, "trips").ok()?.into_int_value();
                let moving = self.builder.build_int_compare(IntPredicate::NE, step, zero, "moving").ok()?;
                (n, moving)
            }
            None => (load_wide(self, pl.fl.stop, false)?, self.ctx.bool_type().const_int(1, false)),
        };
        let work = self.builder.build_int_mul(trips, w.const_int(pl.work, false), "work").ok()?;
        let pays = self.builder.build_int_compare(IntPredicate::UGE, work, w.const_int(WORTH, false), "pays").ok()?;
        let several = self.builder.build_int_compare(IntPredicate::SGE, trips, w.const_int(2, false), "several").ok()?;
        let go = self.builder.build_and(pays, several, "go").ok()?;
        let go = self.builder.build_and(go, go_ok, "go").ok()?;
        let par_b = self.ctx.append_basic_block(func, "in_slices");
        let _ = self.builder.build_conditional_branch(go, par_b, head);
        self.builder.position_at_end(par_b);

        let shared = self.par_shared(fi, pl);
        let child = self.par_child_fn(fi, pl, &shared)?;
        let p = self.ctx.ptr_type(AddressSpace::default());
        // The addresses: every shared local's, then each fold's partials.
        let nfolds = pl.folds.len();
        let env_ty = p.array_type((shared.len() + nfolds) as u32);
        let env = self.stack(env_ty, "slices_env").ok()?;
        for (k, l) in shared.iter().enumerate() {
            let at = unsafe { self.builder.build_gep(env_ty, env, &[i64t.const_zero(), i64t.const_int(k as u64, false)], "env_at").ok()? };
            let addr = match slots[*l as usize] {
                Some(s) => s,
                None => p.const_null(),
            };
            let _ = self.builder.build_store(at, addr);
        }
        let parts_ty = w.array_type(MAX_SLICES as u32);
        let mut partials = Vec::new();
        for r in 0..nfolds {
            let arr = self.stack(parts_ty, "partials").ok()?;
            let _ = self.builder.build_store(arr, parts_ty.const_zero());
            let at = unsafe { self.builder.build_gep(env_ty, env, &[i64t.const_zero(), i64t.const_int((shared.len() + r) as u64, false)], "env_at").ok()? };
            let _ = self.builder.build_store(at, arr);
            partials.push(arr);
        }
        let run = self.rt("cyborg_par_for");
        let trips64 = self.builder.build_int_truncate(trips, i64t, "trips").ok()?;
        let _ = self.builder.build_call(run, &[child.as_global_value().as_pointer_value().into(), env.into(), trips64.into()], "slices");
        // Each fold: what the variable held, plus every slice's partial,
        // once checked.
        for (r, fd) in pl.folds.iter().enumerate() {
            let acc_ty = f.locals[fd.acc.0 as usize].ty.clone();
            let (bits, signed) = crate::precog::smt::int_shape(&acc_ty)?;
            let mut sum = load_wide(self, fd.acc, signed)?;
            for k in 0..MAX_SLICES {
                let at = unsafe { self.builder.build_gep(parts_ty, partials[r], &[i64t.const_zero(), i64t.const_int(k, false)], "part").ok()? };
                let v = self.builder.build_load(w, at, "part").ok()?.into_int_value();
                sum = self.builder.build_int_add(sum, v, "sum").ok()?;
            }
            let (lo, hi) = crate::resolve::int_range(signed, bits as u16);
            // The site the loop would have stopped at: the store into the
            // variable when that is the one left to check, else the add.
            let mut sites: Vec<SiteId> = fd.skip.iter().filter_map(|(b, si)| match &f.blocks[*b].stmts[*si].kind {
                StmtKind::Site(id) if matches!(f.site(*id).kind, SiteKind::Overflow(_) | SiteKind::Narrow { .. }) => Some(*id),
                _ => None,
            }).collect();
            sites.sort_by_key(|id| (!self.gets_check(&f.site(*id).verdict), !matches!(f.site(*id).kind, SiteKind::Narrow { .. }), id.0));
            let site = sites.first().copied();
            if let Some(id) = site {
                if self.gets_check(&f.site(id).verdict) {
                    let below = self.builder.build_int_compare(IntPredicate::SLT, sum, super::const_i128(w, lo), "below").ok()?;
                    let above = self.builder.build_int_compare(IntPredicate::SGT, sum, super::const_i128(w, hi), "above").ok()?;
                    let bad = self.builder.build_or(below, above, "outside").ok()?;
                    self.guard(fi, id, bad, func)?;
                }
            }
            let slot = slots[fd.acc.0 as usize]?;
            let t = self.ty(&acc_ty)?.into_int_type();
            let v = self.builder.build_int_truncate(sum, t, "total").ok()?;
            let _ = self.builder.build_store(slot, v);
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

        let entry = self.ctx.append_basic_block(child, "entry");
        self.builder.position_at_end(entry);
        let env = child.get_nth_param(0)?.into_pointer_value();
        let first = child.get_nth_param(1)?.into_int_value();
        let end = child.get_nth_param(2)?.into_int_value();
        let slice = child.get_nth_param(3)?.into_int_value();
        let env_ty = p.array_type((shared.len() + pl.folds.len()) as u32);
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
            if *l == pl.fl.counter.0 {
                parent_counter = Some(addr);
            } else if Some(LocalId(*l)) == pl.fl.step {
                parent_step = Some(addr);
            } else {
                slots[*l as usize] = Some(addr);
            }
        }
        // The step, read from the parent's frame when the loop never
        // writes it (it never does).
        if let (Some(s), Some(addr)) = (pl.fl.step, parent_step) {
            slots[s.0 as usize] = Some(addr);
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
        // The folds' partials, from 0.
        let mut partials: Vec<PointerValue<'ctx>> = Vec::new();
        for _ in &pl.folds {
            let part = self.stack(w, "partial").ok()?;
            let _ = self.builder.build_store(part, w.const_zero());
            partials.push(part);
        }
        let mut skip = HashSet::new();
        let mut adds = HashMap::new();
        for (r, fd) in pl.folds.iter().enumerate() {
            skip.extend(fd.skip.iter().copied());
            for (b, si, x) in &fd.adds {
                adds.insert((*b, *si), (r, x.clone()));
            }
        }
        self.par_child = Some(ParChild { skip, adds, partials: partials.clone() });
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
        let parts_ty = w.array_type(MAX_SLICES as u32);
        for (r, part) in partials.iter().enumerate() {
            let part = *part;
            let at = unsafe { self.builder.build_gep(env_ty, env, &[i64t.const_zero(), i64t.const_int((shared.len() + r) as u64, false)], "env_at").ok()? };
            let arr = self.builder.build_load(p, at, "partials").ok()?.into_pointer_value();
            let slot = unsafe { self.builder.build_gep(parts_ty, arr, &[i64t.const_zero(), slice], "mine").ok()? };
            let v = self.builder.build_load(w, part, "partial").ok()?;
            let _ = self.builder.build_store(slot, v);
        }
        let _ = self.builder.build_return(None);

        self.cur_flags = saved_flags;
        self.here = saved_here;
        self.op_sites = saved_sites;
        self.par_child = saved_child;
        if let Some(b) = back {
            self.builder.position_at_end(b);
        }
        Some(child)
    }

    /// In a slice: a fold's add becomes an add to the slice's partial.
    pub(super) fn par_fold_add(&mut self, fi: usize, which: usize, x: &Operand, slots: &[Option<PointerValue<'ctx>>]) -> Option<()> {
        let w = self.ctx.i128_type();
        let part = self.par_child.as_ref()?.partials.get(which).copied()?;
        let v = self.operand(fi, x, slots)?.into_int_value();
        let signed = self.operand_signed(fi, x).unwrap_or(true);
        let v = self.builder.build_int_cast_sign_flag(v, w, signed, "term").ok()?;
        let cur = self.builder.build_load(w, part, "partial").ok()?.into_int_value();
        let next = self.builder.build_int_add(cur, v, "partial").ok()?;
        let _ = self.builder.build_store(part, next);
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
