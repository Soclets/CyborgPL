//! MIR to LLVM IR, through inkwell (`design/implementation-languages.md`:
//! inkwell first, `llvm-sys` only where inkwell has not wrapped a call,
//! C++ only for what the C API does not expose).
//!
//! **Precog is the last MIR pass**, so what arrives here is frozen and is
//! lowered as it stands. Each check site already carries its verdict, and
//! the verdict is what decides the instruction:
//!
//! - **proven** → the plain instruction, and the fact handed to LLVM where
//!   LLVM has a home for it (`nsw` / `nuw` on the arithmetic that was
//!   proven not to wrap — `precog.md`: "always where it has a native
//!   home");
//! - **unproven**, working build → the checked instruction, trapping with
//!   the site's own words;
//! - **`nocheck`** → the plain instruction and *no* promise to LLVM: a
//!   trusted site was never proven, and telling LLVM otherwise would turn
//!   the writer's word into undefined behaviour.
//!
//! What is not lowered yet says so by name rather than by crashing.
//! The heap is here — a list is built, grown, indexed, copied and given
//! back (`heap.rs`) — on the layouts the owner decided on 2026-09-22.
//! Still waiting: maps, `concur`, and text built at run time.

use std::collections::HashMap;
use std::path::Path;

use inkwell::basic_block::BasicBlock as LlBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::{AddressSpace, IntPredicate, OptimizationLevel};

use crate::ast::{BinOp, UnOp};
use crate::mir::*;
use crate::source::SourceMap;
use crate::types::{BinW, IntW, Ty};

mod heap;

/// Something the lowering cannot do yet, named so the writer knows what
/// rather than seeing a crash.
pub struct NotYet {
    pub what: String,
    pub span: Option<crate::source::Span>,
}

pub struct Lowered<'ctx> {
    pub module: Module<'ctx>,
}

pub fn lower<'ctx>(ctx: &'ctx Context, p: &Program, sources: &SourceMap, name: &str) -> Result<Lowered<'ctx>, Vec<NotYet>> {
    let module = ctx.create_module(name);
    // The machine's own description, set before anything is lowered: the
    // heap needs to know how big an element is, and only the target data
    // knows that.
    let target = match machine() {
        Ok(m) => {
            module.set_triple(&TargetMachine::get_default_triple());
            module.set_data_layout(&m.get_target_data().get_data_layout());
            m.get_target_data()
        }
        Err(e) => return Err(vec![NotYet { what: format!("this machine has no target description: {}", e), span: None }]),
    };
    let mut cg = Cg { ctx, module: &module, builder: ctx.create_builder(), p, sources, fns: HashMap::new(), externs: HashMap::new(), problems: Vec::new(), strings: HashMap::new(), target, helpers: HashMap::new(), cur_flags: Vec::new() };
    cg.declare_all();
    for i in 0..p.functions.len() {
        cg.define(i);
    }
    cg.emit_entry();
    if !cg.problems.is_empty() {
        return Err(cg.problems);
    }
    // LLVM's own check on what we handed it. A module it refuses is a bug
    // in CCA, not in the program — the same reasoning as the MIR verifier.
    if let Err(e) = module.verify() {
        return Err(vec![NotYet { what: format!("LLVM refused the code CCA generated — this is a bug in CCA, not in the program:\n       {}", e.to_string().replace('\n', "\n       ")), span: None }]);
    }
    Ok(Lowered { module })
}

struct Cg<'a, 'ctx> {
    ctx: &'ctx Context,
    module: &'a Module<'ctx>,
    builder: Builder<'ctx>,
    p: &'a Program,
    sources: &'a SourceMap,
    fns: HashMap<usize, FunctionValue<'ctx>>,
    externs: HashMap<u32, FunctionValue<'ctx>>,
    problems: Vec<NotYet>,
    strings: HashMap<String, PointerValue<'ctx>>,
    /// The machine this is being built for: how big a value is, and how
    /// it is aligned.
    target: inkwell::targets::TargetData,
    /// Generated once per type and called wherever it is needed: dropping
    /// a value, and copying one. Writing them inline would repeat a loop
    /// at every use and could not recurse into a list of lists.
    helpers: HashMap<String, FunctionValue<'ctx>>,
    /// One per local that owns memory, in the function being lowered:
    /// does it still hold a value? A value given away on one path and
    /// kept on another cannot be dropped unconditionally, and cannot be
    /// left undropped either. LLVM folds away the ones whose answer never
    /// changes, which is nearly all of them.
    cur_flags: Vec<Option<PointerValue<'ctx>>>,
}

impl<'a, 'ctx> Cg<'a, 'ctx> {
    fn cannot(&mut self, what: impl Into<String>, span: Option<crate::source::Span>) {
        let what = what.into();
        if !self.problems.iter().any(|p| p.what == what) {
            self.problems.push(NotYet { what, span });
        }
    }

    // ------------------------------------------------------------ types

    /// The LLVM type a Cyborg type is held in. `None` means the lowering
    /// has no shape for it yet.
    fn ty(&mut self, t: &Ty) -> Option<BasicTypeEnum<'ctx>> {
        match t.base() {
            Ty::Int(IntW::Fixed { bits, .. }) => Some(self.ctx.custom_width_int_type(std::num::NonZeroU32::new(*bits as u32)?).ok()?.into()),
            // A width Precog left free reaches here only if nothing ever
            // pinned it; the machine's natural one is the rule
            // (`core-features.md`: a lone scalar takes the natural width).
            Ty::Int(IntW::Free(_)) => Some(self.ctx.i64_type().into()),
            Ty::Bool => Some(self.ctx.bool_type().into()),
            Ty::Bin(BinW::Fixed(16)) => Some(self.ctx.f16_type().into()),
            Ty::Bin(BinW::Fixed(32)) => Some(self.ctx.f32_type().into()),
            Ty::Bin(BinW::Fixed(64)) | Ty::Bin(BinW::Free(_)) => Some(self.ctx.f64_type().into()),
            Ty::Bin(BinW::Fixed(128)) => Some(self.ctx.f128_type().into()),
            Ty::Ptr { .. } => Some(self.ctx.ptr_type(AddressSpace::default()).into()),
            Ty::Char(crate::types::Unit::Byte) => Some(self.ctx.i8_type().into()),
            Ty::Char(crate::types::Unit::Point) => Some(self.ctx.i32_type().into()),
            // The handles the owner decided on 2026-09-22 (`types.md`,
            // `lists.md`, `maps.md`). Each type is exactly as big as it
            // needs to be: a text and an array never grow, so they carry no
            // capacity; a list does, so it carries one; a map that is never
            // walked has no order list to point at.
            Ty::Str(_) => Some(self.handle(2).into()),
            Ty::Seq { kind, .. } => match kind {
                crate::types::SeqKind::Array => Some(self.handle(2).into()),
                // `Free` is Precog's to settle — "a sequence nothing ever
                // appends to becomes an `array`" — and until it does, the
                // shape that can do both is the safe one.
                _ => Some(self.handle(3).into()),
            },
            Ty::Map { order, .. } => match order {
                crate::types::MapOrder::Unordered => Some(self.handle(3).into()),
                _ => Some(self.handle(4).into()),
            },
            Ty::Struct { id, args } => {
                let info = self.p.items.structs[id.0 as usize].clone();
                let mut fields = Vec::new();
                for f in &info.fields {
                    let ft = f.ty.subst(args);
                    match self.ty(&ft) {
                        Some(x) => fields.push(x),
                        None => return None,
                    }
                }
                Some(self.ctx.struct_type(&fields, false).into())
            }
            Ty::Enum { id, args } => {
                let (id, args) = (*id, args.clone());
                self.enum_shape(id, &args).map(|s| s.into())
            }
            Ty::Simd { elem, lanes } => {
                let e = self.ty(elem)?;
                match e {
                    BasicTypeEnum::IntType(i) => Some(i.vec_type(*lanes).into()),
                    BasicTypeEnum::FloatType(f) => Some(f.vec_type(*lanes).into()),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// A handle: a pointer, then `n - 1` machine words. A text and an
    /// array are two fields; a list and an unordered map three; an ordered
    /// map four, the last a pointer to its order list.
    fn handle(&mut self, fields: usize) -> inkwell::types::StructType<'ctx> {
        let p = self.ctx.ptr_type(AddressSpace::default());
        let w = self.ctx.i64_type();
        let mut v: Vec<BasicTypeEnum> = vec![p.into()];
        for i in 1..fields {
            v.push(if i == fields - 1 && fields == 4 { p.into() } else { w.into() });
        }
        self.ctx.struct_type(&v, false)
    }

    fn ty_or_complain(&mut self, t: &Ty, span: Option<crate::source::Span>) -> Option<BasicTypeEnum<'ctx>> {
        let r = self.ty(t);
        if r.is_none() {
            let name = crate::types::TySpelling { ty: t, structs: &self.p.items.structs, enums: &self.p.items.enums }.to_string();
            self.cannot(format!("a `{}` has no shape in the generated code yet: the runtime's layout for text, lists and maps is not decided", name), span);
        }
        r
    }

    // ------------------------------------------------------------ declarations

    fn declare_all(&mut self) {
        for (i, f) in self.p.functions.iter().enumerate() {
            if f.is_main {
                continue;
            }
            let span = f.spans.first().copied();
            if !f.fails.is_empty() {
                self.cannot(format!("`{}` can fail, and a failure has no shape in the generated code yet", f.name), span);
                continue;
            }
            let mut params: Vec<BasicMetadataTypeEnum> = Vec::new();
            let mut ok = true;
            for p in &f.params {
                // A borrow is an address: `lent` and `lent.mut` cross as a
                // pointer, which is what "temporary access without
                // ownership" is on the machine (`ownership-syntax.md`).
                if matches!(f.locals[p.0 as usize].kind, LocalKind::Lent { .. }) {
                    params.push(self.ctx.ptr_type(AddressSpace::default()).into());
                    continue;
                }
                match self.ty_or_complain(&f.locals[p.0 as usize].ty.clone(), span) {
                    Some(t) => params.push(t.into()),
                    None => ok = false,
                }
            }
            if !ok {
                continue;
            }
            let fty = match f.ret {
                Ty::Nothing => self.ctx.void_type().fn_type(&params, false),
                ref r => match self.ty_or_complain(&r.clone(), span) {
                    Some(t) => t.fn_type(&params, false),
                    None => continue,
                },
            };
            let name = format!("cy_{}_{}", i, sanitise(&f.name));
            let v = self.module.add_function(&name, fty, None);
            self.fns.insert(i, v);
        }
        for (i, e) in self.p.items.externs.iter().enumerate() {
            if e.is_let {
                continue;
            }
            let mut params: Vec<BasicMetadataTypeEnum> = Vec::new();
            let mut ok = true;
            for (_, t) in &e.params {
                match self.ty(&t.clone()) {
                    Some(x) => params.push(x.into()),
                    None => ok = false,
                }
            }
            if !ok {
                self.cannot(format!("the `extern fn {}` takes something the generated code has no shape for yet", e.name), None);
                continue;
            }
            let fty = match &e.ret {
                Ty::Nothing => self.ctx.void_type().fn_type(&params, e.variadic),
                r => match self.ty(&r.base().clone()) {
                    Some(t) => t.fn_type(&params, e.variadic),
                    None => {
                        self.cannot(format!("the `extern fn {}` returns something the generated code has no shape for yet", e.name), None);
                        continue;
                    }
                },
            };
            let v = self.module.add_function(&e.name, fty, Some(Linkage::External));
            self.externs.insert(i as u32, v);
        }
        // The runtime's one entry: what a fired check calls.
        let void = self.ctx.void_type();
        let p = self.ctx.ptr_type(AddressSpace::default());
        let i64t = self.ctx.i64_type();
        self.module.add_function("cyborg_check_failed", void.fn_type(&[p.into(), i64t.into()], false), Some(Linkage::External));
    }

    /// `MAIN` becomes C's `main`: the block runs, and its end is exit 0
    /// (`functions.md`: "when the block ends, the process exits 0").
    fn emit_entry(&mut self) {
        let Some(main_idx) = self.p.main else { return };
        let i32t = self.ctx.i32_type();
        let f = self.module.add_function("main", i32t.fn_type(&[], false), None);
        let entry = self.ctx.append_basic_block(f, "entry");
        self.builder.position_at_end(entry);
        let name = format!("cy_main_{}", main_idx.0);
        let body = match self.module.get_function(&name) {
            Some(v) => v,
            None => self.module.add_function(&name, self.ctx.void_type().fn_type(&[], false), None),
        };
        let _ = self.builder.build_call(body, &[], "");
        let _ = self.builder.build_return(Some(&i32t.const_zero()));
    }

    // ------------------------------------------------------------ bodies

    fn define(&mut self, i: usize) {
        let f = &self.p.functions[i];
        let func = if f.is_main {
            let name = format!("cy_main_{}", i);
            match self.module.get_function(&name) {
                Some(v) => v,
                None => self.module.add_function(&name, self.ctx.void_type().fn_type(&[], false), None),
            }
        } else {
            match self.fns.get(&i) {
                Some(v) => *v,
                None => return,
            }
        };
        if f.is_task {
            self.cannot("a `concur` task has no shape in the generated code yet: it needs the runtime's scheduler", f.spans.first().copied());
            return;
        }
        let entry = self.ctx.append_basic_block(func, "entry");
        // One LLVM block per MIR block, made first so an edge can name one
        // that has not been filled in.
        let blocks: Vec<LlBlock> = (0..f.blocks.len()).map(|b| self.ctx.append_basic_block(func, &format!("b{}", b))).collect();
        self.builder.position_at_end(entry);
        // Every local gets a slot; LLVM's own passes promote the ones that
        // never need one.
        let mut slots: Vec<Option<PointerValue>> = Vec::new();
        let lent: Vec<bool> = f.locals.iter().map(|l| matches!(l.kind, LocalKind::Lent { .. })).collect();
        let is_param: Vec<bool> = (0..f.locals.len()).map(|i| f.params.iter().any(|p| p.0 as usize == i)).collect();
        for (li, l) in f.locals.iter().enumerate() {
            let span = Some(f.span(l.span));
            // A lent *parameter* already is an address; it needs no slot of
            // its own, and giving it one would copy what it borrows.
            if lent[li] && is_param[li] {
                slots.push(None);
                continue;
            }
            // A lent local that is not a parameter holds an address: it
            // borrows something that lives elsewhere. Its slot is that
            // address, not a second copy of the value.
            if lent[li] {
                let name = l.name.clone().unwrap_or_else(|| format!("t{}", li));
                let slot = self.builder.build_alloca(self.ctx.ptr_type(AddressSpace::default()), &name).unwrap();
                slots.push(Some(slot));
                continue;
            }
            match self.ty_or_complain(&l.ty.clone(), span) {
                Some(t) => {
                    let name = l.name.clone().unwrap_or_else(|| format!("t{}", li));
                    let slot = self.builder.build_alloca(t, &name).unwrap();
                    slots.push(Some(slot));
                }
                None => slots.push(None),
            }
        }
        // A flag per owning local, so a drop knows whether there is still
        // anything to give back.
        self.cur_flags = vec![None; f.locals.len()];
        for (li, l) in f.locals.iter().enumerate() {
            if lent[li] || !self.owns_memory(&l.ty) {
                continue;
            }
            let flag = self.builder.build_alloca(self.ctx.bool_type(), "alive").unwrap();
            // A parameter arrives holding its value; everything else does
            // not hold one until something puts one there.
            let start = self.ctx.bool_type().const_int(is_param[li] as u64, false);
            let _ = self.builder.build_store(flag, start);
            self.cur_flags[li] = Some(flag);
        }
        if !self.problems.is_empty() {
            return;
        }
        for (pi, p) in f.params.iter().enumerate() {
            let Some(v) = func.get_nth_param(pi as u32) else { continue };
            if lent[p.0 as usize] {
                slots[p.0 as usize] = Some(v.into_pointer_value());
                continue;
            }
            if let Some(slot) = slots[p.0 as usize] {
                let _ = self.builder.build_store(slot, v);
            }
        }
        let _ = self.builder.build_unconditional_branch(blocks[0]);
        for b in 0..f.blocks.len() {
            self.builder.position_at_end(blocks[b]);
            self.block(i, b, &blocks, &slots, func);
        }
    }

    fn block(&mut self, fi: usize, b: usize, blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) {
        let f = &self.p.functions[fi];
        let stmts = f.blocks[b].stmts.clone();
        let mut pending_site: Option<SiteId> = None;
        for s in &stmts {
            match &s.kind {
                StmtKind::Site(id) => {
                    // A bounds check stands on its own — it guards the
                    // index, not the instruction after it — so it is
                    // emitted here rather than carried forward.
                    if matches!(f.site(*id).kind, SiteKind::Bounds) {
                        self.bounds_check(fi, *id, slots, func);
                        pending_site = None;
                    } else {
                        pending_site = Some(*id);
                    }
                }
                StmtKind::Assign { place, rvalue } => {
                    let site = pending_site.take();
                    // The shape of where it is going: a literal has no width
                    // of its own, and a struct is built at its declared one.
                    // Taking a borrow *sets* the borrow; it does not
                    // write through one. Everything else a lent local is
                    // assigned goes through to what it borrows.
                    let binds_borrow = place.proj.is_empty()
                        && matches!(rvalue, Rvalue::Lend { .. })
                        && matches!(f.locals[place.local.0 as usize].kind, LocalKind::Lent { .. })
                        && !f.params.iter().any(|q| *q == place.local);
                    if binds_borrow {
                        if let (Rvalue::Lend { place: src, .. }, Some(slot)) = (rvalue, slots[place.local.0 as usize]) {
                            if let Some((addr, _)) = self.slot_of(fi, src, slots) {
                                let _ = self.builder.build_store(slot, addr);
                            }
                        }
                        continue;
                    }
                    let want_ty = self.slot_of(fi, place, slots).map(|(_, t)| t);
                    let want = want_ty.clone().and_then(|t| self.ty(&t));
                    if let Some(v) = self.rvalue(fi, rvalue, slots, site, func, blocks, want, want_ty.clone()) {
                        self.store_place(fi, place, v, slots);
                        // The place now holds a value worth giving back.
                        if place.proj.is_empty() {
                            if let Some(t) = &want_ty {
                                if self.owns_memory(t) {
                                    self.mark_alive(place.local, true);
                                }
                            }
                        }
                    }
                }
                StmtKind::Op { dest, op, args } => {
                    pending_site = None;
                    match self.op(fi, op, args, slots, func) {
                        Some(v) => {
                            if let (Some(d), Some(v)) = (dest, v) {
                                self.store_place(fi, d, v, slots);
                                if d.proj.is_empty() {
                                    let t = self.p.functions[fi].locals[d.local.0 as usize].ty.clone();
                                    if self.owns_memory(&t) {
                                        self.mark_alive(d.local, true);
                                    }
                                }
                            }
                        }
                        None => {
                            let name = format!("{:?}", op);
                            self.cannot(format!("`{}` has no shape in the generated code yet", readable_op(&name)), Some(f.span(s.span)));
                        }
                    }
                }
                StmtKind::Drop(pl) => {
                    if let Some((slot, ty)) = self.slot_of(fi, pl, slots) {
                        let flag = self.cur_flags.get(pl.local.0 as usize).copied().flatten();
                        self.emit_drop(pl, slot, &ty, flag, func);
                    }
                }
                StmtKind::EndLend(_) | StmtKind::Nop => {}
                StmtKind::Task { .. } => self.cannot("starting a task has no shape in the generated code yet", Some(f.span(s.span))),
                StmtKind::Asm { .. } => self.cannot("`inline.ASM` has no shape in the generated code yet", Some(f.span(s.span))),
            }
        }
        let term = f.blocks[b].term.clone();
        match &term {
            Terminator::Goto(t) => {
                let _ = self.builder.build_unconditional_branch(blocks[t.0 as usize]);
            }
            Terminator::If { cond, then, else_ } => {
                let c = self.operand(fi, cond, slots);
                match c {
                    Some(BasicValueEnum::IntValue(v)) => {
                        let _ = self.builder.build_conditional_branch(v, blocks[then.0 as usize], blocks[else_.0 as usize]);
                    }
                    _ => {
                        let _ = self.builder.build_unconditional_branch(blocks[then.0 as usize]);
                    }
                }
            }
            Terminator::Call { callee, args, dest, ok, fail } => {
                if !fail.is_empty() {
                    self.cannot("a call that can fail has no shape in the generated code yet", Some(f.span(f.blocks[b].term_span)));
                }
                let v = self.call(fi, callee, args, slots);
                if let (Some(d), Some(v)) = (dest, v) {
                    self.store_place(fi, d, v, slots);
                }
                let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
            }
            Terminator::Return => {
                match f.ret_local {
                    Some(r) => match slots[r.0 as usize] {
                        Some(slot) => {
                            let t = self.ty(&f.locals[r.0 as usize].ty.clone());
                            match t {
                                Some(t) => {
                                    let v = self.builder.build_load(t, slot, "ret").unwrap();
                                    let _ = self.builder.build_return(Some(&v));
                                }
                                None => {
                                    let _ = self.builder.build_return(None);
                                }
                            }
                        }
                        None => {
                            let _ = self.builder.build_return(None);
                        }
                    },
                    None => {
                        let _ = self.builder.build_return(None);
                    }
                }
            }
            Terminator::Trap(site) => self.trap(fi, *site),
            Terminator::Unreachable => {
                let _ = self.builder.build_unreachable();
            }
            Terminator::Fail { .. } => {
                self.cannot("`fail` has no shape in the generated code yet", Some(f.span(f.blocks[b].term_span)));
                let _ = self.builder.build_unreachable();
            }
            Terminator::Switch { value, cases } => {
                if self.emit_match(fi, value, cases, blocks, slots, func).is_none() {
                    self.cannot("this `match` has no shape in the generated code yet", Some(f.span(f.blocks[b].term_span)));
                    let _ = self.builder.build_unreachable();
                }
            }
            Terminator::Whichever { .. } | Terminator::Join { .. } => {
                self.cannot("waiting on a task has no shape in the generated code yet", Some(f.span(f.blocks[b].term_span)));
                let _ = self.builder.build_unreachable();
            }
        }
    }

    // ------------------------------------------------------------ values

    fn slot_of(&mut self, fi: usize, p: &Place, slots: &[Option<PointerValue<'ctx>>]) -> Option<(PointerValue<'ctx>, Ty)> {
        let f = &self.p.functions[fi];
        let base_ty = f.locals[p.local.0 as usize].ty.clone();
        let mut slot = slots[p.local.0 as usize]?;
        let mut ty = base_ty;
        // A lent local's slot holds the address it borrows; a lent
        // parameter *is* that address already.
        let borrowed = matches!(f.locals[p.local.0 as usize].kind, LocalKind::Lent { .. }) && !f.params.iter().any(|q| *q == p.local);
        if borrowed {
            slot = self.builder.build_load(self.ctx.ptr_type(AddressSpace::default()), slot, "borrowed").ok()?.into_pointer_value();
        }
        for pr in &p.proj {
            match pr {
                Proj::Field(i) => {
                    if matches!(ty.base(), Ty::Enum { .. }) {
                        // A field of an enum is one of its cases: what the
                        // arm of a `match` binds.
                        slot = self.case_slot(slot, &ty)?;
                        ty = crate::mir::build::projected_ty(&ty, std::slice::from_ref(pr), &self.p.items);
                        continue;
                    }
                    let st = self.ty(&ty)?;
                    let sty = match st {
                        BasicTypeEnum::StructType(s) => s,
                        _ => return None,
                    };
                    slot = self.builder.build_struct_gep(sty, slot, *i as u32, "field").ok()?;
                    ty = crate::mir::build::projected_ty(&ty, std::slice::from_ref(pr), &self.p.items);
                }
                Proj::Index(o) => {
                    let idx = self.operand(fi, &o.clone(), slots)?;
                    let idx = match idx {
                        BasicValueEnum::IntValue(i) => i,
                        _ => return None,
                    };
                    let (next, elem) = self.index_slot(slot, &ty, idx)?;
                    slot = next;
                    ty = elem;
                }
                _ => return None,
            }
        }
        Some((slot, ty))
    }

    fn operand(&mut self, fi: usize, o: &Operand, slots: &[Option<PointerValue<'ctx>>]) -> Option<BasicValueEnum<'ctx>> {
        match o {
            Operand::Const(c) => self.constant(fi, c),
            Operand::Read(p) | Operand::Move(p) => {
                let (slot, ty) = self.slot_of(fi, p, slots)?;
                let t = self.ty(&ty)?;
                let v = self.builder.build_load(t, slot, "v").ok()?;
                // What was given away is no longer here to give back.
                if matches!(o, Operand::Move(_)) && p.proj.is_empty() {
                    self.mark_alive(p.local, false);
                }
                Some(v)
            }
        }
    }

    fn constant(&mut self, fi: usize, c: &Const) -> Option<BasicValueEnum<'ctx>> {
        match c {
            Const::Int(v) => Some(self.ctx.i64_type().const_int(*v as u64, true).into()),
            Const::Bool(b) => Some(self.ctx.bool_type().const_int(*b as u64, false).into()),
            Const::Bin(t) => Some(self.ctx.f64_type().const_float(t.parse().unwrap_or(0.0)).into()),
            Const::Duration(n) => Some(self.ctx.i64_type().const_int(*n as u64, true).into()),
            Const::SizeOf { .. } => Some(self.ctx.i64_type().const_int(8, false).into()),
            Const::Char(s) => Some(self.ctx.i32_type().const_int(s.chars().next().map(|c| c as u64).unwrap_or(0), false).into()),
            Const::Str(text) => {
                // Known at compile time, so it needs no heap: the bytes go
                // into the binary and the handle points at them
                // (`types.md`). Precog knows this memory is not the heap, so
                // nothing frees it and no sentinel is paid.
                let bytes = self.global_bytes(text);
                let len = self.ctx.i64_type().const_int(text.len() as u64, false);
                let h = self.handle(2);
                Some(h.const_named_struct(&[bytes.into(), len.into()]).into())
            }
            Const::Const(id) => {
                let _ = fi;
                self.cannot(format!("the const '{}' is not folded into the generated code yet", self.p.items.consts[id.0 as usize].name), None);
                None
            }
            Const::Nothing => None,
        }
    }

    /// A value, with the site that guards it — when there is one and it was
    /// not proven, the instruction is the checked one.
    #[allow(clippy::too_many_arguments)]
    fn rvalue(&mut self, fi: usize, rv: &Rvalue, slots: &[Option<PointerValue<'ctx>>], site: Option<SiteId>, func: FunctionValue<'ctx>, blocks: &[LlBlock<'ctx>], want: Option<BasicTypeEnum<'ctx>>, want_ty: Option<Ty>) -> Option<BasicValueEnum<'ctx>> {
        match rv {
            Rvalue::Use(o) => {
                let v = self.operand(fi, o, slots)?;
                Some(match want {
                    Some(t) => self.fit(v, t),
                    None => v,
                })
            }
            Rvalue::Unary { op, operand } => {
                let v = self.operand(fi, operand, slots)?;
                match (op, v) {
                    (UnOp::Neg, BasicValueEnum::IntValue(i)) => Some(self.builder.build_int_neg(i, "neg").ok()?.into()),
                    (UnOp::Neg, BasicValueEnum::FloatValue(x)) => Some(self.builder.build_float_neg(x, "neg").ok()?.into()),
                    (UnOp::Not, BasicValueEnum::IntValue(i)) => Some(self.builder.build_not(i, "not").ok()?.into()),
                    (UnOp::NotBit, BasicValueEnum::IntValue(i)) => Some(self.builder.build_not(i, "notbit").ok()?.into()),
                    _ => None,
                }
            }
            Rvalue::Binary { op, lhs, rhs } => {
                let a = self.operand(fi, lhs, slots)?;
                let b = self.operand(fi, rhs, slots)?;
                self.binary(fi, op, a, b, site, func, blocks)
            }
            Rvalue::Convert { to, operand } => {
                let v = self.operand(fi, operand, slots)?;
                let t = self.ty(&to.clone())?;
                match (v, t) {
                    (BasicValueEnum::IntValue(i), BasicTypeEnum::IntType(it)) => {
                        let signed = matches!(to.base(), Ty::Int(IntW::Fixed { signed: true, .. }) | Ty::Int(IntW::Free(_)));
                        Some(self.builder.build_int_cast_sign_flag(i, it, signed, "conv").ok()?.into())
                    }
                    (BasicValueEnum::IntValue(i), BasicTypeEnum::FloatType(ft)) => Some(self.builder.build_signed_int_to_float(i, ft, "conv").ok()?.into()),
                    (BasicValueEnum::FloatValue(x), BasicTypeEnum::FloatType(ft)) => Some(self.builder.build_float_cast(x, ft, "conv").ok()?.into()),
                    _ => {
                        self.cannot("this conversion has no shape in the generated code yet", None);
                        None
                    }
                }
            }
            Rvalue::Lend { place, .. } => {
                // A lend of a plain value is its address; C wants exactly that.
                let (slot, _) = self.slot_of(fi, place, slots)?;
                Some(slot.into())
            }
            Rvalue::MakeStruct { fields, .. } => {
                let shape = match want {
                    Some(BasicTypeEnum::StructType(s)) => Some(s),
                    _ => None,
                };
                let mut vals = Vec::new();
                for (i, o) in fields.iter().enumerate() {
                    let v = self.operand(fi, o, slots)?;
                    let v = match shape.and_then(|s| s.get_field_type_at_index(i as u32)) {
                        Some(t) => self.fit(v, t),
                        None => v,
                    };
                    vals.push(v);
                }
                let tys: Vec<BasicTypeEnum> = vals.iter().map(|v| v.get_type()).collect();
                let st = match shape {
                    Some(s) => s,
                    None => self.ctx.struct_type(&tys, false),
                };
                let mut agg = st.get_undef();
                for (i, v) in vals.into_iter().enumerate() {
                    agg = self.builder.build_insert_value(agg, v, i as u32, "f").ok()?.into_struct_value();
                }
                Some(agg.into())
            }
            Rvalue::MakeEnum { id, case, payload } => {
                let ty = want_ty?;
                self.make_enum(fi, *id, *case, payload, &ty, slots)
            }
            Rvalue::MakeList(ops) => {
                let ty = want_ty?;
                self.make_list(fi, ops, &ty, slots)
            }
            Rvalue::Duplicate(p) => {
                // `copy` is a second, independent value — an allocation
                // (`ownership-syntax.md`). Sharing one buffer between two
                // owners would free it twice.
                let (src, ty) = self.slot_of(fi, p, slots)?;
                match self.copy_fn(&ty) {
                    Some(copier) => {
                        let shape = self.ty(&ty)?;
                        let tmp = self.builder.build_alloca(shape, "copy").ok()?;
                        let _ = self.builder.build_call(copier, &[tmp.into(), src.into()], "");
                        self.builder.build_load(shape, tmp, "copied").ok()
                    }
                    None => self.operand(fi, &Operand::Read(p.clone()), slots),
                }
            }
            _ => {
                self.cannot("this value has no shape in the generated code yet", None);
                None
            }
        }
    }

    fn binary(&mut self, fi: usize, op: &BinOp, a: BasicValueEnum<'ctx>, b: BasicValueEnum<'ctx>, site: Option<SiteId>, func: FunctionValue<'ctx>, blocks: &[LlBlock<'ctx>]) -> Option<BasicValueEnum<'ctx>> {
        let verdict = site.map(|s| self.p.functions[fi].site(s).verdict.clone());
        let proven = matches!(verdict, Some(Verdict::Proven { .. }));
        let trusted = matches!(verdict, Some(Verdict::Trusted));
        let needs_check = site.is_some() && !proven && !trusted;
        let (a, b) = match (a, b) {
            // A bin literal has no width of its own either: it takes the
            // other side's, the same rule as an integer literal.
            (BasicValueEnum::FloatValue(x), BasicValueEnum::FloatValue(y)) if x.get_type() != y.get_type() => {
                match (a, b) {
                    (_, _) if is_double(y.get_type(), self.ctx) => (a, self.fit(b, x.get_type().into())),
                    _ => (self.fit(a, y.get_type().into()), b),
                }
            }
            (BasicValueEnum::IntValue(x), BasicValueEnum::IntValue(y)) if x.get_type().get_bit_width() != y.get_type().get_bit_width() => {
                // One side is a literal, which has no width of its own: it
                // takes the other's.
                if x.get_type().get_bit_width() > y.get_type().get_bit_width() {
                    (a, self.fit(b, x.get_type().into()))
                } else {
                    (self.fit(a, y.get_type().into()), b)
                }
            }
            _ => (a, b),
        };
        if let (BasicValueEnum::IntValue(x), BasicValueEnum::IntValue(y)) = (a, b) {
            let signed = true;
            let r = match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul if needs_check => {
                    let name = match op {
                        BinOp::Add => "llvm.sadd.with.overflow",
                        BinOp::Sub => "llvm.ssub.with.overflow",
                        _ => "llvm.smul.with.overflow",
                    };
                    return self.checked(fi, name, x, y, site.unwrap(), func, blocks);
                }
                // Proven not to wrap: the fact goes to LLVM, which is the
                // whole point of proving it (`precog.md`).
                BinOp::Add => self.builder.build_int_nsw_add(x, y, "add").ok()?,
                BinOp::Sub => self.builder.build_int_nsw_sub(x, y, "sub").ok()?,
                BinOp::Mul => self.builder.build_int_nsw_mul(x, y, "mul").ok()?,
                BinOp::WrapAdd => self.builder.build_int_add(x, y, "wadd").ok()?,
                BinOp::WrapSub => self.builder.build_int_sub(x, y, "wsub").ok()?,
                BinOp::WrapMul => self.builder.build_int_mul(x, y, "wmul").ok()?,
                BinOp::Div => self.builder.build_int_signed_div(x, y, "div").ok()?,
                BinOp::Mod => self.builder.build_int_signed_rem(x, y, "rem").ok()?,
                BinOp::AndBit | BinOp::And => self.builder.build_and(x, y, "and").ok()?,
                BinOp::OrBit | BinOp::Or => self.builder.build_or(x, y, "or").ok()?,
                BinOp::Xor => self.builder.build_xor(x, y, "xor").ok()?,
                BinOp::Shl => self.builder.build_left_shift(x, y, "shl").ok()?,
                BinOp::Shr => self.builder.build_right_shift(x, y, signed, "shr").ok()?,
                BinOp::Eq => self.builder.build_int_compare(IntPredicate::EQ, x, y, "eq").ok()?,
                BinOp::Ne => self.builder.build_int_compare(IntPredicate::NE, x, y, "ne").ok()?,
                BinOp::Lt => self.builder.build_int_compare(IntPredicate::SLT, x, y, "lt").ok()?,
                BinOp::Le => self.builder.build_int_compare(IntPredicate::SLE, x, y, "le").ok()?,
                BinOp::Gt => self.builder.build_int_compare(IntPredicate::SGT, x, y, "gt").ok()?,
                BinOp::Ge => self.builder.build_int_compare(IntPredicate::SGE, x, y, "ge").ok()?,
                _ => {
                    self.cannot(format!("`{}` has no shape in the generated code yet", op.spelled()), None);
                    return None;
                }
            };
            return Some(r.into());
        }
        if let (BasicValueEnum::FloatValue(x), BasicValueEnum::FloatValue(y)) = (a, b) {
            use inkwell::FloatPredicate as FP;
            let r: BasicValueEnum = match op {
                BinOp::Add => self.builder.build_float_add(x, y, "fadd").ok()?.into(),
                BinOp::Sub => self.builder.build_float_sub(x, y, "fsub").ok()?.into(),
                BinOp::Mul => self.builder.build_float_mul(x, y, "fmul").ok()?.into(),
                BinOp::Div => self.builder.build_float_div(x, y, "fdiv").ok()?.into(),
                BinOp::Eq => self.builder.build_float_compare(FP::OEQ, x, y, "feq").ok()?.into(),
                BinOp::Ne => self.builder.build_float_compare(FP::ONE, x, y, "fne").ok()?.into(),
                BinOp::Lt => self.builder.build_float_compare(FP::OLT, x, y, "flt").ok()?.into(),
                BinOp::Le => self.builder.build_float_compare(FP::OLE, x, y, "fle").ok()?.into(),
                BinOp::Gt => self.builder.build_float_compare(FP::OGT, x, y, "fgt").ok()?.into(),
                BinOp::Ge => self.builder.build_float_compare(FP::OGE, x, y, "fge").ok()?.into(),
                _ => {
                    self.cannot(format!("`{}` on bins has no shape in the generated code yet", op.spelled()), None);
                    return None;
                }
            };
            return Some(r);
        }
        None
    }

    /// An unproven arithmetic site: the checked instruction, and a trap
    /// that says what happened in the writer's words.
    fn checked(&mut self, fi: usize, intrinsic: &str, x: IntValue<'ctx>, y: IntValue<'ctx>, site: SiteId, func: FunctionValue<'ctx>, _blocks: &[LlBlock<'ctx>]) -> Option<BasicValueEnum<'ctx>> {
        let Some(id) = inkwell::intrinsics::Intrinsic::find(intrinsic) else {
            self.cannot(format!("LLVM has no `{}` here", intrinsic), None);
            return None;
        };
        let Some(f) = id.get_declaration(self.module, &[x.get_type().into()]) else {
            self.cannot(format!("LLVM would not give `{}` for this width", intrinsic), None);
            return None;
        };
        let call = self.builder.build_call(f, &[x.into(), y.into()], "chk").ok()?;
        let agg = basic_result(call)?.into_struct_value();
        let value = self.builder.build_extract_value(agg, 0, "v").ok()?;
        let flag = self.builder.build_extract_value(agg, 1, "o").ok()?.into_int_value();
        let bad = self.ctx.append_basic_block(func, "check_failed");
        let good = self.ctx.append_basic_block(func, "checked");
        let _ = self.builder.build_conditional_branch(flag, bad, good);
        self.builder.position_at_end(bad);
        self.trap(fi, site);
        self.builder.position_at_end(good);
        Some(value)
    }

    /// Stop the program where a check fired, naming the site
    /// (`diagnostics.md`: "the trap at run time names the site").
    fn trap(&mut self, fi: usize, site: SiteId) {
        let f = &self.p.functions[fi];
        let s = f.site(site);
        let span = f.span(s.span);
        let file = self.sources.get(span.file);
        let (line, col) = file.line_col(span.start);
        let spell = |t: &Ty| crate::types::TySpelling { ty: t, structs: &self.p.items.structs, enums: &self.p.items.enums }.to_string();
        let msg = format!("check: {}\nwhere: {}, line {}, column {}\n", s.kind.sentence(&spell), file.display_name(), line, col);
        let ptr = self.global_bytes(&msg);
        let len = self.ctx.i64_type().const_int(msg.len() as u64, false);
        if let Some(handler) = self.module.get_function("cyborg_check_failed") {
            let args: Vec<BasicMetadataValueEnum> = vec![ptr.into(), len.into()];
            let _ = self.builder.build_call(handler, &args, "");
        }
        let _ = self.builder.build_unreachable();
    }

    fn global_bytes(&mut self, s: &str) -> PointerValue<'ctx> {
        if let Some(p) = self.strings.get(s) {
            return *p;
        }
        let g = self.builder.build_global_string_ptr(s, "msg").unwrap();
        let p = g.as_pointer_value();
        self.strings.insert(s.to_string(), p);
        p
    }

    /// A value at the width the place it is going has. A literal has no
    /// width of its own until it lands somewhere (`types.md`).
    fn fit(&mut self, v: BasicValueEnum<'ctx>, want: BasicTypeEnum<'ctx>) -> BasicValueEnum<'ctx> {
        match (v, want) {
            (BasicValueEnum::IntValue(i), BasicTypeEnum::IntType(t)) if i.get_type().get_bit_width() != t.get_bit_width() => {
                self.builder.build_int_cast_sign_flag(i, t, true, "fit").map(|x| x.into()).unwrap_or(v)
            }
            (BasicValueEnum::IntValue(i), BasicTypeEnum::FloatType(t)) => self.builder.build_signed_int_to_float(i, t, "fit").map(|x| x.into()).unwrap_or(v),
            (BasicValueEnum::FloatValue(x), BasicTypeEnum::FloatType(t)) if x.get_type() != t => self.builder.build_float_cast(x, t, "fit").map(|y| y.into()).unwrap_or(v),
            _ => v,
        }
    }

    fn call(&mut self, fi: usize, callee: &Callee, args: &[Arg], slots: &[Option<PointerValue<'ctx>>]) -> Option<BasicValueEnum<'ctx>> {
        let mut vals: Vec<BasicMetadataValueEnum> = Vec::new();
        for a in args {
            let v = match a {
                Arg::Value(o) => self.operand(fi, o, slots)?,
                Arg::Copy(p) => self.operand(fi, &Operand::Read(p.clone()), slots)?,
                Arg::Give(p) => self.operand(fi, &Operand::Move(p.clone()), slots)?,
                Arg::Lend { place, .. } => {
                    let (slot, ty) = self.slot_of(fi, place, slots)?;
                    // "A lend satisfies a pointer: `write[1, lend 'bytes', 3]`
                    // hands C the address of the lent *data*" (`ffi.md`), so
                    // what crosses is the handle's pointer, not the handle.
                    if matches!(callee, Callee::Extern(_)) && is_handle(&ty) {
                        let shape = self.ty(&ty)?;
                        let st = match shape {
                            BasicTypeEnum::StructType(s) => s,
                            _ => return None,
                        };
                        let field = self.builder.build_struct_gep(st, slot, 0, "data").ok()?;
                        self.builder.build_load(self.ctx.ptr_type(AddressSpace::default()), field, "p").ok()?
                    } else {
                        slot.into()
                    }
                }
                Arg::Spread { .. } => {
                    self.cannot("`...` at a call has no shape in the generated code yet", None);
                    return None;
                }
            };
            vals.push(v.into());
        }
        let f = match callee {
            Callee::Fn(idx) => *self.fns.get(&(idx.0 as usize))?,
            Callee::Extern(id) => *self.externs.get(&id.0)?,
            _ => {
                self.cannot("this call has no shape in the generated code yet", None);
                return None;
            }
        };
        // Each argument at the width the callee declared.
        let want = f.get_type().get_param_types();
        for (i, w) in want.iter().enumerate() {
            if let Some(slot) = vals.get_mut(i) {
                if let (Ok(v), Ok(t)) = (BasicValueEnum::try_from(*slot), BasicTypeEnum::try_from(*w)) {
                    *slot = self.fit(v, t).into();
                }
            }
        }
        let call = self.builder.build_call(f, &vals, "call").ok()?;
        basic_result(call)
    }

    /// The language's own operations. A length is a field of the handle,
    /// which is why the handle carries one.
    fn op(&mut self, fi: usize, op: &Op, args: &[Operand], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<Option<BasicValueEnum<'ctx>>> {
        match op {
            // `list.append` hands nothing back (`lists.md`), so it is
            // handled and has no value — which is not the same as not
            // being handled at all.
            Op::ListAppend => {
                self.list_append(fi, args, slots, func)?;
                Some(None)
            }
            Op::ListRemove => Some(Some(self.list_remove(fi, args, slots)?)),
            // Bytes is the length a `str` already carries; the other two
            // units have to walk the text, and that is std's to write.
            Op::StrBytes | Op::ListLength | Op::ArrayLength | Op::MapLength => {
                let p = args.first()?.place()?;
                let (slot, ty) = self.slot_of(fi, p, slots)?;
                let shape = self.ty(&ty)?;
                let st = match shape {
                    BasicTypeEnum::StructType(s) => s,
                    _ => return None,
                };
                let field = self.builder.build_struct_gep(st, slot, 1, "len").ok()?;
                let raw = self.builder.build_load(self.ctx.i64_type(), field, "n").ok()?;
                Some(Some(raw))
            }
            _ => None,
        }
    }

    /// `'xs'[pos i]` must be within the list: 1-based, so 1 through the
    /// length (`lists.md`). Proven sites carry no check; the rest trap
    /// with the site's own words.
    fn bounds_check(&mut self, fi: usize, id: SiteId, slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) {
        let site = self.p.functions[fi].site(id).clone();
        if matches!(site.verdict, Verdict::Proven { .. } | Verdict::Trusted) {
            return;
        }
        // Reading, never moving: a check site looks at the value, it does
        // not take it.
        let read = |o: &Operand| match o.place() {
            Some(p) => Operand::Read(p.clone()),
            None => o.clone(),
        };
        let (Some(i), Some(n)) = (site.operands.first().map(read), site.operands.get(1).map(read)) else { return };
        let (Some(i), Some(n)) = (self.operand(fi, &i, slots), self.operand(fi, &n, slots)) else { return };
        let w = self.ctx.i64_type();
        let i = self.fit(i, w.into()).into_int_value();
        let n = self.fit(n, w.into()).into_int_value();
        let one = w.const_int(1, false);
        let Ok(too_low) = self.builder.build_int_compare(IntPredicate::SLT, i, one, "low") else { return };
        let Ok(too_high) = self.builder.build_int_compare(IntPredicate::SGT, i, n, "high") else { return };
        let Ok(bad) = self.builder.build_or(too_low, too_high, "outside") else { return };
        let fail = self.ctx.append_basic_block(func, "index_outside");
        let good = self.ctx.append_basic_block(func, "index_ok");
        let _ = self.builder.build_conditional_branch(bad, fail, good);
        self.builder.position_at_end(fail);
        self.trap(fi, id);
        self.builder.position_at_end(good);
    }

    fn store_place(&mut self, fi: usize, p: &Place, v: BasicValueEnum<'ctx>, slots: &[Option<PointerValue<'ctx>>]) {
        if let Some((slot, ty)) = self.slot_of(fi, p, slots) {
            // The slot's width is what the type says; a value computed at a
            // different width is fitted to it, which is the store the check
            // site already asked about.
            let want = self.ty(&ty);
            let v = match (v, want) {
                (BasicValueEnum::IntValue(i), Some(BasicTypeEnum::IntType(t))) if i.get_type().get_bit_width() != t.get_bit_width() => self.builder.build_int_cast_sign_flag(i, t, true, "fit").unwrap().into(),
                _ => v,
            };
            let _ = self.builder.build_store(slot, v);
        }
    }
}

/// The value a call produced, when it produced one.
fn basic_result(call: inkwell::values::CallSiteValue<'_>) -> Option<BasicValueEnum<'_>> {
    match call.try_as_basic_value() {
        inkwell::values::ValueKind::Basic(v) => Some(v),
        _ => None,
    }
}

/// A bin constant is built at 64 bits until something says otherwise, so
/// that is the side to move when two widths meet.
fn is_double(t: inkwell::types::FloatType<'_>, ctx: &Context) -> bool {
    t == ctx.f64_type()
}

/// A value the runtime holds behind a handle.
fn is_handle(t: &Ty) -> bool {
    matches!(t.base(), Ty::Str(_) | Ty::Seq { .. } | Ty::Map { .. })
}

fn sanitise(name: &str) -> String {
    name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect()
}

fn readable_op(debug: &str) -> String {
    let base = debug.split('(').next().unwrap_or(debug);
    let mut out = String::new();
    for (i, c) in base.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('.');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

/// Write the module as an object file for this machine.
/// This machine, described: the triple, the CPU and what it can do.
fn machine() -> Result<TargetMachine, String> {
    Target::initialize_native(&InitializationConfig::default()).map_err(|e| e.to_string())?;
    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).map_err(|e| e.to_string())?;
    target
        .create_target_machine(&triple, &TargetMachine::get_host_cpu_name().to_string(), &TargetMachine::get_host_cpu_features().to_string(), OptimizationLevel::Default, RelocMode::PIC, CodeModel::Default)
        .ok_or_else(|| "this machine has no target description".to_string())
}

pub fn write_object(module: &Module, path: &Path) -> Result<(), String> {
    let machine = machine()?;
    module.set_triple(&TargetMachine::get_default_triple());
    module.set_data_layout(&machine.get_target_data().get_data_layout());
    machine.write_to_file(module, FileType::Object, path).map_err(|e| e.to_string())
}

pub use inkwell;

/// Link the object with the runtime, using the system linker
/// (`implementation-languages.md`: LLVM, then the system linker).
pub fn link(obj: &Path, out: &Path) -> Result<(), String> {
    let runtime = runtime_path().ok_or_else(|| "the runtime is not built: `cargo build -p cca-runtime --release`".to_string())?;
    let status = std::process::Command::new("cc")
        .arg(obj)
        .arg(&runtime)
        .arg("-o")
        .arg(out)
        .status()
        .map_err(|e| format!("the linker would not start: {}", e))?;
    if status.success() {
        Ok(())
    } else {
        Err("the linker refused the program".into())
    }
}

fn runtime_path() -> Option<std::path::PathBuf> {
    // Beside the compiler, which is where cargo puts it.
    let exe = std::env::current_exe().ok()?;
    for dir in exe.parent()?.ancestors().take(4) {
        for profile in ["release", "debug"] {
            let p = dir.join(profile).join("libcca_runtime.a");
            if p.is_file() {
                return Some(p);
            }
        }
        let p = dir.join("libcca_runtime.a");
        if p.is_file() {
            return Some(p);
        }
    }
    None
}
