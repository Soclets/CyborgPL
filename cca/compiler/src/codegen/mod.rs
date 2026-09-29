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

mod cabi;
mod deci;
mod heap;
mod tasks;

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
    lower_opts(ctx, p, sources, name, false)
}

/// `assert_proofs`: the build that checks Precog (`precog.md`, attack
/// (d)). Every site Precog proved keeps a run-time check anyway, and
/// nothing proven is promised to LLVM — no `nsw`, no `nuw` — so a wrong
/// proof shows up as `proof failed:` and exit 104 instead of as silent
/// undefined behaviour. A `nocheck` site proved nothing and is not
/// asserted.
pub fn lower_opts<'ctx>(ctx: &'ctx Context, p: &Program, sources: &SourceMap, name: &str, assert_proofs: bool) -> Result<Lowered<'ctx>, Vec<NotYet>> {
    let module = ctx.create_module(name);
    // The machine's own description, set before anything is lowered: the
    // heap needs to know how big an element is, and only the target data
    // knows that.
    let target = match machine() {
        Ok(m) => {
            module.set_triple(&host_triple());
            module.set_data_layout(&m.get_target_data().get_data_layout());
            m.get_target_data()
        }
        Err(e) => return Err(vec![NotYet { what: format!("this machine has no target description: {}", e), span: None }]),
    };
    let mut cg = Cg { ctx, module: &module, builder: ctx.create_builder(), p, sources, fns: HashMap::new(), env_fns: HashMap::new(), externs: HashMap::new(), problems: Vec::new(), strings: HashMap::new(), target, helpers: HashMap::new(), cur_flags: Vec::new(), cur_groups: Vec::new(), cur_outcomes: HashMap::new(), assert_proofs, here: (0, 0), cur_fi: 0, op_sites: Vec::new() };
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
    /// Function literals that capture: body index to how many captures.
    /// Each takes the block of captured addresses as its first parameter.
    env_fns: HashMap<usize, usize>,
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
    /// In the function being lowered: one group per block that starts
    /// tasks, and where each task that can fail leaves its outcome.
    cur_groups: Vec<PointerValue<'ctx>>,
    cur_outcomes: HashMap<u32, PointerValue<'ctx>>,
    /// `--assert-proofs`: proven sites are checked too.
    assert_proofs: bool,
    /// The statement being lowered, (block, index): where Precog's ranges
    /// for it are kept.
    here: (u32, u32),
    /// The function being written.
    cur_fi: usize,
    /// The check sites guarding the operation being lowered.
    op_sites: Vec<SiteId>,
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
            // Its layout (BID), in an integer of its width.
            Ty::Deci(32) => Some(self.ctx.i32_type().into()),
            Ty::Deci(64) => Some(self.ctx.i64_type().into()),
            Ty::Deci(_) => Some(self.ctx.i128_type().into()),
            Ty::Ptr { .. } => Some(self.ctx.ptr_type(AddressSpace::default()).into()),
            // One block the runtime owns; the program holds a pointer to it.
            Ty::Channel(_) => Some(self.ctx.ptr_type(AddressSpace::default()).into()),
            Ty::Char(crate::types::Unit::Byte) => Some(self.ctx.i8_type().into()),
            // "One representation underneath: nanoseconds in an integer wide
            // enough for centuries" (`concurrency.md`): 64 bits hold 292
            // years, and a duration is never negative.
            Ty::Duration => Some(self.ctx.i64_type().into()),
            // A function held as a value (`functions-as-values.md`): its
            // code and the block of what it captured, null when it captured
            // nothing — then the code is the plain function, which is what
            // C wants of a callback (`ffi.md`). Provisional (`cca/README.md`).
            Ty::Fn(_) => {
                let p = self.ctx.ptr_type(AddressSpace::default());
                Some(self.ctx.struct_type(&[p.into(), p.into()], false).into())
            }
            // A variadic bundle of values of different types: side by side,
            // walked by unrolling (`functions.md`).
            Ty::Bundle(ts) => {
                let mut fields = Vec::new();
                for t in ts.clone() {
                    fields.push(self.ty(&t)?);
                }
                Some(self.ctx.struct_type(&fields, false).into())
            }
            // A unit not written is Precog's to pick (`types.md`: "from the
            // content and its use"); one code point is the flat, fast unit.
            // A literal of several points is a grapheme, and waits.
            Ty::Char(crate::types::Unit::Point) | Ty::Char(crate::types::Unit::Free(_)) => Some(self.ctx.i32_type().into()),
            // The handles the owner decided on 2026-09-22 (`types.md`,
            // `lists.md`, `maps.md`). Each type is exactly as big as it
            // needs to be: a text and an array never grow, so they carry no
            // capacity; a list does, so it carries one; a map that is never
            // walked has no order list to point at.
            // A grapheme is a text of its own (`types.md`: "variable size ...
            // an owned value like `str`"): held as one, pointer and length.
            Ty::Str(_) | Ty::Char(crate::types::Unit::Grapheme) => Some(self.handle(2).into()),
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
            // A vector whose lane count nothing settled has no shape; LLVM
            // panics on one, so it is refused here instead.
            Ty::Simd { lanes: 0, .. } => None,
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
            // Text, lists and maps do have a shape now; what is left is
            // what the design has not settled or the lowering has not
            // reached, so the message says which.
            let why = match t.base() {
                Ty::Channel(_) => ": a channel's layout waits for `concur` to generate code",
                Ty::Fn(_) => ": a function held as a value has no shape yet",
                Ty::Char(crate::types::Unit::Grapheme) => ": a grapheme is a text of its own and waits with the rest of `char`",
                _ => "",
            };
            self.cannot(format!("a `{}` has no shape in the generated code yet{}", name, why), span);
        }
        r
    }

    // ------------------------------------------------------------ declarations

    fn declare_all(&mut self) {
        for f in &self.p.functions {
            for b in &f.blocks {
                for s in &b.stmts {
                    if let StmtKind::Assign { rvalue: Rvalue::Closure { body, captures }, .. } = &s.kind {
                        if !captures.is_empty() {
                            self.env_fns.insert(body.0 as usize, captures.len());
                        }
                    }
                }
            }
        }
        for (i, f) in self.p.functions.iter().enumerate() {
            if f.is_main {
                continue;
            }
            let span = f.spans.first().copied();
            let mut params: Vec<BasicMetadataTypeEnum> = Vec::new();
            if self.env_fns.contains_key(&i) {
                params.push(self.ctx.ptr_type(AddressSpace::default()).into());
            }
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
            // A function that can fail hands back the answer *or* a
            // reason, which is one value with a tag on it.
            let fty = if !f.fails.is_empty() {
                match self.fail_shape(i) {
                    Some(t) => t.fn_type(&params, false),
                    None => {
                        self.cannot(format!("`{}` fails with something the generated code has no shape for yet", f.name), span);
                        continue;
                    }
                }
            } else if f.ret_lent.is_some() {
                // A borrow handed back is an address, like a `lent` one.
                self.ctx.ptr_type(AddressSpace::default()).fn_type(&params, false)
            } else {
                match f.ret {
                    Ty::Nothing => self.ctx.void_type().fn_type(&params, false),
                    ref r => match self.ty_or_complain(&r.clone(), span) {
                        Some(t) => t.fn_type(&params, false),
                        None => continue,
                    },
                }
            };
            let name = format!("cy_{}_{}", i, sanitise(&f.name));
            let v = self.module.add_function(&name, fty, None);
            self.fns.insert(i, v);
        }
        for (i, e) in self.p.items.externs.iter().enumerate() {
            if e.is_let {
                continue;
            }
            // C calling back with a struct by value takes a thunk in C's
            // convention, which is not built.
            let back = e.params.iter().any(|(_, t)| match t.base() {
                Ty::Fn(f) => f.params.iter().chain(std::iter::once(&(crate::types::ParamKind::Owned, f.ret.clone()))).any(|(_, p)| matches!(p.base(), Ty::Struct { .. })),
                _ => false,
            });
            if back {
                self.cannot(format!("the `extern fn {}` takes a callback that passes or returns a struct by value, which needs a thunk in C's calling convention, and that is not built yet", e.name), None);
                continue;
            }
            let mut params: Vec<BasicMetadataTypeEnum> = Vec::new();
            let mut ok = true;
            // A struct returned through room the caller gives (`cabi.rs`).
            let sret = match &e.ret {
                Ty::Nothing => None,
                r => match self.c_pass(&r.base().clone(), true) {
                    Some(cabi::CPass::Indirect(t)) => Some(t),
                    _ => None,
                },
            };
            if sret.is_some() {
                params.push(self.ctx.ptr_type(AddressSpace::default()).into());
            }
            for (_, t) in &e.params {
                // A callback slot holds an address and nothing else (`ffi.md`).
                if matches!(t.base(), Ty::Fn(_)) {
                    params.push(self.ctx.ptr_type(AddressSpace::default()).into());
                    continue;
                }
                match (self.ty(&t.clone()), self.c_pass(&t.clone(), false)) {
                    (Some(x), Some(cabi::CPass::Direct)) => params.push(x.into()),
                    (Some(_), Some(cabi::CPass::Coerce(c))) => params.push(c.into()),
                    (Some(_), Some(cabi::CPass::Indirect(_))) => params.push(self.ctx.ptr_type(AddressSpace::default()).into()),
                    _ => ok = false,
                }
            }
            if !ok {
                self.cannot(format!("the `extern fn {}` takes something the generated code has no shape for yet", e.name), None);
                continue;
            }
            let fty = match &e.ret {
                Ty::Nothing => self.ctx.void_type().fn_type(&params, e.variadic),
                _ if sret.is_some() => self.ctx.void_type().fn_type(&params, e.variadic),
                r => match (self.ty(&r.base().clone()), self.c_pass(&r.base().clone(), true)) {
                    (Some(t), Some(cabi::CPass::Direct)) => t.fn_type(&params, e.variadic),
                    (Some(_), Some(cabi::CPass::Coerce(c))) => c.fn_type(&params, e.variadic),
                    _ => {
                        self.cannot(format!("the `extern fn {}` returns something the generated code has no shape for yet", e.name), None);
                        continue;
                    }
                },
            };
            let v = self.module.add_function(&e.name, fty, Some(Linkage::External));
            if let Some(t) = sret {
                let kind = inkwell::attributes::Attribute::get_named_enum_kind_id("sret");
                v.add_attribute(inkwell::attributes::AttributeLoc::Param(0), self.ctx.create_type_attribute(kind, inkwell::types::AnyType::as_any_type_enum(&t)));
            }
            self.externs.insert(i as u32, v);
        }
        // The runtime's one entry: what a fired check calls.
        let void = self.ctx.void_type();
        let p = self.ctx.ptr_type(AddressSpace::default());
        let i64t = self.ctx.i64_type();
        self.module.add_function("cyborg_check_failed", void.fn_type(&[p.into(), i64t.into()], false), Some(Linkage::External));
        self.module.add_function("cyborg_proof_failed", void.fn_type(&[p.into(), i64t.into()], false), Some(Linkage::External));
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
        if self.uses_tasks() {
            // MAIN runs as a task, so it can wait like one.
            let run = self.module.get_function("cyborg_run").unwrap_or_else(|| {
                let p = self.ctx.ptr_type(AddressSpace::default());
                self.module.add_function("cyborg_run", self.ctx.void_type().fn_type(&[p.into()], false), None)
            });
            let _ = self.builder.build_call(run, &[body.as_global_value().as_pointer_value().into()], "");
        } else {
            let _ = self.builder.build_call(body, &[], "");
        }
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
            // `extern let` (`ffi.md`): the place a write lands is C's own
            // variable, not a copy of it on the stack.
            if let Some(id) = l.name.as_deref().and_then(|n| n.strip_prefix("extern:")).and_then(|n| n.parse::<u32>().ok()) {
                slots.push(self.extern_global(id));
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
        self.cur_fi = i;
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
        self.task_frame(i);
        if !self.problems.is_empty() {
            return;
        }
        // A literal that captured: its first parameter is the block of the
        // addresses it borrowed, one per capture, and its captures are its
        // first locals (`check_instance` declares them before the
        // parameters). Each lent capture's slot holds the address.
        // A lent capture's slot holds the address it borrows; an owned one
        // (`copy`, `give`) lives in the block itself, which is its slot.
        let skip = match self.env_fns.get(&i).copied() {
            Some(n) => {
                let env = func.get_nth_param(0).map(|v| v.into_pointer_value());
                let p = self.ctx.ptr_type(AddressSpace::default());
                let Some(st) = self.env_shape(i, n) else { return };
                for c in 0..n {
                    let Some(env) = env else { continue };
                    let Ok(at) = self.builder.build_struct_gep(st, env, 2 + c as u32, "capture") else { continue };
                    if lent[c] {
                        let Some(slot) = slots[c] else { continue };
                        let addr = self.builder.build_load(p, at, "captured").unwrap();
                        let _ = self.builder.build_store(slot, addr);
                    } else {
                        slots[c] = Some(at);
                    }
                }
                1
            }
            None => 0,
        };
        for (pi, p) in f.params.iter().enumerate() {
            let Some(v) = func.get_nth_param(pi as u32 + skip) else { continue };
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
        // The arithmetic guards on the next instruction — all of them: a
        // division is guarded twice, once against a zero divisor and once
        // against the lowest value divided by -1, and keeping only the last
        // site read dropped the first check.
        let mut pending: Vec<SiteId> = Vec::new();
        for (si, s) in stmts.iter().enumerate() {
            self.here = (b as u32, si as u32);
            match &s.kind {
                StmtKind::Site(id) => match f.site(*id).kind {
                    // These stand on their own: they guard a value that is
                    // already there, not the instruction after them.
                    SiteKind::Bounds => self.bounds_check(fi, *id, slots, func),
                    SiteKind::SliceRange => self.slice_check(fi, *id, slots, func),
                    SiteKind::NegativeExponent => self.negative_exponent_check(fi, *id, slots, func),
                    SiteKind::SimdRange { lanes } => self.simd_range_check(fi, *id, lanes, slots, func),
                    SiteKind::SimdSome { lanes } => self.simd_some_check(fi, *id, lanes, slots, func),
                    SiteKind::Narrow { .. } => self.narrow_check(fi, *id, slots, func),
                    SiteKind::LengthBound { .. } => self.length_check(fi, *id, slots, func),
                    SiteKind::Overflow(_) | SiteKind::DivZero | SiteKind::ModZero | SiteKind::ShiftWidth | SiteKind::ShiftCount => pending.push(*id),
                    // Checked inside the map, by the use that follows.
                    SiteKind::KeyHashAgrees => pending.push(*id),
                    _ => {}
                },
                StmtKind::Assign { place, rvalue } => {
                    let sites = std::mem::take(&mut pending);
                    // The shape of where it is going: a literal has no width
                    // of its own, and a struct is built at its declared one.
                    // What a lent local holds is an address, so assigning
                    // to one stores an address — whether it is a fresh
                    // borrow (`_3 = lend 'xs'`) or another borrow passed
                    // along (`_4 = _3`). Writing the borrowed *value*
                    // there instead would write through a pointer that
                    // has not been set yet.
                    let binds_borrow = place.proj.is_empty()
                        && matches!(f.locals[place.local.0 as usize].kind, LocalKind::Lent { .. })
                        && !f.params.iter().any(|q| *q == place.local);
                    if binds_borrow {
                        let src = match rvalue {
                            Rvalue::Lend { place, .. } => Some(place.clone()),
                            Rvalue::Use(o) => o.place().cloned(),
                            _ => None,
                        };
                        if let (Some(src), Some(slot)) = (src, slots[place.local.0 as usize]) {
                            if let Some((addr, _)) = self.slot_of(fi, &src, slots) {
                                let _ = self.builder.build_store(slot, addr);
                                continue;
                            }
                        }
                        self.cannot("this borrow has no shape in the generated code yet", Some(f.span(s.span)));
                        continue;
                    }
                    let want_ty = self.slot_of(fi, place, slots).map(|(_, t)| t);
                    let want = want_ty.clone().and_then(|t| self.ty(&t));
                    if let Some(v) = self.rvalue(fi, rvalue, slots, &sites, func, blocks, want, want_ty.clone()) {
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
                    // The sites guarding this operation (`simd.sum`'s).
                    self.op_sites = std::mem::take(&mut pending);
                    let dest_ty = dest.as_ref().and_then(|d| self.slot_of(fi, d, slots)).map(|(_, t)| t);
                    match self.op(fi, op, args, slots, func, dest_ty) {
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
                StmtKind::Task { detached, body, captures, group, id } => {
                    let span = Some(f.span(s.span));
                    if self.emit_task(fi, *detached, *body, captures, *group, *id, slots, span).is_none() {
                        self.cannot("this task has no shape in the generated code yet", span);
                    }
                }
                StmtKind::Asm { text, ins, outs, clobbers } => {
                    if self.inline_asm(fi, text, ins, outs, clobbers, slots).is_none() {
                        self.cannot("this `inline.ASM` has no shape in the generated code yet", Some(f.span(s.span)));
                    }
                }
            }
        }
        // The sites guarding a call that ends the block (`map.get`'s).
        self.op_sites = std::mem::take(&mut pending);
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
            // `map.get` and `map.remove` fail when the key is not there
            // (`maps.md`), and that is the only reason either gives.
            Terminator::Call { callee: Callee::Op(Op::ChannelTake), args, dest, ok, fail } if !fail.is_empty() => {
                if self.chan_take(fi, args, dest, *ok, fail, blocks, slots, func).is_none() {
                    self.cannot("this `channel.take` has no shape in the generated code yet", Some(f.span(f.blocks[b].term_span)));
                    let _ = self.builder.build_unreachable();
                }
            }
            Terminator::Call { callee: Callee::Op(op @ (Op::MapGet | Op::MapRemove)), args, dest, ok, fail } if !fail.is_empty() => {
                if self.map_lookup(fi, op, args, dest, *ok, fail, blocks, slots, func).is_none() {
                    self.cannot("this map lookup has no shape in the generated code yet", Some(f.span(f.blocks[b].term_span)));
                    let _ = self.builder.build_unreachable();
                }
            }
            Terminator::Call { callee, args, dest, ok, fail } if !fail.is_empty() => {
                if self.emit_failing_call(fi, callee, args, dest, *ok, fail, blocks, slots, func).is_none() {
                    // Say whose call: a std one waits on the std Limb, not
                    // on the base language.
                    let what = match callee {
                        Callee::Op(Op::Std(name)) => format!("`std::{}`, a call that can fail, has no shape in the generated code yet: std's", name),
                        _ => "this call that can fail has no shape in the generated code yet".to_string(),
                    };
                    self.cannot(what, Some(f.span(f.blocks[b].term_span)));
                    let _ = self.builder.build_unreachable();
                }
            }
            Terminator::Call { callee, args, dest, ok, .. } => {
                let v = self.call(fi, callee, args, slots);
                // A borrow handed back: the variable holding it holds the
                // address (`-> lent`, owner 2026-09-29).
                let borrows = dest.as_ref().is_some_and(|d| d.proj.is_empty() && matches!(f.locals[d.local.0 as usize].kind, LocalKind::Lent { .. }));
                if let (true, Some(d), Some(v)) = (borrows, dest, v) {
                    if let Some(slot) = slots[d.local.0 as usize] {
                        let _ = self.builder.build_store(slot, v);
                    }
                } else if let (Some(d), Some(v)) = (dest, v) {
                    self.store_place(fi, d, v, slots);
                    // What came back is here now, and is given back at the
                    // end like anything else held. Without this an owned
                    // answer — a text a function built — was never freed.
                    if d.proj.is_empty() {
                        self.mark_alive(d.local, true);
                    }
                }
                let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
            }
            Terminator::Return if !f.fails.is_empty() => {
                if self.emit_ok_return(fi, slots).is_none() {
                    self.cannot("this `return` from a function that can fail has no shape in the generated code yet", Some(f.span(f.blocks[b].term_span)));
                    let _ = self.builder.build_unreachable();
                }
            }
            Terminator::Return => {
                match f.ret_local {
                    // A borrow: the address its slot holds.
                    Some(r) if f.ret_lent.is_some() => {
                        let p = self.ctx.ptr_type(AddressSpace::default());
                        match slots[r.0 as usize] {
                            Some(slot) => {
                                let v = self.builder.build_load(p, slot, "ret").unwrap();
                                let _ = self.builder.build_return(Some(&v));
                            }
                            None => {
                                let _ = self.builder.build_return(Some(&p.const_null()));
                            }
                        }
                    }
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
            Terminator::Fail { index, payload } => {
                if self.emit_fail(fi, *index, payload, slots).is_none() {
                    self.cannot("this `fail` has no shape in the generated code yet", Some(f.span(f.blocks[b].term_span)));
                    let _ = self.builder.build_unreachable();
                }
            }
            Terminator::Switch { value, cases } => {
                if self.emit_match(fi, value, cases, blocks, slots, func).is_none() {
                    self.cannot("this `match` has no shape in the generated code yet", Some(f.span(f.blocks[b].term_span)));
                    let _ = self.builder.build_unreachable();
                }
            }
            Terminator::Join { group, ok } => {
                if self.emit_join(*group).is_none() {
                    self.cannot("this wait for tasks has no shape in the generated code yet", Some(f.span(f.blocks[b].term_span)));
                }
                let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
            }
            Terminator::TaskOutcome { id, body, ok, fail } => {
                if self.emit_task_outcome(fi, *id, *body, *ok, fail, blocks, slots, func).is_none() {
                    self.cannot("this task's failure has no shape in the generated code yet", Some(f.span(f.blocks[b].term_span)));
                    let _ = self.builder.build_unreachable();
                }
            }
            Terminator::Whichever { arms, closed } => {
                if self.emit_whichever(fi, arms, *closed, blocks, slots, func).is_none() {
                    self.cannot("this `whichever` has no shape in the generated code yet", Some(f.span(f.blocks[b].term_span)));
                    let _ = self.builder.build_unreachable();
                }
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
                // A value of a bundle of different types: its field.
                Proj::BundleElem(i) => {
                    let Ty::Bundle(ts) = ty.base() else { return None };
                    let et = ts.get(*i)?.clone();
                    let st = match self.ty(&ty)? {
                        BasicTypeEnum::StructType(s) => s,
                        _ => return None,
                    };
                    slot = self.builder.build_struct_gep(st, slot, *i as u32, "elem").ok()?;
                    ty = et;
                }
                #[allow(unreachable_patterns)]
                _ => {
                    self.cannot("this place has no shape in the generated code yet", None);
                    return None;
                }
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
            // A literal is built at 64 bits, or at 128 when 64 cannot hold
            // it signed — `10000000000000000000` is a fine `uint64`, and
            // built at 64 bits and then widened it would read as negative.
            Const::Int(v) if *v > i64::MAX as i128 || *v < i64::MIN as i128 => {
                let t = self.ctx.i128_type();
                let lo = *v as u128 as u64;
                let hi = ((*v as u128) >> 64) as u64;
                Some(t.const_int_arbitrary_precision(&[lo, hi]).into())
            }
            Const::Int(v) => Some(self.ctx.i64_type().const_int(*v as u64, true).into()),
            Const::Bool(b) => Some(self.ctx.bool_type().const_int(*b as u64, false).into()),
            Const::Bin(t) => Some(self.ctx.f64_type().const_float(t.parse().unwrap_or(0.0)).into()),
            Const::Deci { bits, value } => {
                let t = match bits {
                    32 => self.ctx.i32_type(),
                    64 => self.ctx.i64_type(),
                    _ => self.ctx.i128_type(),
                };
                Some(t.const_int_arbitrary_precision(&[*value as u64, (*value >> 64) as u64]).into())
            }
            Const::Duration(n) => Some(self.ctx.i64_type().const_int(*n as u64, true).into()),
            // C's size and alignment of the type (`ffi.md`), from the
            // target's own layout. It was 8 for everything, and qsort
            // walked an `int32` list 8 bytes at a time.
            Const::SizeOf { align, ty } => {
                let t = self.ty(&ty.clone())?;
                let n = if *align { self.target.get_abi_alignment(&t) as u64 } else { self.target.get_abi_size(&t) };
                Some(self.ctx.i64_type().const_int(n, false).into())
            }
            Const::Char(s) => {
                if s.chars().count() != 1 {
                    self.cannot(format!("the character `{}` is more than one code point, a grapheme, which waits with the rest of `char`", s), None);
                    return None;
                }
                Some(self.ctx.i32_type().const_int(s.chars().next().map(|c| c as u64).unwrap_or(0), false).into())
            }
            Const::Str(text) => {
                // Known at compile time, so it needs no heap: the bytes go
                // into the binary and the handle points at them
                // (`types.md`). Precog knows this memory is not the heap, so
                // nothing frees it and no sentinel is paid.
                let bytes = self.global_bytes(text);
                // The mark that says these bytes are the binary's own and
                // must never be freed.
                let len = self.ctx.i64_type().const_int(text.len() as u64 | (1u64 << 63), false);
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
    fn rvalue(&mut self, fi: usize, rv: &Rvalue, slots: &[Option<PointerValue<'ctx>>], sites: &[SiteId], func: FunctionValue<'ctx>, blocks: &[LlBlock<'ctx>], want: Option<BasicTypeEnum<'ctx>>, want_ty: Option<Ty>) -> Option<BasicValueEnum<'ctx>> {
        match rv {
            Rvalue::Use(o) => {
                // A bin literal landing in a bin16 or bin32 is written at
                // that width, rounded once from its decimal: read at 64 bits
                // first and then narrowed, a literal near a halfway point is
                // rounded twice and can land a step off.
                if let (Operand::Const(Const::Bin(text)), Some(BasicTypeEnum::FloatType(t))) = (o, want) {
                    return Some(unsafe { t.const_float_from_string(&text.replace('_', "")) }.into());
                }
                // A character literal landing in a grapheme: its text.
                if let (Operand::Const(Const::Char(s)), Some(BasicTypeEnum::StructType(_))) = (o, want) {
                    return Some(self.const_text(s));
                }
                let v = self.operand(fi, o, slots)?;
                let signed = self.operand_signed(fi, o).or_else(|| want_ty.as_ref().and_then(int_signed)).unwrap_or(true);
                Some(match want {
                    Some(t) => self.fit_as(v, t, signed),
                    None => v,
                })
            }
            Rvalue::Unary { op, operand } => {
                let v = self.operand(fi, operand, slots)?;
                if let (UnOp::Neg, BasicValueEnum::IntValue(i), Some(Ty::Deci(_))) = (op, v, self.operand_cy_ty(fi, operand).map(|t| t.base().clone())) {
                    return self.deci_neg(i);
                }
                match (op, v) {
                    (UnOp::Neg, BasicValueEnum::IntValue(i)) => Some(self.builder.build_int_neg(i, "neg").ok()?.into()),
                    (UnOp::Neg, BasicValueEnum::FloatValue(x)) => Some(self.builder.build_float_neg(x, "neg").ok()?.into()),
                    (UnOp::Not, BasicValueEnum::IntValue(i)) => Some(self.builder.build_not(i, "not").ok()?.into()),
                    (UnOp::NotBit, BasicValueEnum::IntValue(i)) => Some(self.builder.build_not(i, "notbit").ok()?.into()),
                    // A mask flips lane by lane.
                    (UnOp::Not, BasicValueEnum::VectorValue(m)) => Some(self.builder.build_not(m, "not").ok()?.into()),
                    _ => None,
                }
            }
            Rvalue::Binary { op, lhs, rhs } => {
                // A grapheme beside a character literal: the literal as text.
                let grapheme = |this: &mut Self, o: &Operand| matches!(this.operand_cy_ty(fi, o).map(|t| t.base().clone()), Some(Ty::Char(crate::types::Unit::Grapheme)));
                let (lg, rg) = (grapheme(self, lhs), grapheme(self, rhs));
                let a = match lhs {
                    Operand::Const(Const::Char(s)) if rg => self.const_text(s),
                    _ => self.operand(fi, lhs, slots)?,
                };
                let b = match rhs {
                    Operand::Const(Const::Char(s)) if lg => self.const_text(s),
                    _ => self.operand(fi, rhs, slots)?,
                };
                // A decimal is its layout, not a number LLVM knows: every
                // operation, `==` too (`1.50 == 1.5`), is the runtime's.
                let deci = [lhs, rhs].iter().find_map(|o| match self.operand_cy_ty(fi, o).map(|t| t.base().clone()) {
                    Some(Ty::Deci(bits)) => Some(bits),
                    _ => None,
                });
                if let Some(bits) = deci {
                    let exp_signed = self.operand_signed(fi, rhs).unwrap_or(true);
                    return self.deci_binary(fi, op, a, b, bits, exp_signed, func);
                }
                if a.is_vector_value() || b.is_vector_value() {
                    let lane = self.operand_lane(fi, lhs).or_else(|| self.operand_lane(fi, rhs));
                    return self.vector_binary(fi, op, a, b, lane, sites, func);
                }
                // `==` on an enum, a struct, a text, a list: the same shape
                // all the way down (`enums.md`: "== works on a case").
                if matches!(op, BinOp::Eq | BinOp::Ne) && a.is_struct_value() && b.is_struct_value() {
                    let ty = self.operand_cy_ty(fi, lhs).or_else(|| self.operand_cy_ty(fi, rhs))?;
                    let f = self.same_fn(&ty)?;
                    let pa = self.builder.build_alloca(a.get_type(), "eq_a").ok()?;
                    let pb = self.builder.build_alloca(b.get_type(), "eq_b").ok()?;
                    let _ = self.builder.build_store(pa, a);
                    let _ = self.builder.build_store(pb, b);
                    let same = basic_result(self.builder.build_call(f, &[pa.into(), pb.into()], "same").ok()?)?.into_int_value();
                    return Some(if *op == BinOp::Eq { same.into() } else { self.builder.build_not(same, "differ").ok()?.into() });
                }
                // `++` on text is an allocation, not an instruction. A
                // character beside text is its text first (`` `é` ++ "!" ``);
                // the one made for it is given back once joined.
                if matches!(op, BinOp::Join) {
                    let ty = want_ty.clone().unwrap_or(Ty::Str(crate::types::Enc::Utf8));
                    let mut made = Vec::new();
                    let mut as_text = |this: &mut Self, o: &Operand, v: BasicValueEnum<'ctx>| -> Option<BasicValueEnum<'ctx>> {
                        if v.is_struct_value() {
                            return Some(v);
                        }
                        let t = this.to_text(fi, o, v, &ty)?;
                        made.push(t);
                        Some(t)
                    };
                    let a = as_text(self, lhs, a)?;
                    let b = as_text(self, rhs, b)?;
                    let joined = self.str_concat(a, b, &ty)?;
                    for t in made {
                        self.free_text(t);
                    }
                    return Some(joined);
                }
                // LLVM's integers carry no sign; the Cyborg type does. A
                // literal takes its context's family (`types.md`), so the
                // side that is a place decides.
                // Both sides literals: neither has a family, so the result's
                // type says it — sign and width both. A product of two bare
                // numbers going into a `uint64` is an unsigned 64-bit
                // multiply; checked as signed, 2 x 7726793855274730819 was
                // an overflow that never happens. (The ordinary build folds
                // it to its answer; `--assert-proofs` computes it.)
                // Plain math is exact (owner, 2026-09-23): each operand is
                // taken at its true value, by its own sign — a literal by its
                // written one — and `+ - x` are done in 128 bits, where no
                // two values of 65 bits can overflow an add and the site
                // asks the only question left: is the answer still one some
                // 64-bit type holds? `/` and `mod` cannot leave that range,
                // so they are done at the exact width itself.
                if want_ty.as_ref().map_or(false, |t| t.is_exact()) && !op.is_comparison() && !matches!(op, BinOp::WrapAdd | BinOp::WrapSub | BinOp::WrapMul | BinOp::WrapPow) {
                    if let (BasicValueEnum::IntValue(x), BasicValueEnum::IntValue(y)) = (a, b) {
                        // Where every value the operands and the answer
                        // take fits 64 bits, that is the width it is done
                        // at: the same answer, and a 65-bit `mod` is a
                        // library call.
                        let range = if self.assert_proofs { None } else { self.p.functions[fi].ranges.get(&self.here).copied() };
                        if let Some((lo, hi)) = range {
                            let unsigned = lo >= 0 && hi <= u64::MAX as i128;
                            if unsigned || (lo >= i64::MIN as i128 && hi <= i64::MAX as i128) {
                                let w = self.ctx.i64_type();
                                let at64 = |this: &mut Self, o: &Operand, v: IntValue<'ctx>| -> Option<IntValue<'ctx>> {
                                    if let Operand::Const(Const::Int(c)) = o {
                                        return Some(const_i128(w, *c));
                                    }
                                    let s = this.operand_signed(fi, o).unwrap_or(true);
                                    this.builder.build_int_cast_sign_flag(v, w, s, "at64").ok()
                                };
                                let x = at64(self, lhs, x)?;
                                let y = at64(self, rhs, y)?;
                                let r = match op {
                                    BinOp::Add if unsigned => self.builder.build_int_nuw_add(x, y, "add").ok()?,
                                    BinOp::Add => self.builder.build_int_nsw_add(x, y, "add").ok()?,
                                    BinOp::Sub if unsigned => self.builder.build_int_nuw_sub(x, y, "sub").ok()?,
                                    BinOp::Sub => self.builder.build_int_nsw_sub(x, y, "sub").ok()?,
                                    BinOp::Mul if unsigned => self.builder.build_int_nuw_mul(x, y, "mul").ok()?,
                                    BinOp::Mul => self.builder.build_int_nsw_mul(x, y, "mul").ok()?,
                                    // A divisor that can be zero is still
                                    // its own site; `binary` guards it.
                                    _ => self.binary(fi, op, x.into(), y.into(), sites, func, blocks, !unsigned)?.into_int_value(),
                                };
                                let t = self.ty(want_ty.as_ref()?)?.into_int_type();
                                return Some(self.builder.build_int_cast_sign_flag(r, t, !unsigned, "exact").ok()?.into());
                            }
                        }
                        let plain = matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow);
                        let work = if plain { self.ctx.i128_type() } else { self.ctx.custom_width_int_type(std::num::NonZeroU32::new(crate::types::EXACT_BITS as u32)?).ok()? };
                        let at_true = |this: &mut Self, o: &Operand, v: IntValue<'ctx>| -> Option<IntValue<'ctx>> {
                            if let Operand::Const(Const::Int(c)) = o {
                                return Some(const_i128(work, *c));
                            }
                            let s = this.operand_signed(fi, o).unwrap_or(true);
                            if v.get_type().get_bit_width() == work.get_bit_width() {
                                return Some(v);
                            }
                            this.builder.build_int_cast_sign_flag(v, work, s, "exact").ok()
                        };
                        let x = at_true(self, lhs, x)?;
                        let y = at_true(self, rhs, y)?;
                        let r = self.binary(fi, op, x.into(), y.into(), sites, func, blocks, true)?.into_int_value();
                        if plain {
                            self.past_64_bits(fi, r, sites, func);
                        }
                        let t = self.ty(want_ty.as_ref()?)?.into_int_type();
                        return Some(if r.get_type().get_bit_width() == t.get_bit_width() { r.into() } else { self.builder.build_int_truncate(r, t, "exact").ok()?.into() });
                    }
                }
                // A comparison is of the two numbers, whatever their types:
                // each side is extended by its *own* sign, one bit wider
                // when the signs differ, and compared signed. Extending both
                // by the left side's sign made a `uint8` of 201 into -55,
                // and `198 > 201` came out true (check claude, seed 77).
                if op.is_comparison() {
                    if let (BasicValueEnum::IntValue(x), BasicValueEnum::IntValue(y)) = (a, b) {
                        // A literal is compared at its written value, never
                        // squeezed into the other side's width: `'v' > 200`
                        // on an `int8` is false, not a comparison with -56.
                        let lit = |o: &Operand| if let Operand::Const(Const::Int(c)) = o { Some(*c) } else { None };
                        let (la, lb) = (lit(lhs), lit(rhs));
                        let (sa, sb) = (self.operand_signed(fi, lhs), self.operand_signed(fi, rhs));
                        let (xw, yw) = (x.get_type().get_bit_width(), y.get_type().get_bit_width());
                        let mixed = matches!((sa, sb), (Some(p), Some(q)) if p != q || xw != yw);
                        // Only a place's width limits this: a literal is
                        // written at its value whatever width it was built
                        // at, and 2^63 is built at 128 bits — counted, it
                        // sent `2^63 </== 'v1'` to a 16-bit compare with 0
                        // (check claude, seed 1086).
                        let widest = match (la, lb) {
                            (Some(_), None) => yw,
                            (None, Some(_)) => xw,
                            _ => xw.max(yw),
                        };
                        if (la.is_some() != lb.is_some() || mixed) && widest <= 127 {
                            let w = self.ctx.i128_type();
                            let x = match la { Some(c) => const_i128(w, c), None => self.builder.build_int_cast_sign_flag(x, w, sa.unwrap_or(true), "cmpl").ok()? };
                            let y = match lb { Some(c) => const_i128(w, c), None => self.builder.build_int_cast_sign_flag(y, w, sb.unwrap_or(true), "cmpr").ok()? };
                            return self.binary(fi, op, x.into(), y.into(), sites, func, blocks, true);
                        }
                    }
                }
                // Bins: the operation is done at the answer's own width (a
                // comparison at the wider side's), each operand brought to
                // it and a literal written straight at it. An `fadd float`
                // stored into a free-width `bin`'s `double` left half of it
                // garbage. In arithmetic a bare literal has the typed side's
                // width (owner, 2026-09-24: a); in a comparison it stays at
                // 64 bits, as in the oracle: `'f' > 0.1` with a bin32 0.1 is
                // true.
                if let (BasicValueEnum::FloatValue(x), BasicValueEnum::FloatValue(y)) = (a, b) {
                    let at = if op.is_comparison() {
                        let (xt, yt) = (x.get_type(), y.get_type());
                        let bits = |t: inkwell::types::FloatType<'ctx>| if t == self.ctx.f16_type() { 16 } else if t == self.ctx.f32_type() { 32 } else { 64 };
                        if bits(xt) >= bits(yt) { xt } else { yt }
                    } else {
                        match want {
                            Some(BasicTypeEnum::FloatType(t)) => t,
                            _ => x.get_type(),
                        }
                    };
                    let bring = |this: &mut Self, o: &Operand, v: inkwell::values::FloatValue<'ctx>| -> Option<inkwell::values::FloatValue<'ctx>> {
                        if let Operand::Const(Const::Bin(text)) = o {
                            return Some(unsafe { at.const_float_from_string(&text.replace('_', "")) });
                        }
                        if v.get_type() == at {
                            return Some(v);
                        }
                        this.builder.build_float_cast(v, at, "at_width").ok()
                    };
                    let x = bring(self, lhs, x)?;
                    let y = bring(self, rhs, y)?;
                    return self.binary(fi, op, x.into(), y.into(), sites, func, blocks, true);
                }
                let signed = self.operand_signed(fi, lhs).or_else(|| self.operand_signed(fi, rhs)).or_else(|| want_ty.as_ref().and_then(int_signed)).unwrap_or(true);
                let (a, b) = match (a, b, want) {
                    (BasicValueEnum::IntValue(x), BasicValueEnum::IntValue(y), Some(t @ BasicTypeEnum::IntType(_))) if x.is_const() && y.is_const() && !op.is_comparison() => (self.fit_as(a, t, signed), self.fit_as(b, t, signed)),
                    _ => (a, b),
                };
                self.binary(fi, op, a, b, sites, func, blocks, signed)
            }
            Rvalue::Convert { to, operand, useful } => {
                let mut v = self.operand(fi, operand, slots)?;
                if let Some(from) = self.operand_cy_ty(fi, operand) {
                    if matches!(from.base(), Ty::Deci(_)) && !matches!(to.base(), Ty::Str(_)) || matches!(to.base(), Ty::Deci(_)) {
                        let from_signed = self.operand_signed(fi, operand).unwrap_or(true);
                        return self.deci_convert(fi, v, &from, to, *useful, from_signed, func);
                    }
                }
                // A C enum's number is its tag (`cabi.rs`, `tag_of`).
                if let BasicValueEnum::StructValue(s) = v {
                    if matches!(self.operand_cy_ty(fi, operand).as_ref().map(|t| t.base()), Some(Ty::Enum { .. })) {
                        v = self.builder.build_extract_value(s, 0, "number").ok()?;
                    }
                }
                if let Ty::Str(_) | Ty::Char(crate::types::Unit::Grapheme) = to.base() {
                    return self.to_text(fi, operand, v, to);
                }
                let t = self.ty(&to.clone())?;
                // Widening extends by what the value *is*, not by what it
                // becomes: a `uint8` of 250 widened to 128 bits is 250, and
                // sign-extending it made it -6.
                let from_signed = self.operand_signed(fi, operand).unwrap_or(true);
                match (v, t) {
                    (BasicValueEnum::IntValue(i), BasicTypeEnum::IntType(it)) => Some(self.builder.build_int_cast_sign_flag(i, it, from_signed, "conv").ok()?.into()),
                    (BasicValueEnum::IntValue(i), BasicTypeEnum::FloatType(ft)) if !from_signed => Some(self.builder.build_unsigned_int_to_float(i, ft, "conv").ok()?.into()),
                    (BasicValueEnum::IntValue(i), BasicTypeEnum::FloatType(ft)) => Some(self.builder.build_signed_int_to_float(i, ft, "conv").ok()?.into()),
                    (BasicValueEnum::FloatValue(x), BasicTypeEnum::FloatType(ft)) => Some(self.builder.build_float_cast(x, ft, "conv").ok()?.into()),
                    _ => {
                        self.cannot("this conversion has no shape in the generated code yet", None);
                        None
                    }
                }
            }
            // A named function as a value: its code, and nothing captured.
            Rvalue::FnRef(idx) => {
                let f = *self.fns.get(&(idx.0 as usize))?;
                Some(self.fn_value(f.as_global_value().as_pointer_value(), None))
            }
            // A literal: its code, and — when it captured — a block of the
            // addresses it borrowed. A lend cannot outlive what it borrows,
            // so the block lives in this function's frame, made once at its
            // entry (not once per trip of a loop).
            // A block of captures begins with how to give it back and how to
            // copy it — null for one that only borrows. One that owns what it
            // captured (`copy`, `give`: owner, 2026-09-24: a) is on the
            // heap, the function value's to free; one that only lends
            // lives in this frame.
            Rvalue::Closure { body, captures } => {
                let bi = body.0 as usize;
                let f = *self.fns.get(&bi)?;
                if captures.is_empty() {
                    return Some(self.fn_value(f.as_global_value().as_pointer_value(), None));
                }
                let p = self.ctx.ptr_type(AddressSpace::default());
                let st = self.env_shape(bi, captures.len())?;
                let owns = captures.iter().any(|(v, _)| matches!(v, crate::ast::Verb::Copy | crate::ast::Verb::Give));
                let env = if owns {
                    let size = self.target.get_abi_size(&st);
                    self.alloc(self.ctx.i64_type().const_int(size, false))
                } else {
                    self.entry_alloca(func, st.into(), "captures")?
                };
                let (dropper, copier) = if owns {
                    let (d, c) = self.env_helpers(bi, captures.len())?;
                    (d.as_global_value().as_pointer_value(), c.as_global_value().as_pointer_value())
                } else {
                    (p.const_null(), p.const_null())
                };
                let _ = self.builder.build_store(self.builder.build_struct_gep(st, env, 0, "drop").ok()?, dropper);
                let _ = self.builder.build_store(self.builder.build_struct_gep(st, env, 1, "copy").ok()?, copier);
                for (c, (verb, place)) in captures.iter().enumerate() {
                    let at = self.builder.build_struct_gep(st, env, 2 + c as u32, "capture").ok()?;
                    let (addr, ty) = self.slot_of(fi, place, slots)?;
                    match verb {
                        crate::ast::Verb::Lend | crate::ast::Verb::LendMut => {
                            let _ = self.builder.build_store(at, addr);
                        }
                        crate::ast::Verb::Copy => match self.copy_fn(&ty) {
                            Some(cf) => {
                                let _ = self.builder.build_call(cf, &[at.into(), addr.into()], "");
                            }
                            None => {
                                let lt = self.ty(&ty)?;
                            let v = self.builder.build_load(lt, addr, "v").ok()?;
                                let _ = self.builder.build_store(at, v);
                            }
                        },
                        crate::ast::Verb::Give => {
                            let lt = self.ty(&ty)?;
                            let v = self.builder.build_load(lt, addr, "v").ok()?;
                            let _ = self.builder.build_store(at, v);
                            if place.proj.is_empty() {
                                self.mark_alive(place.local, false);
                            }
                        }
                    }
                }
                Some(self.fn_value(f.as_global_value().as_pointer_value(), Some(env)))
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
            Rvalue::MakeChannel { elem, capacity } => self.make_channel(elem, *capacity),
            Rvalue::MakeMap(pairs) => {
                let ty = want_ty?;
                self.make_map(fi, pairs, &ty, slots)
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
            // Reading C's variable: a nocheck site, a plain load.
            Rvalue::ExternRead(id) => {
                let g = self.extern_global(id.0)?;
                let t = self.ty(&self.p.items.externs[id.0 as usize].ret.clone())?;
                self.builder.build_load(t, g, "extern").ok()
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn binary(&mut self, fi: usize, op: &BinOp, a: BasicValueEnum<'ctx>, b: BasicValueEnum<'ctx>, sites: &[SiteId], func: FunctionValue<'ctx>, blocks: &[LlBlock<'ctx>], signed: bool) -> Option<BasicValueEnum<'ctx>> {
        // The guard of each kind that still needs its check: not proven,
        // not trusted.
        let open = |this: &Self, want: &dyn Fn(&SiteKind) -> bool| -> Option<SiteId> {
            sites.iter().copied().find(|s| {
                let site = this.p.functions[fi].site(*s);
                want(&site.kind) && this.gets_check(&site.verdict)
            })
        };
        let overflow = open(self, &|k| matches!(k, SiteKind::Overflow(_)));
        let divzero = open(self, &|k| matches!(k, SiteKind::DivZero | SiteKind::ModZero));
        let shift = open(self, &|k| matches!(k, SiteKind::ShiftWidth | SiteKind::ShiftCount));
        let needs_check = overflow.is_some();
        let site = overflow;
        // A promise to LLVM only where Precog proved it, and not when the
        // proofs are being checked. A `nocheck` site proved nothing: the
        // writer's word is not a fact to hand the optimiser (the module
        // comment above), and giving it `nsw` made a lie undefined.
        let promise = !self.assert_proofs
            && sites.iter().any(|s| {
                let st = self.p.functions[fi].site(*s);
                matches!(st.kind, SiteKind::Overflow(_)) && matches!(st.verdict, Verdict::Proven { .. })
            });
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
                // The literal is the one that moves, whichever is wider: a
                // `uint64` times 10^19 is a `uint64` multiply, and the
                // literal's 128-bit form is only how it was written down.
                // Widening the other side instead checked the product at
                // 128 bits, where it never overflows, and then truncated it.
                if y.is_const() && !x.is_const() {
                    (a, self.fit_as(b, x.get_type().into(), signed))
                } else if x.is_const() && !y.is_const() {
                    (self.fit_as(a, y.get_type().into(), signed), b)
                } else if x.get_type().get_bit_width() > y.get_type().get_bit_width() {
                    (a, self.fit_as(b, x.get_type().into(), signed))
                } else {
                    (self.fit_as(a, y.get_type().into(), signed), b)
                }
            }
            _ => (a, b),
        };
        if let (BasicValueEnum::IntValue(x), BasicValueEnum::IntValue(y)) = (a, b) {
            // Powers are a loop, not an instruction.
            if matches!(op, BinOp::Pow | BinOp::WrapPow) {
                let y = if y.get_type() == x.get_type() { y } else { self.builder.build_int_cast_sign_flag(y, x.get_type(), signed, "exp").ok()? };
                return self.ipow(fi, x, y, signed, if *op == BinOp::Pow { site } else { None }, func).map(|v| v.into());
            }
            let (lt, le, gt, ge) = if signed {
                (IntPredicate::SLT, IntPredicate::SLE, IntPredicate::SGT, IntPredicate::SGE)
            } else {
                (IntPredicate::ULT, IntPredicate::ULE, IntPredicate::UGT, IntPredicate::UGE)
            };
            let r = match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul if needs_check => {
                    let name = match (op, signed) {
                        (BinOp::Add, true) => "llvm.sadd.with.overflow",
                        (BinOp::Sub, true) => "llvm.ssub.with.overflow",
                        (_, true) => "llvm.smul.with.overflow",
                        (BinOp::Add, false) => "llvm.uadd.with.overflow",
                        (BinOp::Sub, false) => "llvm.usub.with.overflow",
                        (_, false) => "llvm.umul.with.overflow",
                    };
                    return self.checked(fi, name, x, y, site.unwrap(), func, blocks);
                }
                // Proven not to wrap: the fact goes to LLVM, which is the
                // whole point of proving it (`precog.md`) — `nsw` for a
                // signed type, `nuw` for an unsigned one. Telling LLVM
                // "no signed wrap" about unsigned arithmetic is a promise
                // about the wrong thing.
                BinOp::Add if !promise => self.builder.build_int_add(x, y, "add").ok()?,
                BinOp::Sub if !promise => self.builder.build_int_sub(x, y, "sub").ok()?,
                BinOp::Mul if !promise => self.builder.build_int_mul(x, y, "mul").ok()?,
                BinOp::Add if signed => self.builder.build_int_nsw_add(x, y, "add").ok()?,
                BinOp::Sub if signed => self.builder.build_int_nsw_sub(x, y, "sub").ok()?,
                BinOp::Mul if signed => self.builder.build_int_nsw_mul(x, y, "mul").ok()?,
                BinOp::Add => self.builder.build_int_nuw_add(x, y, "add").ok()?,
                BinOp::Sub => self.builder.build_int_nuw_sub(x, y, "sub").ok()?,
                BinOp::Mul => self.builder.build_int_nuw_mul(x, y, "mul").ok()?,
                BinOp::WrapAdd => self.builder.build_int_add(x, y, "wadd").ok()?,
                BinOp::WrapSub => self.builder.build_int_sub(x, y, "wsub").ok()?,
                BinOp::WrapMul => self.builder.build_int_mul(x, y, "wmul").ok()?,
                // `/` floors and `mod` takes the divisor's sign
                // (`operators.md`): `-7 / 2 = -4`, `-7 mod 2 = 1`. Unsigned,
                // the hardware's answer already is that.
                BinOp::Div | BinOp::Mod => {
                    // A zero divisor, and the lowest signed value divided by
                    // -1, are both undefined to LLVM: checked before the
                    // divide, never after.
                    if let Some(s) = divzero {
                        let zero = self.builder.build_int_compare(IntPredicate::EQ, y, y.get_type().const_zero(), "by_zero").ok()?;
                        self.guard(fi, s, zero, func)?;
                    }
                    if let (true, Some(s)) = (signed, site) {
                        let bits = x.get_type().get_bit_width();
                        // The lowest value at this width, any width.
                        let min = const_i128(x.get_type(), if bits >= 128 { i128::MIN } else { -(1i128 << (bits - 1)) });
                        let is_min = self.builder.build_int_compare(IntPredicate::EQ, x, min, "is_min").ok()?;
                        let by_m1 = self.builder.build_int_compare(IntPredicate::EQ, y, y.get_type().const_all_ones(), "by_m1").ok()?;
                        let both = self.builder.build_and(is_min, by_m1, "min_by_m1").ok()?;
                        self.guard(fi, s, both, func)?;
                    }
                    match (op, signed) {
                        (BinOp::Div, false) => self.builder.build_int_unsigned_div(x, y, "div").ok()?,
                        (_, false) => self.builder.build_int_unsigned_rem(x, y, "rem").ok()?,
                        (BinOp::Div, true) => self.floor_div(x, y, false)?,
                        _ => self.floor_div(x, y, true)?,
                    }
                }
                BinOp::AndBit | BinOp::And => self.builder.build_and(x, y, "and").ok()?,
                BinOp::OrBit | BinOp::Or => self.builder.build_or(x, y, "or").ok()?,
                BinOp::Xor => self.builder.build_xor(x, y, "xor").ok()?,
                BinOp::Shl | BinOp::Shr => {
                    // Shifting by the width or more is poison to LLVM.
                    if let Some(s) = shift {
                        let width = y.get_type().const_int(x.get_type().get_bit_width() as u64, false);
                        let past = self.builder.build_int_compare(IntPredicate::UGE, y, width, "past_width").ok()?;
                        self.guard(fi, s, past, func)?;
                    }
                    if *op == BinOp::Shl {
                        let out = self.builder.build_left_shift(x, y, "shl").ok()?;
                        // "`shl` past the width or into overflow is a check
                        // site" (`operators.md`): a bit shifted out is lost,
                        // and shifting back does not give the value again.
                        if let Some(s) = shift {
                            let back = self.builder.build_right_shift(out, y, signed, "back").ok()?;
                            let lost = self.builder.build_int_compare(IntPredicate::NE, back, x, "bits_lost").ok()?;
                            self.guard(fi, s, lost, func)?;
                        }
                        out
                    } else {
                        self.builder.build_right_shift(x, y, signed, "shr").ok()?
                    }
                }
                BinOp::Eq => self.builder.build_int_compare(IntPredicate::EQ, x, y, "eq").ok()?,
                BinOp::Ne => self.builder.build_int_compare(IntPredicate::NE, x, y, "ne").ok()?,
                BinOp::Lt => self.builder.build_int_compare(lt, x, y, "lt").ok()?,
                BinOp::Le => self.builder.build_int_compare(le, x, y, "le").ok()?,
                BinOp::Gt => self.builder.build_int_compare(gt, x, y, "gt").ok()?,
                BinOp::Ge => self.builder.build_int_compare(ge, x, y, "ge").ok()?,
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
        // Never skip an instruction quietly: whatever reaches here would
        // leave its destination unwritten.
        self.cannot(format!("`{}` on these values has no shape in the generated code yet", op.spelled()), None);
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
        let (msg, handler) = self.site_message(fi, site);
        let ptr = self.global_bytes(&msg);
        let len = self.ctx.i64_type().const_int(msg.len() as u64, false);
        if let Some(handler) = self.module.get_function(handler) {
            let args: Vec<BasicMetadataValueEnum> = vec![ptr.into(), len.into()];
            let _ = self.builder.build_call(handler, &args, "");
        }
        let _ = self.builder.build_unreachable();
    }

    /// What a trap at this site says, and which runtime entry says it.
    pub(super) fn site_message(&self, fi: usize, site: SiteId) -> (String, &'static str) {
        let f = &self.p.functions[fi];
        let s = f.site(site);
        let span = f.span(s.span);
        let file = self.sources.get(span.file);
        let (line, col) = file.line_col(span.start);
        let spell = |t: &Ty| crate::types::TySpelling { ty: t, structs: &self.p.items.structs, enums: &self.p.items.enums }.to_string();
        // A proven site only traps in the proof-checking build, and when it
        // does, it is Precog that was wrong, not the program.
        let proof = matches!(s.verdict, Verdict::Proven { .. });
        let (head, handler) = if proof { ("proof failed", "cyborg_proof_failed") } else { ("check", "cyborg_check_failed") };
        (format!("{}: {}\nwhere: {}, line {}, column {}\n", head, s.sentence(&spell), file.display_name(), line, col), handler)
    }

    pub(super) fn global_bytes(&mut self, s: &str) -> PointerValue<'ctx> {
        if let Some(p) = self.strings.get(s) {
            return *p;
        }
        let g = self.builder.build_global_string_ptr(s, "msg").unwrap();
        let p = g.as_pointer_value();
        self.strings.insert(s.to_string(), p);
        p
    }

    /// A value at the width the place it is going has. A literal has no
    /// width of its own until it lands somewhere (`types.md`). This one
    /// assumes a signed value — right for a literal, which is built signed;
    /// anything else says which it is through `fit_as`.
    fn fit(&mut self, v: BasicValueEnum<'ctx>, want: BasicTypeEnum<'ctx>) -> BasicValueEnum<'ctx> {
        self.fit_as(v, want, true)
    }

    /// Floor division and the mathematician's `mod`, from the hardware's
    /// truncating pair: when the remainder is not zero and its sign differs
    /// from the divisor's, the quotient is one lower and the remainder moves
    /// by the divisor. A few one-cycle instructions beside the divide
    /// (`operators.md`: "the fix-up is the operator's meaning").
    /// "In between, the one limit is the machine's 64 bits" (`operators.md`):
    /// an exact answer below `int64`'s lowest or above `uint64`'s highest
    /// stops here, unless Precog proved it never is.
    fn past_64_bits(&mut self, fi: usize, r: IntValue<'ctx>, sites: &[SiteId], func: FunctionValue<'ctx>) {
        let open = sites.iter().copied().find(|s| {
            let site = self.p.functions[fi].site(*s);
            matches!(site.kind, SiteKind::Overflow(_)) && self.gets_check(&site.verdict)
        });
        let Some(s) = open else { return };
        let t = r.get_type();
        let (Ok(low), Ok(high)) = (
            self.builder.build_int_compare(IntPredicate::SLT, r, const_i128(t, i64::MIN as i128), "below"),
            self.builder.build_int_compare(IntPredicate::SGT, r, const_i128(t, u64::MAX as i128), "above"),
        ) else { return };
        let Ok(out) = self.builder.build_or(low, high, "past_64") else { return };
        let _ = self.guard(fi, s, out, func);
    }

    fn floor_div(&mut self, x: IntValue<'ctx>, y: IntValue<'ctx>, want_mod: bool) -> Option<IntValue<'ctx>> {
        let q = self.builder.build_int_signed_div(x, y, "q").ok()?;
        let r = self.builder.build_int_signed_rem(x, y, "r").ok()?;
        let zero = x.get_type().const_zero();
        let nonzero = self.builder.build_int_compare(IntPredicate::NE, r, zero, "inexact").ok()?;
        let r_neg = self.builder.build_int_compare(IntPredicate::SLT, r, zero, "rneg").ok()?;
        let y_neg = self.builder.build_int_compare(IntPredicate::SLT, y, zero, "yneg").ok()?;
        let differ = self.builder.build_xor(r_neg, y_neg, "differ").ok()?;
        let fix = self.builder.build_and(nonzero, differ, "fix").ok()?;
        if want_mod {
            let moved = self.builder.build_int_add(r, y, "moved").ok()?;
            Some(self.builder.build_select(fix, moved, r, "mod").ok()?.into_int_value())
        } else {
            let one = x.get_type().const_int(1, false);
            let lower = self.builder.build_int_sub(q, one, "lower").ok()?;
            Some(self.builder.build_select(fix, lower, q, "div").ok()?.into_int_value())
        }
    }

    /// Is the value of this operand a signed number? `None` for a literal,
    /// which has no family of its own.
    fn operand_signed(&mut self, fi: usize, o: &Operand) -> Option<bool> {
        let p = o.place()?;
        let f = &self.p.functions[fi];
        let base = f.locals[p.local.0 as usize].ty.clone();
        let t = crate::mir::build::projected_ty(&base, &p.proj, &self.p.items);
        int_signed(&t)
    }

    /// C's variable for an `extern let`, declared once: external linkage,
    /// no initialiser — C defines it.
    fn extern_global(&mut self, id: u32) -> Option<PointerValue<'ctx>> {
        let info = self.p.items.externs.get(id as usize)?.clone();
        if let Some(g) = self.module.get_global(&info.name) {
            return Some(g.as_pointer_value());
        }
        let t = self.ty(&info.ret)?;
        let g = self.module.add_global(t, None, &info.name);
        g.set_linkage(inkwell::module::Linkage::External);
        Some(g.as_pointer_value())
    }

    /// A vector's operator, lane by lane (`simd.md`): the scalar rule N
    /// times, each check asked of every lane at once. A scalar beside a
    /// vector stands for every lane.
    #[allow(clippy::too_many_arguments)]
    fn vector_binary(&mut self, fi: usize, op: &BinOp, a: BasicValueEnum<'ctx>, b: BasicValueEnum<'ctx>, lane: Option<Ty>, sites: &[SiteId], func: FunctionValue<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        let vt = if a.is_vector_value() { a.into_vector_value().get_type() } else { b.into_vector_value().get_type() };
        let splat = |this: &mut Self, v: BasicValueEnum<'ctx>| -> Option<inkwell::values::VectorValue<'ctx>> {
            if v.is_vector_value() {
                return Some(v.into_vector_value());
            }
            let v = this.fit(v, vt.get_element_type());
            let mut out = vt.get_undef();
            for k in 0..vt.get_size() {
                out = this.builder.build_insert_element(out, v, this.ctx.i32_type().const_int(k as u64, false), "lane").ok()?;
            }
            Some(out)
        };
        let x = splat(self, a)?;
        let y = splat(self, b)?;
        let open = |this: &Self, want: &dyn Fn(&SiteKind) -> bool| sites.iter().copied().find(|s| want(&this.p.functions[fi].site(*s).kind) && this.gets_check(&this.p.functions[fi].site(*s).verdict));
        let any = |this: &mut Self, m: inkwell::values::VectorValue<'ctx>| -> Option<IntValue<'ctx>> {
            let id = inkwell::intrinsics::Intrinsic::find("llvm.vector.reduce.or")?;
            let f = id.get_declaration(this.module, &[m.get_type().into()])?;
            Some(basic_result(this.builder.build_call(f, &[m.into()], "any_lane").ok()?)?.into_int_value())
        };
        if vt.get_element_type().is_float_type() {
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
                    self.cannot(format!("`{}` on vectors of bins has no shape in the generated code yet", op.spelled()), None);
                    return None;
                }
            };
            return Some(r);
        }
        let signed = lane.as_ref().and_then(int_signed).unwrap_or(true);
        let bits = vt.get_element_type().into_int_type().get_bit_width();
        let (lt, le, gt, ge) = if signed {
            (IntPredicate::SLT, IntPredicate::SLE, IntPredicate::SGT, IntPredicate::SGE)
        } else {
            (IntPredicate::ULT, IntPredicate::ULE, IntPredicate::UGT, IntPredicate::UGE)
        };
        let zero = vt.const_zero();
        let r: BasicValueEnum = match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul => {
                if let Some(s) = open(self, &|k| matches!(k, SiteKind::Overflow(_))) {
                    let name = match (op, signed) {
                        (BinOp::Add, true) => "llvm.sadd.with.overflow",
                        (BinOp::Add, false) => "llvm.uadd.with.overflow",
                        (BinOp::Sub, true) => "llvm.ssub.with.overflow",
                        (BinOp::Sub, false) => "llvm.usub.with.overflow",
                        (_, true) => "llvm.smul.with.overflow",
                        (_, false) => "llvm.umul.with.overflow",
                    };
                    let id = inkwell::intrinsics::Intrinsic::find(name)?;
                    let f = id.get_declaration(self.module, &[vt.into()])?;
                    let agg = basic_result(self.builder.build_call(f, &[x.into(), y.into()], "chk").ok()?)?.into_struct_value();
                    let v = self.builder.build_extract_value(agg, 0, "v").ok()?;
                    let flags = self.builder.build_extract_value(agg, 1, "o").ok()?.into_vector_value();
                    let bad = any(self, flags)?;
                    self.guard(fi, s, bad, func)?;
                    v
                } else {
                    match op {
                        BinOp::Add => self.builder.build_int_add(x, y, "add").ok()?.into(),
                        BinOp::Sub => self.builder.build_int_sub(x, y, "sub").ok()?.into(),
                        _ => self.builder.build_int_mul(x, y, "mul").ok()?.into(),
                    }
                }
            }
            BinOp::WrapAdd => self.builder.build_int_add(x, y, "wadd").ok()?.into(),
            BinOp::WrapSub => self.builder.build_int_sub(x, y, "wsub").ok()?.into(),
            BinOp::WrapMul => self.builder.build_int_mul(x, y, "wmul").ok()?.into(),
            BinOp::Div | BinOp::Mod => {
                if let Some(s) = open(self, &|k| matches!(k, SiteKind::DivZero | SiteKind::ModZero)) {
                    let z = self.builder.build_int_compare(IntPredicate::EQ, y, zero, "by_zero").ok()?;
                    let bad = any(self, z)?;
                    self.guard(fi, s, bad, func)?;
                }
                if signed {
                    if let Some(s) = open(self, &|k| matches!(k, SiteKind::Overflow(BinOp::Div))) {
                        let min = const_i128(vt.get_element_type().into_int_type(), -(1i128 << (bits - 1)));
                        let mins = splat(self, min.into())?;
                        let m1 = splat(self, vt.get_element_type().into_int_type().const_all_ones().into())?;
                        let is_min = self.builder.build_int_compare(IntPredicate::EQ, x, mins, "is_min").ok()?;
                        let by_m1 = self.builder.build_int_compare(IntPredicate::EQ, y, m1, "by_m1").ok()?;
                        let both = self.builder.build_and(is_min, by_m1, "min_by_m1").ok()?;
                        let bad = any(self, both)?;
                        self.guard(fi, s, bad, func)?;
                    }
                    // Floored, lane by lane (`operators.md`).
                    let q = self.builder.build_int_signed_div(x, y, "q").ok()?;
                    let rm = self.builder.build_int_signed_rem(x, y, "r").ok()?;
                    let nz = self.builder.build_int_compare(IntPredicate::NE, rm, zero, "inexact").ok()?;
                    let rn = self.builder.build_int_compare(IntPredicate::SLT, rm, zero, "rneg").ok()?;
                    let yn = self.builder.build_int_compare(IntPredicate::SLT, y, zero, "yneg").ok()?;
                    let differ = self.builder.build_xor(rn, yn, "differ").ok()?;
                    let fix = self.builder.build_and(nz, differ, "fix").ok()?;
                    if *op == BinOp::Div {
                        let one = splat(self, vt.get_element_type().into_int_type().const_int(1, false).into())?;
                        let lower = self.builder.build_int_sub(q, one, "lower").ok()?;
                        self.builder.build_select(fix, lower, q, "div").ok()?
                    } else {
                        let moved = self.builder.build_int_add(rm, y, "moved").ok()?;
                        self.builder.build_select(fix, moved, rm, "mod").ok()?
                    }
                } else if *op == BinOp::Div {
                    self.builder.build_int_unsigned_div(x, y, "div").ok()?.into()
                } else {
                    self.builder.build_int_unsigned_rem(x, y, "rem").ok()?.into()
                }
            }
            // Masks, lane by lane (owner, 2026-09-27: a); both sides are
            // already worked out, as `and` / `or` always are.
            BinOp::AndBit | BinOp::And => self.builder.build_and(x, y, "and").ok()?.into(),
            BinOp::OrBit | BinOp::Or => self.builder.build_or(x, y, "or").ok()?.into(),
            BinOp::Xor => self.builder.build_xor(x, y, "xor").ok()?.into(),
            BinOp::Shl | BinOp::Shr => {
                // The count is 0 to width - 1 in every lane (owner,
                // 2026-09-23); `shl` also may not push bits off the end.
                if let Some(s) = open(self, &|k| matches!(k, SiteKind::ShiftWidth | SiteKind::ShiftCount)) {
                    let w = splat(self, vt.get_element_type().into_int_type().const_int(bits as u64, false).into())?;
                    let past = self.builder.build_int_compare(IntPredicate::UGE, y, w, "past").ok()?;
                    let mut bad = any(self, past)?;
                    if *op == BinOp::Shl {
                        let shifted = self.builder.build_left_shift(x, y, "shl").ok()?;
                        let back = self.builder.build_right_shift(shifted, y, signed, "back").ok()?;
                        let lost = self.builder.build_int_compare(IntPredicate::NE, back, x, "lost").ok()?;
                        let lost = any(self, lost)?;
                        bad = self.builder.build_or(bad, lost, "shl_bad").ok()?;
                    }
                    self.guard(fi, s, bad, func)?;
                }
                if *op == BinOp::Shl {
                    self.builder.build_left_shift(x, y, "shl").ok()?.into()
                } else {
                    self.builder.build_right_shift(x, y, signed, "shr").ok()?.into()
                }
            }
            BinOp::Eq => self.builder.build_int_compare(IntPredicate::EQ, x, y, "eq").ok()?.into(),
            BinOp::Ne => self.builder.build_int_compare(IntPredicate::NE, x, y, "ne").ok()?.into(),
            BinOp::Lt => self.builder.build_int_compare(lt, x, y, "lt").ok()?.into(),
            BinOp::Le => self.builder.build_int_compare(le, x, y, "le").ok()?.into(),
            BinOp::Gt => self.builder.build_int_compare(gt, x, y, "gt").ok()?.into(),
            BinOp::Ge => self.builder.build_int_compare(ge, x, y, "ge").ok()?.into(),
            _ => {
                self.cannot(format!("`{}` on vectors has no shape in the generated code yet", op.spelled()), None);
                return None;
            }
        };
        Some(r)
    }

    /// Whether two types are the same memory: containers' elements alike,
    /// all the way down (a bound changes no layout).
    fn same_layout(&mut self, a: &Ty, b: &Ty) -> bool {
        match (a.base(), b.base()) {
            (Ty::Seq { elem: x, .. }, Ty::Seq { elem: y, .. }) | (Ty::Channel(x), Ty::Channel(y)) => {
                let (x, y) = ((**x).clone(), (**y).clone());
                self.same_layout(&x, &y)
            }
            (Ty::Map { key: k1, val: v1, .. }, Ty::Map { key: k2, val: v2, .. }) => {
                let (k1, v1, k2, v2) = ((**k1).clone(), (**v1).clone(), (**k2).clone(), (**v2).clone());
                self.same_layout(&k1, &k2) && self.same_layout(&v1, &v2)
            }
            (x, y) => {
                let (x, y) = (x.clone(), y.clone());
                self.ty(&x) == self.ty(&y)
            }
        }
    }

    /// The block a literal's captures live in: how to give it back, how to
    /// copy it, then each capture — a lent one's address, an owned one's
    /// value. The literal's first locals are its captures, in order.
    fn env_shape(&mut self, body: usize, n: usize) -> Option<inkwell::types::StructType<'ctx>> {
        let p = self.ctx.ptr_type(AddressSpace::default());
        let mut fields: Vec<BasicTypeEnum> = vec![p.into(), p.into()];
        for c in 0..n {
            let l = self.p.functions[body].locals.get(c)?.clone();
            if matches!(l.kind, LocalKind::Lent { .. }) {
                fields.push(p.into());
            } else {
                fields.push(self.ty(&l.ty)?);
            }
        }
        Some(self.ctx.struct_type(&fields, false))
    }

    /// How a block of owned captures is given back and copied: each owned
    /// capture by its own type's way, lends as the addresses they are.
    fn env_helpers(&mut self, body: usize, n: usize) -> Option<(FunctionValue<'ctx>, FunctionValue<'ctx>)> {
        let dname = format!("cy_env_drop_{}", body);
        let cname = format!("cy_env_copy_{}", body);
        if let (Some(d), Some(c)) = (self.helpers.get(&dname).copied(), self.helpers.get(&cname).copied()) {
            return Some((d, c));
        }
        let st = self.env_shape(body, n)?;
        let p = self.ctx.ptr_type(AddressSpace::default());
        let tys: Vec<(bool, Ty)> = (0..n).map(|c| {
            let l = &self.p.functions[body].locals[c];
            (matches!(l.kind, LocalKind::Lent { .. }), l.ty.clone())
        }).collect();
        let here = self.builder.get_insert_block();
        let d = self.module.add_function(&dname, self.ctx.void_type().fn_type(&[p.into()], false), Some(inkwell::module::Linkage::Internal));
        let c = self.module.add_function(&cname, p.fn_type(&[p.into()], false), Some(inkwell::module::Linkage::Internal));
        self.helpers.insert(dname, d);
        self.helpers.insert(cname, c);
        // Give back: each owned capture, then the block.
        let entry = self.ctx.append_basic_block(d, "entry");
        self.builder.position_at_end(entry);
        let env = d.get_nth_param(0)?.into_pointer_value();
        for (k, (lent, ty)) in tys.iter().enumerate() {
            if *lent {
                continue;
            }
            if let Some(inner) = self.drop_fn(ty) {
                let at = self.builder.build_struct_gep(st, env, 2 + k as u32, "capture").ok()?;
                let _ = self.builder.build_call(inner, &[at.into()], "");
            }
        }
        let free = self.rt("cyborg_free");
        let _ = self.builder.build_call(free, &[env.into()], "");
        let _ = self.builder.build_return(None);
        // Copy: a new block, each owned capture a second value of its own.
        let entry = self.ctx.append_basic_block(c, "entry");
        self.builder.position_at_end(entry);
        let src = c.get_nth_param(0)?.into_pointer_value();
        let size = self.target.get_abi_size(&st);
        let dst = self.alloc(self.ctx.i64_type().const_int(size, false));
        for k in 0..2 {
            let s = self.builder.build_struct_gep(st, src, k, "h").ok()?;
            let v = self.builder.build_load(p, s, "h").ok()?;
            let _ = self.builder.build_store(self.builder.build_struct_gep(st, dst, k, "h").ok()?, v);
        }
        for (k, (lent, ty)) in tys.iter().enumerate() {
            let s = self.builder.build_struct_gep(st, src, 2 + k as u32, "s").ok()?;
            let t = self.builder.build_struct_gep(st, dst, 2 + k as u32, "d").ok()?;
            match (lent, self.copy_fn(ty)) {
                (false, Some(cf)) => {
                    let _ = self.builder.build_call(cf, &[t.into(), s.into()], "");
                }
                _ => {
                    let lt: BasicTypeEnum = if *lent { p.into() } else { self.ty(ty)? };
                    let v = self.builder.build_load(lt, s, "v").ok()?;
                    let _ = self.builder.build_store(t, v);
                }
            }
        }
        let _ = self.builder.build_return(Some(&dst));
        if let Some(b) = here {
            self.builder.position_at_end(b);
        }
        Some((d, c))
    }

    /// `copy 'x'` handed to a call: a second, independent value — the
    /// callee owns it and gives it back. Read as the same bytes, the callee
    /// freed the caller's own list.
    pub(super) fn copied(&mut self, fi: usize, p: &Place, slots: &[Option<PointerValue<'ctx>>]) -> Option<BasicValueEnum<'ctx>> {
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

    /// `nocheck inline.ASM { … } with [ … ]` (`ffi.md`): the text verbatim
    /// for LLVM's assembler, each `->` binding copied into its register
    /// before, each `<-` stored from its register after, each `clobber`
    /// said. x29 and x30 are spelled `fp` and `lr` in what LLVM gets (the
    /// compiler rule of 2026-09-21: LLVM 22 ignores their `xN` clobbers).
    /// Memory and the flags are always marked written — a syscall may write
    /// through a pointer it was handed — which costs optimisation, never
    /// correctness. Whole registers: a value goes in widened to 64 bits by
    /// its own sign, and comes out narrowed to its place.
    fn inline_asm(&mut self, fi: usize, text: &str, ins: &[(Place, String)], outs: &[(Place, String)], clobbers: &[String], slots: &[Option<PointerValue<'ctx>>]) -> Option<()> {
        let abi = |r: &str| -> String {
            match r {
                "x29" | "w29" => "fp".into(),
                "x30" | "w30" => "lr".into(),
                other => other.to_string(),
            }
        };
        let reg_ty = |this: &Self, r: &str| -> BasicTypeEnum<'ctx> {
            if r.starts_with('w') { this.ctx.i32_type().into() } else { this.ctx.i64_type().into() }
        };
        let mut constraints: Vec<String> = Vec::new();
        let mut out_tys: Vec<BasicTypeEnum> = Vec::new();
        for (_, r) in outs {
            constraints.push(format!("={{{}}}", abi(r)));
            out_tys.push(reg_ty(self, r));
        }
        let mut args: Vec<BasicMetadataValueEnum> = Vec::new();
        let mut arg_tys: Vec<BasicMetadataTypeEnum> = Vec::new();
        for (p, r) in ins {
            constraints.push(format!("{{{}}}", abi(r)));
            let v = self.operand(fi, &Operand::Read(p.clone()), slots)?;
            let signed = self.operand_signed(fi, &Operand::Read(p.clone())).unwrap_or(true);
            let t = reg_ty(self, r);
            let v = match v {
                BasicValueEnum::PointerValue(pv) if t.is_int_type() => self.builder.build_ptr_to_int(pv, t.into_int_type(), "addr").ok()?.into(),
                other => self.fit_as(other, t, signed),
            };
            args.push(v.into());
            arg_tys.push(t.into());
        }
        for c in clobbers {
            constraints.push(format!("~{{{}}}", abi(c)));
        }
        constraints.push("~{memory}".into());
        constraints.push("~{cc}".into());
        let ret: Option<BasicTypeEnum> = match out_tys.len() {
            0 => None,
            1 => Some(out_tys[0]),
            _ => Some(self.ctx.struct_type(&out_tys, false).into()),
        };
        let fty = match ret {
            Some(r) => r.fn_type(&arg_tys, false),
            None => self.ctx.void_type().fn_type(&arg_tys, false),
        };
        let asm = self.ctx.create_inline_asm(fty, text.replace('$', "$$"), constraints.join(","), true, false, None, false);
        let call = self.builder.build_indirect_call(fty, asm, &args, "asm").ok()?;
        let result = basic_result(call);
        for (i, (p, _)) in outs.iter().enumerate() {
            let v = match (&result, out_tys.len()) {
                (Some(v), 1) => *v,
                (Some(BasicValueEnum::StructValue(s)), _) => self.builder.build_extract_value(*s, i as u32, "out").ok()?,
                _ => return None,
            };
            let (_, ty) = self.slot_of(fi, p, slots)?;
            let want = self.ty(&ty)?;
            let v = match want {
                BasicTypeEnum::PointerType(pt) => self.builder.build_int_to_ptr(v.into_int_value(), pt, "ptr").ok()?.into(),
                other => self.fit_as(v, other, true),
            };
            self.store_place(fi, p, v, slots);
        }
        Some(())
    }

    /// A function value: code and captures (null for none).
    fn fn_value(&mut self, code: PointerValue<'ctx>, env: Option<PointerValue<'ctx>>) -> BasicValueEnum<'ctx> {
        let p = self.ctx.ptr_type(AddressSpace::default());
        let st = self.ctx.struct_type(&[p.into(), p.into()], false);
        let env = env.unwrap_or_else(|| p.const_null());
        let v = st.get_undef();
        let v = self.builder.build_insert_value(v, code, 0, "code").unwrap();
        self.builder.build_insert_value(v, env, 1, "env").unwrap().into_struct_value().into()
    }

    /// Room on the stack made at the function's entry, so a loop does not
    /// make it again on every trip.
    fn entry_alloca(&mut self, func: FunctionValue<'ctx>, t: BasicTypeEnum<'ctx>, name: &str) -> Option<PointerValue<'ctx>> {
        let entry = func.get_first_basic_block()?;
        let b = self.ctx.create_builder();
        match entry.get_first_instruction() {
            Some(i) => b.position_before(&i),
            None => b.position_at_end(entry),
        }
        b.build_alloca(t, name).ok()
    }

    /// Calling a function held as a value, with the arguments already made:
    /// with no captures its code is the plain function; with some, the
    /// block of them goes first. Which one is known only at run time.
    fn call_held(&mut self, fv: BasicValueEnum<'ctx>, ft: &crate::types::FnTy, vals: &mut Vec<BasicMetadataValueEnum<'ctx>>, signs: &[bool]) -> Option<Option<BasicValueEnum<'ctx>>> {
        let p = self.ctx.ptr_type(AddressSpace::default());
        let mut params: Vec<BasicMetadataTypeEnum> = Vec::new();
        for (k, t) in &ft.params {
            if matches!(k, crate::types::ParamKind::Lent | crate::types::ParamKind::LentMut) {
                params.push(p.into());
            } else {
                params.push(self.ty(t)?.into());
            }
        }
        // Each argument at the parameter's width, and a value handed to a
        // lent parameter given a place to point at — as a direct call does.
        for (i, w) in params.iter().enumerate() {
            let Some(slot) = vals.get(i).copied() else { continue };
            let (Ok(v), Ok(t)) = (BasicValueEnum::try_from(slot), BasicTypeEnum::try_from(*w)) else { continue };
            if t.is_pointer_type() && !v.is_pointer_value() {
                let tmp = self.builder.build_alloca(v.get_type(), "lent_value").ok()?;
                let _ = self.builder.build_store(tmp, v);
                vals[i] = tmp.into();
                continue;
            }
            vals[i] = self.fit_as(v, t, signs.get(i).copied().unwrap_or(true)).into();
        }
        // A function that can fail hands back its variant (answer or reason).
        let ret: Option<BasicTypeEnum> = if !ft.fails.is_empty() {
            Some(self.fn_ty_fail_shape(ft)?.into())
        } else if ft.ret == Ty::Nothing {
            None
        } else {
            Some(self.ty(&ft.ret)?)
        };
        let mut with_env = vec![p.into()];
        with_env.extend(params.iter().copied());
        let (plain_t, env_t) = match ret {
            Some(r) => (r.fn_type(&params, false), r.fn_type(&with_env, false)),
            None => (self.ctx.void_type().fn_type(&params, false), self.ctx.void_type().fn_type(&with_env, false)),
        };
        let s = fv.into_struct_value();
        let code = self.builder.build_extract_value(s, 0, "code").ok()?.into_pointer_value();
        let env = self.builder.build_extract_value(s, 1, "env").ok()?.into_pointer_value();
        let func = self.builder.get_insert_block()?.get_parent()?;
        let bare = self.ctx.append_basic_block(func, "no_captures");
        let captured = self.ctx.append_basic_block(func, "with_captures");
        let join = self.ctx.append_basic_block(func, "called");
        let none = self.builder.build_is_null(env, "no_env").ok()?;
        let _ = self.builder.build_conditional_branch(none, bare, captured);
        self.builder.position_at_end(bare);
        let a = self.builder.build_indirect_call(plain_t, code, vals, "call").ok()?;
        let _ = self.builder.build_unconditional_branch(join);
        self.builder.position_at_end(captured);
        let mut ev: Vec<BasicMetadataValueEnum> = vec![env.into()];
        ev.extend(vals.iter().copied());
        let b = self.builder.build_indirect_call(env_t, code, &ev, "call").ok()?;
        let _ = self.builder.build_unconditional_branch(join);
        self.builder.position_at_end(join);
        let Some(r) = ret else { return Some(None) };
        let phi = self.builder.build_phi(r, "held").ok()?;
        let (Some(av), Some(bv)) = (basic_result(a), basic_result(b)) else { return None };
        phi.add_incoming(&[(&av, bare), (&bv, captured)]);
        Some(Some(phi.as_basic_value()))
    }

    /// The Cyborg type of an operand held in a place.
    fn operand_cy_ty(&mut self, fi: usize, o: &Operand) -> Option<Ty> {
        if let Operand::Const(Const::Deci { bits, .. }) = o {
            return Some(Ty::Deci(*bits));
        }
        let p = o.place()?;
        let f = &self.p.functions[fi];
        let base = f.locals[p.local.0 as usize].ty.clone();
        Some(crate::mir::build::projected_ty(&base, &p.proj, &self.p.items))
    }

    /// The lane type of a `simd[T, N]` operand.
    fn operand_lane(&mut self, fi: usize, o: &Operand) -> Option<Ty> {
        let p = o.place()?;
        let f = &self.p.functions[fi];
        let base = f.locals[p.local.0 as usize].ty.clone();
        match crate::mir::build::projected_ty(&base, &p.proj, &self.p.items).base() {
            Ty::Simd { elem, .. } => Some((**elem).clone()),
            _ => None,
        }
    }

    /// `simd.sum` is a check site (`simd.md`): the lanes are added at twice
    /// their width, where N of them cannot overflow, and the answer is
    /// checked against T unless Precog proved it fits.
    fn simd_sum(&mut self, fi: usize, v: inkwell::values::VectorValue<'ctx>, signed: bool, func: FunctionValue<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        let et = v.get_type().get_element_type().into_int_type();
        let bits = et.get_bit_width();
        let wide_t = self.ctx.custom_width_int_type(std::num::NonZeroU32::new((bits * 2).min(128))?).ok()?;
        let wide_v = wide_t.vec_type(v.get_type().get_size());
        let wide = if signed { self.builder.build_int_s_extend(v, wide_v, "wide").ok()? } else { self.builder.build_int_z_extend(v, wide_v, "wide").ok()? };
        let id = inkwell::intrinsics::Intrinsic::find("llvm.vector.reduce.add")?;
        let f = id.get_declaration(self.module, &[wide_v.into()])?;
        let total = basic_result(self.builder.build_call(f, &[wide.into()], "sum").ok()?)?.into_int_value();
        let site = self.op_sites.iter().copied().find(|s| matches!(self.p.functions[fi].site(*s).kind, SiteKind::Overflow(_)));
        if let Some(s) = site {
            if self.gets_check(&self.p.functions[fi].site(s).verdict.clone()) {
                let (lo, hi) = crate::resolve::int_range(signed, bits as u16);
                let below = self.builder.build_int_compare(if signed { IntPredicate::SLT } else { IntPredicate::ULT }, total, const_i128(wide_t, lo), "below").ok()?;
                let above = self.builder.build_int_compare(if signed { IntPredicate::SGT } else { IntPredicate::UGT }, total, const_i128(wide_t, hi), "above").ok()?;
                let bad = self.builder.build_or(below, above, "sum_out").ok()?;
                self.guard(fi, s, bad, func)?;
            }
        }
        Some(self.builder.build_int_truncate(total, et, "sum").ok()?.into())
    }

    /// The `T` of a `ptr[T]` operand.
    fn pointee(&mut self, fi: usize, o: &Operand) -> Option<Ty> {
        let p = o.place()?;
        let f = &self.p.functions[fi];
        let base = f.locals[p.local.0 as usize].ty.clone();
        match crate::mir::build::projected_ty(&base, &p.proj, &self.p.items).base() {
            Ty::Ptr { elem, .. } => Some((**elem).clone()),
            _ => None,
        }
    }

    /// A value at another width, extended by the sign it has.
    fn fit_as(&mut self, v: BasicValueEnum<'ctx>, want: BasicTypeEnum<'ctx>, signed: bool) -> BasicValueEnum<'ctx> {
        match (v, want) {
            (BasicValueEnum::IntValue(i), BasicTypeEnum::IntType(t)) if i.get_type().get_bit_width() != t.get_bit_width() => {
                self.builder.build_int_cast_sign_flag(i, t, signed, "fit").map(|x| x.into()).unwrap_or(v)
            }
            (BasicValueEnum::IntValue(i), BasicTypeEnum::FloatType(t)) => self.builder.build_signed_int_to_float(i, t, "fit").map(|x| x.into()).unwrap_or(v),
            (BasicValueEnum::FloatValue(x), BasicTypeEnum::FloatType(t)) if x.get_type() != t => self.builder.build_float_cast(x, t, "fit").map(|y| y.into()).unwrap_or(v),
            _ => v,
        }
    }

    fn call(&mut self, fi: usize, callee: &Callee, args: &[Arg], slots: &[Option<PointerValue<'ctx>>]) -> Option<BasicValueEnum<'ctx>> {
        let mut vals: Vec<BasicMetadataValueEnum> = Vec::new();
        // Each argument's own sign, for widening it to the parameter.
        let signs: Vec<bool> = args
            .iter()
            .map(|a| match a {
                Arg::Value(o) => self.operand_signed(fi, o).unwrap_or(true),
                Arg::Copy(p) | Arg::Give(p) => self.operand_signed(fi, &Operand::Read(p.clone())).unwrap_or(true),
                _ => true,
            })
            .collect();
        // A function that carries what it captured, handed over whole: the
        // parameter owns it in the callee's instance (owner, 2026-09-24: a)
        // and gives it back. Anywhere else — a held function's parameter, a
        // bundle — it is not built yet. A lend is fine: nothing changes hands.
        let owning_params: Vec<bool> = match callee {
            Callee::Fn(idx) => {
                let cf = &self.p.functions[idx.0 as usize];
                cf.params.iter().map(|p| matches!(cf.locals[p.0 as usize].ty.base(), Ty::Fn(f) if f.owning)).collect()
            }
            _ => Vec::new(),
        };
        for (ai, a) in args.iter().enumerate() {
            if owning_params.get(ai).copied().unwrap_or(false) {
                continue;
            }
            let place = match a {
                Arg::Give(p) | Arg::Copy(p) => p,
                Arg::Value(o) => match o.place() {
                    Some(p) => p,
                    None => continue,
                },
                _ => continue,
            };
            if let Some((_, t)) = self.slot_of(fi, place, slots) {
                if matches!(t.base(), Ty::Fn(f) if f.owning) {
                    self.cannot("handing a function that carries what it captured (`copy`, `give`) to a parameter has no shape in the generated code yet: whether the parameter owns it is asked", None);
                    return None;
                }
            }
        }
        let variadic = match callee {
            Callee::Fn(idx) if self.p.functions[idx.0 as usize].variadic => Some(idx.0 as usize),
            _ => None,
        };
        let mut frees = Vec::new();
        if let Some(ci) = variadic {
            let (v, f) = self.variadic_args(fi, ci, args, slots)?;
            vals = v;
            frees = f;
        }
        for a in args.iter().filter(|_| variadic.is_none()) {
            let v = match a {
                Arg::Value(o) => self.operand(fi, o, slots)?,
                Arg::Copy(p) => self.copied(fi, p, slots)?,
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
        // A list, map or channel handed over by place has the callee's
        // layout only if its elements have the callee's widths: a
        // `list[int64]` read as a `list[int32]` is garbage. What that should
        // do is not decided (`cca/README.md`); until it is, it is not built.
        let want: Option<Vec<Ty>> = match callee {
            // A variadic's bundle is built from the rest by the call, so
            // only its fixed parameters meet an argument one to one.
            Callee::Fn(idx) => {
                let callee_f = &self.p.functions[idx.0 as usize];
                let fixed = if callee_f.variadic { callee_f.params.len().saturating_sub(1) } else { callee_f.params.len() };
                Some(callee_f.params[..fixed].iter().map(|p| callee_f.locals[p.0 as usize].ty.clone()).collect())
            }
            // A list lent to C's pointer: C reads its elements at the
            // pointer's width.
            Callee::Extern(id) => Some(self.p.items.externs[id.0 as usize].params.iter().map(|(_, t)| match t.base() {
                Ty::Ptr { elem, .. } => Ty::Seq { elem: elem.clone(), kind: crate::types::SeqKind::List },
                _ => t.clone(),
            }).collect()),
            _ => None,
        };
        if let Some(want) = want {
            for (a, w) in args.iter().zip(&want) {
                let place = match a {
                    Arg::Lend { place, .. } | Arg::Copy(place) | Arg::Give(place) => place,
                    _ => continue,
                };
                let (_, have) = self.slot_of(fi, place, slots)?;
                if matches!(callee, Callee::Extern(_)) && !(matches!(have.base(), Ty::Seq { .. }) && matches!(w.base(), Ty::Seq { .. })) {
                    continue;
                }
                if !self.same_layout(&have, w) {
                    let spell = |t: &Ty| crate::types::TySpelling { ty: t, structs: &self.p.items.structs, enums: &self.p.items.enums }.to_string();
                    self.cannot(format!("handing a `{}` to a parameter that takes a `{}` has no shape in the generated code yet: their elements differ in width", spell(&have), spell(w)), None);
                    return None;
                }
            }
        }
        let f = match callee {
            Callee::Fn(idx) => *self.fns.get(&(idx.0 as usize))?,
            Callee::Extern(id) => *self.externs.get(&id.0)?,
            Callee::Value(o) => {
                let ft = match self.operand_cy_ty(fi, o)?.base() {
                    Ty::Fn(ft) => (**ft).clone(),
                    _ => return None,
                };
                let fv = self.operand(fi, o, slots)?;
                return self.call_held(fv, &ft, &mut vals, &signs)?;
            }
            _ => {
                self.cannot("this call has no shape in the generated code yet", None);
                return None;
            }
        };
        // A function handed to C is its code: it captured nothing (the
        // checker refuses one that did), so the code is the plain function.
        if let Callee::Extern(id) = callee {
            let kinds: Vec<bool> = self.p.items.externs[id.0 as usize].params.iter().map(|(_, t)| matches!(t.base(), Ty::Fn(_))).collect();
            for (v, is_fn) in vals.iter_mut().zip(kinds) {
                if let (true, Ok(BasicValueEnum::StructValue(s))) = (is_fn, BasicValueEnum::try_from(*v)) {
                    *v = self.builder.build_extract_value(s, 0, "code").ok()?.into();
                }
            }
        }
        // What crosses to C by value crosses as C passes it (`cabi.rs`).
        let mut sret_slot = None;
        let mut ret_pass = cabi::CPass::Direct;
        if let Callee::Extern(id) = callee {
            let e = self.p.items.externs[id.0 as usize].clone();
            for (i, (_, t)) in e.params.iter().enumerate() {
                if matches!(t.base(), Ty::Fn(_)) {
                    continue;
                }
                let how = self.c_pass(t, false)?;
                if let (cabi::CPass::Direct, _) | (_, None) = (how, vals.get(i)) {
                    continue;
                }
                let v = BasicValueEnum::try_from(vals[i]).ok()?;
                vals[i] = self.to_c(v, how)?;
            }
            if e.ret != Ty::Nothing {
                ret_pass = self.c_pass(&e.ret.base().clone(), true)?;
                if let cabi::CPass::Indirect(t) = ret_pass {
                    let room = self.here_alloca(t, "c_sret")?;
                    vals.insert(0, room.into());
                    sret_slot = Some(room);
                }
            }
        }
        // Each argument at the width the callee declared.
        let want = f.get_type().get_param_types();
        for (i, w) in want.iter().enumerate() {
            // A value handed to a `lent` parameter — a literal, or what
            // another call gave back — has no place of its own for the
            // address to point at, so it gets one for the call.
            if let (Some(slot), Ok(BasicTypeEnum::PointerType(_))) = (vals.get(i).copied(), BasicTypeEnum::try_from(*w)) {
                if let Ok(v) = BasicValueEnum::try_from(slot) {
                    if !v.is_pointer_value() {
                        let tmp = self.builder.build_alloca(v.get_type(), "lent_value").ok()?;
                        let _ = self.builder.build_store(tmp, v);
                        vals[i] = tmp.into();
                        continue;
                    }
                }
            }
            if let Some(slot) = vals.get_mut(i) {
                if let (Ok(v), Ok(t)) = (BasicValueEnum::try_from(*slot), BasicTypeEnum::try_from(*w)) {
                    let signed = signs.get(i).copied().unwrap_or(true);
                    *slot = self.fit_as(v, t, signed).into();
                }
            }
        }
        let call = self.builder.build_call(f, &vals, "call").ok()?;
        if let (Some(_), cabi::CPass::Indirect(t)) = (sret_slot, ret_pass) {
            let kind = inkwell::attributes::Attribute::get_named_enum_kind_id("sret");
            call.add_attribute(inkwell::attributes::AttributeLoc::Param(0), self.ctx.create_type_attribute(kind, inkwell::types::AnyType::as_any_type_enum(&t)));
        }
        // A lent bundle's buffer was made for the call; what was in it
        // belongs to the arguments' owners.
        for d in frees {
            let free = self.rt("cyborg_free");
            let _ = self.builder.build_call(free, &[d.into()], "");
        }
        match (ret_pass, sret_slot) {
            (cabi::CPass::Indirect(t), Some(room)) => self.builder.build_load(t, room, "from_c").ok(),
            (cabi::CPass::Coerce(_), _) => {
                let Callee::Extern(id) = callee else { return basic_result(call) };
                let r = self.p.items.externs[id.0 as usize].ret.base().clone();
                let t = self.ty(&r)?;
                let v = basic_result(call)?;
                self.reinterpret(v, t)
            }
            _ => basic_result(call),
        }
    }

    /// The language's own operations. A length is a field of the handle,
    /// which is why the handle carries one.
    fn op(&mut self, fi: usize, op: &Op, args: &[Operand], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>, dest_ty: Option<Ty>) -> Option<Option<BasicValueEnum<'ctx>>> {
        match op {
            // `list.append` hands nothing back (`lists.md`), so it is
            // handled and has no value — which is not the same as not
            // being handled at all.
            Op::ListAppend => {
                self.list_append(fi, args, slots, func)?;
                Some(None)
            }
            Op::ListRemove => Some(Some(self.list_remove(fi, args, slots)?)),
            // SIMD (`simd.md`): a vector is N lanes of T in one register.
            Op::SimdFill => {
                let out = dest_ty?;
                let vt = self.ty(&out)?.into_vector_type();
                let x = self.operand(fi, args.first()?, slots)?;
                let x = self.fit(x, vt.get_element_type());
                let mut v = vt.get_undef();
                for k in 0..vt.get_size() {
                    v = self.builder.build_insert_element(v, x, self.ctx.i32_type().const_int(k as u64, false), "lane").ok()?;
                }
                Some(Some(v.into()))
            }
            Op::SimdLoad | Op::SimdStore => {
                // The lanes start at `pos i` (1-based); their range was
                // checked at the op's own site.
                let place = args.first()?.place()?.clone();
                let (at, ty) = self.slot_of(fi, &place, slots)?;
                let shape = match self.ty(&ty)? {
                    BasicTypeEnum::StructType(s) => s,
                    _ => return None,
                };
                let elem = Self::elem_of(&ty)?;
                let et = self.ty(&elem)?;
                let esize = self.size_of_ty(&elem)?;
                let w = self.ctx.i64_type();
                let i = self.operand(fi, args.get(1)?, slots)?.into_int_value();
                let signed = self.operand_signed(fi, args.get(1)?).unwrap_or(true);
                let i = self.builder.build_int_cast_sign_flag(i, w, signed, "pos").ok()?;
                let zb = self.builder.build_int_sub(i, w.const_int(1, false), "zb").ok()?;
                let off = self.builder.build_int_mul(zb, w.const_int(esize, false), "off").ok()?;
                let data = self.builder.build_struct_gep(shape, at, 0, "data").ok()?;
                let data = self.builder.build_load(self.ctx.ptr_type(AddressSpace::default()), data, "p").ok()?.into_pointer_value();
                let from = unsafe { self.builder.build_gep(self.ctx.i8_type(), data, &[off], "lanes").ok()? };
                let align = esize.max(1) as u32;
                if *op == Op::SimdLoad {
                    let vt = self.ty(&dest_ty?)?.into_vector_type();
                    let v = self.builder.build_load(vt, from, "load").ok()?.into_vector_value();
                    v.as_instruction().map(|i| i.set_alignment(align));
                    Some(Some(v.into()))
                } else {
                    let v = self.operand(fi, args.get(2)?, slots)?;
                    let _ = et;
                    let st = self.builder.build_store(from, v).ok()?;
                    st.set_alignment(align).ok()?;
                    Some(None)
                }
            }
            // A new order fixed when compiling: one instruction.
            Op::SimdShuffle(order) => {
                let v = self.operand(fi, args.first()?, slots)?.into_vector_value();
                let i32t = self.ctx.i32_type();
                let mask: Vec<IntValue<'ctx>> = order.iter().map(|l| i32t.const_int(*l as u64, false)).collect();
                let mask = inkwell::types::VectorType::const_vector(&mask);
                Some(Some(self.builder.build_shuffle_vector(v, v.get_type().get_undef(), mask, "shuffle").ok()?.into()))
            }
            Op::SimdLoadSome | Op::SimdStoreSome => Some(self.simd_some(fi, op, args, slots, dest_ty)?),
            Op::SimdAll | Op::SimdAny | Op::SimdMax | Op::SimdMin | Op::SimdSum => {
                let v = self.operand(fi, args.first()?, slots)?.into_vector_value();
                let et = v.get_type().get_element_type();
                let lane = self.operand_lane(fi, args.first()?);
                let float = et.is_float_type();
                let signed = lane.as_ref().and_then(int_signed).unwrap_or(true);
                let name = match (op, float, signed) {
                    (Op::SimdAll, _, _) => "llvm.vector.reduce.and",
                    (Op::SimdAny, _, _) => "llvm.vector.reduce.or",
                    (Op::SimdMax, true, _) => "llvm.vector.reduce.fmax",
                    (Op::SimdMin, true, _) => "llvm.vector.reduce.fmin",
                    (Op::SimdMax, false, true) => "llvm.vector.reduce.smax",
                    (Op::SimdMax, false, false) => "llvm.vector.reduce.umax",
                    (Op::SimdMin, false, true) => "llvm.vector.reduce.smin",
                    (Op::SimdMin, false, false) => "llvm.vector.reduce.umin",
                    (_, true, _) => "llvm.vector.reduce.fadd",
                    _ => "llvm.vector.reduce.add",
                };
                if *op == Op::SimdSum && !float {
                    return Some(Some(self.simd_sum(fi, v, signed, func)?));
                }
                let id = inkwell::intrinsics::Intrinsic::find(name)?;
                let f = id.get_declaration(self.module, &[v.get_type().into()])?;
                let call = if name == "llvm.vector.reduce.fadd" {
                    let zero = et.into_float_type().const_float(0.0);
                    self.builder.build_call(f, &[zero.into(), v.into()], "reduce").ok()?
                } else {
                    self.builder.build_call(f, &[v.into()], "reduce").ok()?
                };
                Some(Some(basic_result(call)?))
            }
            // C's memory (`ffi.md`), all `nocheck` sites: the writer vouches
            // for the address. `read` and `write` move one `T`; `at` moves
            // a whole number of `T`s, never bytes.
            Op::PtrRead => {
                let p = self.operand(fi, args.first()?, slots)?.into_pointer_value();
                let elem = self.pointee(fi, args.first()?)?;
                let t = self.ty(&elem)?;
                Some(Some(self.builder.build_load(t, p, "read").ok()?))
            }
            Op::PtrWrite => {
                let p = self.operand(fi, args.first()?, slots)?.into_pointer_value();
                let elem = self.pointee(fi, args.first()?)?;
                let t = self.ty(&elem)?;
                let v = self.operand(fi, args.get(1)?, slots)?;
                let v = match (v, t) {
                    (BasicValueEnum::IntValue(i), BasicTypeEnum::IntType(it)) if i.get_type().get_bit_width() != it.get_bit_width() => {
                        let signed = self.operand_signed(fi, args.get(1)?).unwrap_or(true);
                        self.builder.build_int_cast_sign_flag(i, it, signed, "fit").ok()?.into()
                    }
                    _ => self.fit(v, t),
                };
                let _ = self.builder.build_store(p, v);
                Some(None)
            }
            Op::PtrAt => {
                let p = self.operand(fi, args.first()?, slots)?.into_pointer_value();
                let elem = self.pointee(fi, args.first()?)?;
                let t = self.ty(&elem)?;
                let i = self.operand(fi, args.get(1)?, slots)?.into_int_value();
                let signed = self.operand_signed(fi, args.get(1)?).unwrap_or(true);
                let i = self.builder.build_int_cast_sign_flag(i, self.ctx.i64_type(), signed, "idx").ok()?;
                Some(Some(unsafe { self.builder.build_gep(t, p, &[i], "at").ok()? }.into()))
            }
            // A text's code points or grapheme clusters: a walk over its bytes
            // (`strings.md`: three counts, no single length).
            Op::StrPoints | Op::StrGraphemes => {
                let p = args.first()?.place()?;
                let (slot, ty) = self.slot_of(fi, p, slots)?;
                let st = match self.ty(&ty)? {
                    BasicTypeEnum::StructType(s) => s,
                    _ => return None,
                };
                let data = self.load_field_ptr(st, slot, 0);
                let n = self.load_field_int(st, slot, 1);
                let f = self.rt(if *op == Op::StrPoints { "cyborg_str_points" } else { "cyborg_str_graphemes" });
                let c = basic_result(self.builder.build_call(f, &[data.into(), n.into()], "count").ok()?)?;
                Some(Some(c))
            }
            // `str.each.<unit>`: the text's units, as a sequence the loop
            // walks (`strings.md`) — bytes copied, points decoded, grapheme
            // clusters each a text of its own (the sequence owns them).
            Op::StrEach(unit) => {
                let p = args.first()?.place()?;
                let (slot, ty) = self.slot_of(fi, p, slots)?;
                let st = match self.ty(&ty)? {
                    BasicTypeEnum::StructType(s) => s,
                    _ => return None,
                };
                let data = self.load_field_ptr(st, slot, 0);
                let raw = self.load_field_int(st, slot, 1);
                let bytes = self.real_length(raw);
                let w = self.ctx.i64_type();
                let out_ty = dest_ty.clone()?;
                let elem = Self::elem_of(&out_ty)?;
                let esize = self.size_of_ty(&elem)?;
                let count = match unit {
                    crate::types::Unit::Byte => bytes,
                    crate::types::Unit::Point | crate::types::Unit::Free(_) => {
                        let f = self.rt("cyborg_str_points");
                        basic_result(self.builder.build_call(f, &[data.into(), raw.into()], "n").ok()?)?.into_int_value()
                    }
                    crate::types::Unit::Grapheme => {
                        let f = self.rt("cyborg_str_graphemes");
                        basic_result(self.builder.build_call(f, &[data.into(), raw.into()], "n").ok()?)?.into_int_value()
                    }
                };
                let size = self.builder.build_int_mul(count, w.const_int(esize, false), "size").ok()?;
                let buf = self.alloc(size);
                match unit {
                    crate::types::Unit::Byte => {
                        let f = self.rt("cyborg_copy");
                        let _ = self.builder.build_call(f, &[buf.into(), data.into(), bytes.into()], "");
                    }
                    crate::types::Unit::Point | crate::types::Unit::Free(_) => {
                        let f = self.rt("cyborg_str_decode_points");
                        let _ = self.builder.build_call(f, &[data.into(), raw.into(), buf.into()], "");
                    }
                    crate::types::Unit::Grapheme => {
                        let f = self.rt("cyborg_str_split_graphemes");
                        let _ = self.builder.build_call(f, &[data.into(), raw.into(), buf.into()], "");
                    }
                }
                let shape = match self.ty(&out_ty)? {
                    BasicTypeEnum::StructType(s) => s,
                    _ => return None,
                };
                let mut agg = shape.get_undef();
                agg = self.builder.build_insert_value(agg, buf, 0, "p").ok()?.into_struct_value();
                agg = self.builder.build_insert_value(agg, count, 1, "n").ok()?.into_struct_value();
                if shape.count_fields() > 2 {
                    agg = self.builder.build_insert_value(agg, count, 2, "c").ok()?.into_struct_value();
                }
                Some(Some(agg.into()))
            }
            // The number under a character (`conversion.md`): a byte's is a
            // `uint8`, a point's a `uint32`, and the char already is it.
            Op::CharCode => {
                let v = self.operand(fi, args.first()?, slots)?.into_int_value();
                let want = match dest_ty.as_ref().and_then(|t| self.ty(t)) {
                    Some(BasicTypeEnum::IntType(t)) => t,
                    _ => v.get_type(),
                };
                Some(Some(self.builder.build_int_z_extend_or_bit_cast(v, want, "code").ok()?.into()))
            }
            // C's null (`ffi.md`): a pointer compared with nothing.
            Op::PtrIsNull => {
                let v = self.operand(fi, args.first()?, slots)?.into_pointer_value();
                Some(Some(self.builder.build_is_null(v, "is_null").ok()?.into()))
            }
            // A list is a pointer, a length and a capacity, an array the
            // first two (`lists.md`, layout): the elements stay where they
            // are, and only the handle changes. A list's spare capacity is
            // still one allocation, freed whole.
            Op::ToList | Op::ToArray => {
                let out = dest_ty?;
                let shape = match self.ty(&out)? {
                    BasicTypeEnum::StructType(s) => s,
                    _ => return None,
                };
                let v = self.operand(fi, args.first()?, slots)?.into_struct_value();
                let p = self.builder.build_extract_value(v, 0, "p").ok()?;
                let n = self.builder.build_extract_value(v, 1, "n").ok()?;
                let mut agg = shape.get_undef();
                agg = self.builder.build_insert_value(agg, p, 0, "p").ok()?.into_struct_value();
                agg = self.builder.build_insert_value(agg, n, 1, "n").ok()?.into_struct_value();
                if shape.count_fields() > 2 {
                    // An array's capacity is its length; a list keeps its own.
                    let c = if v.get_type().count_fields() > 2 { self.builder.build_extract_value(v, 2, "c").ok()? } else { n };
                    agg = self.builder.build_insert_value(agg, c, 2, "c").ok()?.into_struct_value();
                }
                Some(Some(agg.into()))
            }
            Op::ListSlice => {
                let out = dest_ty?;
                Some(Some(self.list_slice(fi, args, &out, slots, func)?))
            }
            Op::MapSet => {
                self.map_set(fi, args, slots)?;
                Some(None)
            }
            Op::MapHas => Some(Some(self.map_has(fi, args, slots)?)),
            Op::ChannelSend => {
                self.chan_send(fi, args, slots)?;
                Some(None)
            }
            Op::ChannelRelease => {
                self.chan_release(fi, args, slots)?;
                Some(None)
            }
            Op::MapKeys | Op::MapValues | Op::MapPairs => {
                let out = dest_ty?;
                Some(Some(self.map_walk(fi, op, args, &out, slots, func)?))
            }
            // Bytes is the length a `str` already carries; the other two
            // units have to walk the text, and that is std's to write.
            Op::StrBytes | Op::ListLength | Op::ArrayLength | Op::MapLength => {
                let p = args.first()?.place()?;
                let (slot, ty) = self.slot_of(fi, p, slots)?;
                // A bundle's length is its shape's: known here (`functions.md`).
                if let Ty::Bundle(ts) = ty.base() {
                    return Some(Some(self.ctx.i64_type().const_int(ts.len() as u64, false).into()));
                }
                let text = matches!(ty.base(), Ty::Str(_));
                let shape = self.ty(&ty)?;
                let st = match shape {
                    BasicTypeEnum::StructType(s) => s,
                    _ => return None,
                };
                let field = self.builder.build_struct_gep(st, slot, 1, "len").ok()?;
                let raw = self.builder.build_load(self.ctx.i64_type(), field, "n").ok()?;
                // A text's length carries a mark saying which memory it is
                // in; the count is the rest of it.
                if text {
                    let n = self.real_length(raw.into_int_value());
                    return Some(Some(n.into()));
                }
                Some(Some(raw))
            }
            _ => None,
        }
    }

    /// Does this site get a run-time check? Unproven ones do; a `nocheck`
    /// one never does; a proven one only in the proof-checking build.
    fn gets_check(&self, v: &Verdict) -> bool {
        match v {
            Verdict::Trusted => false,
            Verdict::Proven { .. } => self.assert_proofs,
            _ => true,
        }
    }

    /// Trap at `site` when `bad` holds; carry on otherwise.
    fn guard(&mut self, fi: usize, site: SiteId, bad: IntValue<'ctx>, func: FunctionValue<'ctx>) -> Option<()> {
        let fail = self.ctx.append_basic_block(func, "check_failed");
        let good = self.ctx.append_basic_block(func, "checked");
        let _ = self.builder.build_conditional_branch(bad, fail, good);
        self.builder.position_at_end(fail);
        self.trap(fi, site);
        self.builder.position_at_end(good);
        Some(())
    }

    /// Into a narrower type, or a range type: the value has to be one the
    /// target holds (`conversion.md`: "a runtime check unless Precog proves
    /// the fit"). Compared in 128 bits, the value extended by its own sign,
    /// so no comparison can itself wrap.
    fn narrow_check(&mut self, fi: usize, id: SiteId, slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) {
        let site = self.p.functions[fi].site(id).clone();
        if !self.gets_check(&site.verdict) {
            return;
        }
        let SiteKind::Narrow { to } = &site.kind else { return };
        // Into a `deci` range type: compared as decimals, by the runtime.
        if let (Ty::Deci(bits), Some(crate::types::Bound::Bin { lo, hi })) = (to.base(), to.bound()) {
            let Some(o) = site.operands.first() else { return };
            let o = match o.place() {
                Some(p) => Operand::Read(p.clone()),
                None => o.clone(),
            };
            let Some(BasicValueEnum::IntValue(v)) = self.operand(fi, &o, slots) else { return };
            let Some(bad) = self.deci_outside(v, *bits, &lo.clone(), &hi.clone()) else { return };
            let _ = self.guard(fi, id, bad, func);
            return;
        }
        if let Ty::Bin(crate::types::BinW::Fixed(bits @ (16 | 32))) = to.base() {
            return self.bin_narrow_check(fi, id, *bits, &site, slots, func);
        }
        // A duration is never negative (owner, 2026-09-22): the site holds
        // both sides of the subtraction, and it fails when the second is
        // the larger. Only the first was looked at, and it always fit.
        if *to == Ty::Duration && site.operands.len() == 2 {
            let read = |o: &Operand| match o.place() {
                Some(p) => Operand::Read(p.clone()),
                None => o.clone(),
            };
            let (a, b) = (read(&site.operands[0]), read(&site.operands[1]));
            let (Some(BasicValueEnum::IntValue(x)), Some(BasicValueEnum::IntValue(y))) = (self.operand(fi, &a, slots), self.operand(fi, &b, slots)) else { return };
            let w = self.ctx.i64_type();
            let x = self.fit_as(x.into(), w.into(), true).into_int_value();
            let y = self.fit_as(y.into(), w.into(), true).into_int_value();
            let Ok(bad) = self.builder.build_int_compare(IntPredicate::SLT, x, y, "below_zero") else { return };
            let _ = self.guard(fi, id, bad, func);
            return;
        }
        let Some((lo, hi)) = crate::precog::smt::int_bounds(to) else {
            if !matches!(to.base(), Ty::Bin(_)) {
                self.cannot(format!("a narrowing into `{}` has no run-time check yet", crate::types::TySpelling { ty: to, structs: &self.p.items.structs, enums: &self.p.items.enums }), Some(self.p.functions[fi].span(site.span)));
            }
            return;
        };
        let Some(o) = site.operands.first() else { return };
        let o = match o.place() {
            Some(p) => Operand::Read(p.clone()),
            None => o.clone(),
        };
        let signed = self.operand_signed(fi, &o).unwrap_or(true);
        let Some(BasicValueEnum::IntValue(v)) = self.operand(fi, &o, slots) else { return };
        let w = self.ctx.i128_type();
        let x = self.fit_as(v.into(), w.into(), signed).into_int_value();
        let (Ok(below), Ok(above)) = (
            self.builder.build_int_compare(IntPredicate::SLT, x, const_i128(w, lo), "below"),
            self.builder.build_int_compare(IntPredicate::SGT, x, const_i128(w, hi), "above"),
        ) else { return };
        let Ok(bad) = self.builder.build_or(below, above, "outside") else { return };
        let _ = self.guard(fi, id, bad, func);
    }

    /// A value going into a bin16 or bin32 that Precog could not prove fits:
    /// it fits when rounding it to the width gives a finite value — a finite
    /// value that rounds to infinity went past the largest one (65520 into a
    /// bin16). Until now a bin narrowing generated no check at all.
    fn bin_narrow_check(&mut self, fi: usize, id: SiteId, bits: u16, site: &crate::mir::Site, slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) {
        let Some(o) = site.operands.first() else { return };
        let o = match o.place() {
            Some(p) => Operand::Read(p.clone()),
            None => o.clone(),
        };
        let target = if bits == 16 { self.ctx.f16_type() } else { self.ctx.f32_type() };
        // A literal is checked as it is written: rounded once from its
        // digits at the width. Read at 64 bits first, `…568447.0` was the
        // limit, and the asserted proof fired on a value that fits.
        if let Operand::Const(Const::Bin(text)) = &o {
            let r = unsafe { target.const_float_from_string(&text.replace('_', "")) };
            let Ok(rz) = self.builder.build_float_sub(r, r, "rz") else { return };
            let Ok(went) = self.builder.build_float_compare(inkwell::FloatPredicate::UNE, rz, target.const_zero(), "rounds_past") else { return };
            let _ = self.guard(fi, id, went, func);
            return;
        }
        let Some(BasicValueEnum::FloatValue(x)) = self.operand(fi, &o, slots) else { return };
        if x.get_type() == target {
            return;
        }
        let Ok(r) = self.builder.build_float_trunc(x, target, "rounded") else { return };
        let Ok(r) = self.builder.build_float_ext(r, x.get_type(), "back") else { return };
        // Infinite after, finite before: x - x is 0 only for a finite x,
        // and r - r is not 0 (NaN) only for an infinite (or NaN) r.
        let (Ok(xz), Ok(rz)) = (self.builder.build_float_sub(x, x, "xz"), self.builder.build_float_sub(r, r, "rz")) else { return };
        let zero = x.get_type().const_zero();
        let (Ok(finite), Ok(went)) = (
            self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, xz, zero, "finite"),
            self.builder.build_float_compare(inkwell::FloatPredicate::UNE, rz, zero, "went_past"),
        ) else { return };
        let Ok(bad) = self.builder.build_and(finite, went, "rounds_past") else { return };
        let _ = self.guard(fi, id, bad, func);
    }

    /// A bounded list, map or text after a change: its length has to be
    /// inside the bound.
    fn length_check(&mut self, fi: usize, id: SiteId, slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) {
        let site = self.p.functions[fi].site(id).clone();
        if !self.gets_check(&site.verdict) {
            return;
        }
        let SiteKind::LengthBound { to } = &site.kind else { return };
        let (lo, hi, unit) = match to {
            Ty::Bounded { bound: crate::types::Bound::Len { lo, hi, unit }, .. } => (*lo as i128, *hi as i128, *unit),
            _ => return,
        };
        let span = Some(self.p.functions[fi].span(site.span));
        let _ = span;
        let Some(p) = site.operands.first().and_then(|o| o.place()).cloned() else { return };
        let Some((slot, ty)) = self.slot_of(fi, &p, slots) else { return };
        let Some(BasicTypeEnum::StructType(st)) = self.ty(&ty) else { return };
        let Ok(field) = self.builder.build_struct_gep(st, slot, 1, "len") else { return };
        let Ok(raw) = self.builder.build_load(self.ctx.i64_type(), field, "n") else { return };
        let mut n = raw.into_int_value();
        if matches!(ty.base(), Ty::Str(_)) {
            n = self.real_length(n);
            // Counted in code points or grapheme clusters: a walk over it.
            if let Some(u @ (crate::types::Unit::Point | crate::types::Unit::Grapheme)) = unit {
                let data = self.load_field_ptr(st, slot, 0);
                let f = self.rt(if u == crate::types::Unit::Point { "cyborg_str_points" } else { "cyborg_str_graphemes" });
                let Some(c) = self.builder.build_call(f, &[data.into(), raw.into()], "count").ok().and_then(basic_result) else { return };
                n = c.into_int_value();
            }
        }
        let w = self.ctx.i128_type();
        let Ok(n) = self.builder.build_int_z_extend(n, w, "n128") else { return };
        let (Ok(below), Ok(above)) = (
            self.builder.build_int_compare(IntPredicate::SLT, n, const_i128(w, lo), "below"),
            self.builder.build_int_compare(IntPredicate::SGT, n, const_i128(w, hi), "above"),
        ) else { return };
        let Ok(bad) = self.builder.build_or(below, above, "outside") else { return };
        let _ = self.guard(fi, id, bad, func);
    }

    /// `'xs'[i]` must be within the list: 1-based, so 1 through the
    /// length (`lists.md`). Proven sites carry no check; the rest trap
    /// with the site's own words.
    fn bounds_check(&mut self, fi: usize, id: SiteId, slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) {
        let site = self.p.functions[fi].site(id).clone();
        if !self.gets_check(&site.verdict) {
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
        // In 128 bits, each extended by its own sign, so a `uint64`
        // position past 2^63 is a big number and not a negative one.
        let w = self.ctx.i128_type();
        let si = site.operands.first().and_then(|o| self.operand_signed(fi, o)).unwrap_or(true);
        let sn = site.operands.get(1).and_then(|o| self.operand_signed(fi, o)).unwrap_or(false);
        let i = self.fit_as(i, w.into(), si).into_int_value();
        let n = self.fit_as(n, w.into(), sn).into_int_value();
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

    /// `x xx e` by squaring: the answer takes the base once per set bit of
    /// the exponent, the base squared between. With `check`, every multiply
    /// is overflow-checked at this width — and each one only happens when
    /// the answer needs it (the multiplier is 1 otherwise), so a check never
    /// fires on work the answer did not need. Without it (`xx~`, or proven),
    /// the multiplies wrap. The exponent was checked non-negative at its
    /// own site.
    fn ipow(&mut self, fi: usize, x: IntValue<'ctx>, y: IntValue<'ctx>, signed: bool, check: Option<SiteId>, func: FunctionValue<'ctx>) -> Option<IntValue<'ctx>> {
        let t = x.get_type();
        let one = t.const_int(1, false);
        let pre = self.builder.get_insert_block()?;
        let head = self.ctx.append_basic_block(func, "pow_head");
        let body = self.ctx.append_basic_block(func, "pow_body");
        let done = self.ctx.append_basic_block(func, "pow_done");
        let _ = self.builder.build_unconditional_branch(head);
        self.builder.position_at_end(head);
        let r = self.builder.build_phi(t, "r").ok()?;
        let bs = self.builder.build_phi(t, "b").ok()?;
        let e = self.builder.build_phi(t, "e").ok()?;
        let (rv, bv, ev) = (r.as_basic_value().into_int_value(), bs.as_basic_value().into_int_value(), e.as_basic_value().into_int_value());
        let more = self.builder.build_int_compare(IntPredicate::NE, ev, t.const_zero(), "more").ok()?;
        let _ = self.builder.build_conditional_branch(more, body, done);
        self.builder.position_at_end(body);
        let odd = self.builder.build_and(ev, one, "odd").ok()?;
        let odd = self.builder.build_int_compare(IntPredicate::NE, odd, t.const_zero(), "is_odd").ok()?;
        let m = self.builder.build_select(odd, bv, one, "m").ok()?.into_int_value();
        let mul = |this: &mut Self, a: IntValue<'ctx>, b: IntValue<'ctx>| -> Option<IntValue<'ctx>> {
            match check {
                Some(s) => {
                    let name = if signed { "llvm.smul.with.overflow" } else { "llvm.umul.with.overflow" };
                    let id = inkwell::intrinsics::Intrinsic::find(name)?;
                    let f = id.get_declaration(this.module, &[t.into()])?;
                    let agg = basic_result(this.builder.build_call(f, &[a.into(), b.into()], "chk").ok()?)?.into_struct_value();
                    let v = this.builder.build_extract_value(agg, 0, "v").ok()?.into_int_value();
                    let o = this.builder.build_extract_value(agg, 1, "o").ok()?.into_int_value();
                    this.guard(fi, s, o, func)?;
                    Some(v)
                }
                None => this.builder.build_int_mul(a, b, "mul").ok(),
            }
        };
        let r_next = mul(self, rv, m)?;
        let e_next = self.builder.build_right_shift(ev, one, false, "e_next").ok()?;
        let again = self.builder.build_int_compare(IntPredicate::NE, e_next, t.const_zero(), "again").ok()?;
        let sq = self.builder.build_select(again, bv, one, "sq").ok()?.into_int_value();
        let b_next = mul(self, bv, sq)?;
        let end = self.builder.get_insert_block()?;
        let _ = self.builder.build_unconditional_branch(head);
        r.add_incoming(&[(&one, pre), (&r_next, end)]);
        bs.add_incoming(&[(&x, pre), (&b_next, end)]);
        e.add_incoming(&[(&y, pre), (&e_next, end)]);
        self.builder.position_at_end(done);
        Some(rv)
    }

    /// `xx`: the exponent is not negative.
    fn negative_exponent_check(&mut self, fi: usize, id: SiteId, slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) {
        let site = self.p.functions[fi].site(id).clone();
        if !self.gets_check(&site.verdict) {
            return;
        }
        let Some(o) = site.operands.first() else { return };
        if self.operand_signed(fi, o) == Some(false) {
            return;
        }
        let read = match o.place() {
            Some(p) => Operand::Read(p.clone()),
            None => o.clone(),
        };
        let Some(BasicValueEnum::IntValue(e)) = self.operand(fi, &read, slots) else { return };
        let Ok(neg) = self.builder.build_int_compare(IntPredicate::SLT, e, e.get_type().const_zero(), "negative") else { return };
        let _ = self.guard(fi, id, neg, func);
    }

    /// `simd.load` / `simd.store` (`simd.md`): every lane from `pos i` is
    /// inside — `1 <= i` and `i + N - 1 <= length`, in 128 bits.
    fn simd_range_check(&mut self, fi: usize, id: SiteId, lanes: u32, slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) {
        let site = self.p.functions[fi].site(id).clone();
        if !self.gets_check(&site.verdict) {
            return;
        }
        let (Some(io), Some(lo)) = (site.operands.first(), site.operands.get(1)) else { return };
        let read = match io.place() {
            Some(p) => Operand::Read(p.clone()),
            None => io.clone(),
        };
        let Some(i) = self.operand(fi, &read, slots) else { return };
        let Some(place) = lo.place().cloned() else { return };
        let Some((at, ty)) = self.slot_of(fi, &place, slots) else { return };
        let Some(BasicTypeEnum::StructType(shape)) = self.ty(&ty) else { return };
        let Ok(lp) = self.builder.build_struct_gep(shape, at, 1, "len") else { return };
        let Ok(n) = self.builder.build_load(self.ctx.i64_type(), lp, "n") else { return };
        let w = self.ctx.i128_type();
        let si = self.operand_signed(fi, io).unwrap_or(true);
        let i = self.fit_as(i, w.into(), si).into_int_value();
        let n = self.fit_as(n, w.into(), false).into_int_value();
        let last = match self.builder.build_int_add(i, w.const_int(lanes.saturating_sub(1) as u64, false), "last") {
            Ok(v) => v,
            Err(_) => return,
        };
        let (Ok(low), Ok(high)) = (
            self.builder.build_int_compare(IntPredicate::SLT, i, w.const_int(1, false), "low"),
            self.builder.build_int_compare(IntPredicate::SGT, last, n, "high"),
        ) else { return };
        let Ok(bad) = self.builder.build_or(low, high, "lanes_out") else { return };
        let _ = self.guard(fi, id, bad, func);
    }

    /// `simd.load.some` / `simd.store.some` (owner, 2026-09-27: a): only the
    /// first k lanes touch memory — LLVM's masked load and store, so the
    /// lanes past the list are never read — and a load's other lanes are
    /// zero.
    fn simd_some(&mut self, fi: usize, op: &Op, args: &[Operand], slots: &[Option<PointerValue<'ctx>>], dest_ty: Option<Ty>) -> Option<Option<BasicValueEnum<'ctx>>> {
        let place = args.first()?.place()?.clone();
        let (at, ty) = self.slot_of(fi, &place, slots)?;
        let shape = match self.ty(&ty)? {
            BasicTypeEnum::StructType(s) => s,
            _ => return None,
        };
        let elem = Self::elem_of(&ty)?;
        let esize = self.size_of_ty(&elem)?;
        let w = self.ctx.i64_type();
        let i = self.operand(fi, args.get(1)?, slots)?.into_int_value();
        let signed = self.operand_signed(fi, args.get(1)?).unwrap_or(true);
        let i = self.builder.build_int_cast_sign_flag(i, w, signed, "pos").ok()?;
        let zb = self.builder.build_int_sub(i, w.const_int(1, false), "zb").ok()?;
        let off = self.builder.build_int_mul(zb, w.const_int(esize, false), "off").ok()?;
        let data = self.builder.build_struct_gep(shape, at, 0, "data").ok()?;
        let data = self.builder.build_load(self.ctx.ptr_type(AddressSpace::default()), data, "p").ok()?.into_pointer_value();
        let from = unsafe { self.builder.build_gep(self.ctx.i8_type(), data, &[off], "lanes").ok()? };
        let vt = if *op == Op::SimdLoadSome { self.ty(&dest_ty?)?.into_vector_type() } else { self.operand(fi, args.get(3)?, slots)?.into_vector_value().get_type() };
        let n = vt.get_size();
        // Lane j is on when j < k.
        let k = self.operand(fi, args.get(2)?, slots)?.into_int_value();
        let ks = self.operand_signed(fi, args.get(2)?).unwrap_or(false);
        let k = self.builder.build_int_cast_sign_flag(k, w, ks, "k").ok()?;
        let lanes: Vec<IntValue<'ctx>> = (0..n).map(|j| w.const_int(j as u64, false)).collect();
        let lanes = inkwell::types::VectorType::const_vector(&lanes);
        let mut kv = w.vec_type(n).get_undef();
        for j in 0..n {
            kv = self.builder.build_insert_element(kv, k, self.ctx.i32_type().const_int(j as u64, false), "k").ok()?;
        }
        let mask = self.builder.build_int_compare(IntPredicate::ULT, lanes, kv, "on").ok()?;
        let align = self.ctx.i32_type().const_int(esize.max(1), false);
        let ptr_t = self.ctx.ptr_type(AddressSpace::default());
        if *op == Op::SimdLoadSome {
            let id = inkwell::intrinsics::Intrinsic::find("llvm.masked.load")?;
            let f = id.get_declaration(self.module, &[vt.into(), ptr_t.into()])?;
            let zero = vt.const_zero();
            // LLVM 22 took the alignment argument off; older ones have it.
            let call = if f.count_params() == 4 {
                self.builder.build_call(f, &[from.into(), align.into(), mask.into(), zero.into()], "load_some").ok()?
            } else {
                self.builder.build_call(f, &[from.into(), mask.into(), zero.into()], "load_some").ok()?
            };
            Some(Some(basic_result(call)?))
        } else {
            let v = self.operand(fi, args.get(3)?, slots)?;
            let id = inkwell::intrinsics::Intrinsic::find("llvm.masked.store")?;
            let f = id.get_declaration(self.module, &[vt.into(), ptr_t.into()])?;
            if f.count_params() == 4 {
                self.builder.build_call(f, &[v.into(), from.into(), align.into(), mask.into()], "").ok()?;
            } else {
                self.builder.build_call(f, &[v.into(), from.into(), mask.into()], "").ok()?;
            }
            Some(None)
        }
    }

    /// `k <= N`, `1 <= i` and `i + k - 1 <= length`, in 128 bits.
    fn simd_some_check(&mut self, fi: usize, id: SiteId, lanes: u32, slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) {
        let site = self.p.functions[fi].site(id).clone();
        if !self.gets_check(&site.verdict) {
            return;
        }
        let (Some(io), Some(lo), Some(ko)) = (site.operands.first(), site.operands.get(1), site.operands.get(2)) else { return };
        let w = self.ctx.i128_type();
        let read = |this: &mut Self, o: &Operand| -> Option<IntValue<'ctx>> {
            let r = match o.place() {
                Some(p) => Operand::Read(p.clone()),
                None => o.clone(),
            };
            let v = this.operand(fi, &r, slots)?;
            let s = this.operand_signed(fi, o).unwrap_or(true);
            Some(this.fit_as(v, w.into(), s).into_int_value())
        };
        let (Some(i), Some(k)) = (read(self, io), read(self, ko)) else { return };
        let Some(place) = lo.place().cloned() else { return };
        let Some((at, ty)) = self.slot_of(fi, &place, slots) else { return };
        let Some(BasicTypeEnum::StructType(shape)) = self.ty(&ty) else { return };
        let Ok(lp) = self.builder.build_struct_gep(shape, at, 1, "len") else { return };
        let Ok(n) = self.builder.build_load(self.ctx.i64_type(), lp, "n") else { return };
        let n = self.fit_as(n, w.into(), false).into_int_value();
        let Some(bad) = (|| {
            let last = self.builder.build_int_add(i, k, "past").ok()?;
            let last = self.builder.build_int_sub(last, w.const_int(1, false), "last").ok()?;
            let many = self.builder.build_int_compare(IntPredicate::SGT, k, w.const_int(lanes as u64, false), "many").ok()?;
            let low = self.builder.build_int_compare(IntPredicate::SLT, i, w.const_int(1, false), "low").ok()?;
            let high = self.builder.build_int_compare(IntPredicate::SGT, last, n, "high").ok()?;
            let b = self.builder.build_or(many, low, "bad").ok()?;
            self.builder.build_or(b, high, "items_out").ok()
        })() else { return };
        let _ = self.guard(fi, id, bad, func);
    }

    /// `1 <= a <= b + 1 <= length + 1`, in 128 bits so no `+ 1` wraps.
    fn slice_check(&mut self, fi: usize, id: SiteId, slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) {
        let site = self.p.functions[fi].site(id).clone();
        if !self.gets_check(&site.verdict) {
            return;
        }
        let w = self.ctx.i128_type();
        let mut v = Vec::new();
        for (k, o) in site.operands.iter().enumerate().take(3) {
            let read = match o.place() {
                Some(p) => Operand::Read(p.clone()),
                None => o.clone(),
            };
            let Some(x) = self.operand(fi, &read, slots) else { return };
            let signed = self.operand_signed(fi, o).unwrap_or(k < 2);
            v.push(self.fit_as(x, w.into(), signed).into_int_value());
        }
        let [a, b, n] = [v[0], v[1], v[2]];
        let one = w.const_int(1, false);
        let Ok(b1) = self.builder.build_int_add(b, one, "b1") else { return };
        let Ok(low) = self.builder.build_int_compare(IntPredicate::SLT, a, one, "low") else { return };
        let Ok(back) = self.builder.build_int_compare(IntPredicate::SGT, a, b1, "backwards") else { return };
        let Ok(high) = self.builder.build_int_compare(IntPredicate::SGT, b, n, "high") else { return };
        let Ok(bad) = self.builder.build_or(low, back, "bad") else { return };
        let Ok(bad) = self.builder.build_or(bad, high, "outside") else { return };
        let fail = self.ctx.append_basic_block(func, "slice_outside");
        let good = self.ctx.append_basic_block(func, "slice_ok");
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
                (BasicValueEnum::IntValue(i), Some(BasicTypeEnum::IntType(t))) if i.get_type().get_bit_width() != t.get_bit_width() => {
                    let signed = int_signed(&ty).unwrap_or(true);
                    self.builder.build_int_cast_sign_flag(i, t, signed, "fit").unwrap().into()
                }
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

/// A 128-bit constant from a Rust `i128`.
fn const_i128<'ctx>(t: inkwell::types::IntType<'ctx>, v: i128) -> IntValue<'ctx> {
    let bits = v as u128;
    t.const_int_arbitrary_precision(&[bits as u64, (bits >> 64) as u64])
}

/// Whether a whole-number type is signed: `None` for anything else.
fn int_signed(t: &Ty) -> Option<bool> {
    match t.base() {
        Ty::Int(IntW::Fixed { signed, .. }) => Some(*signed),
        Ty::Int(IntW::Free(_)) | Ty::Duration => Some(true),
        Ty::Bool | Ty::Char(_) => Some(false),
        _ => None,
    }
}

/// A value the runtime holds behind a handle.
fn is_handle(t: &Ty) -> bool {
    matches!(t.base(), Ty::Str(_) | Ty::Char(crate::types::Unit::Grapheme) | Ty::Seq { .. } | Ty::Map { .. })
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

/// This machine's triple, with the OS version the linker will use. LLVM
/// reads a macOS version off the Darwin one, and on this machine it reads one
/// too many (darwin27 → macOS 28 on macOS 27), so every link warned that the
/// object was built for a newer macOS. `cc` links for the deployment target,
/// or else the running macOS; the object says the same.
fn host_triple() -> inkwell::targets::TargetTriple {
    static TRIPLE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let t = TRIPLE.get_or_init(|| {
        let default = TargetMachine::get_default_triple().as_str().to_string_lossy().into_owned();
        if !cfg!(target_os = "macos") {
            return default;
        }
        let version = std::env::var("MACOSX_DEPLOYMENT_TARGET").ok().filter(|v| !v.is_empty()).or_else(|| {
            let out = std::process::Command::new("sw_vers").arg("-productVersion").output().ok()?;
            Some(String::from_utf8(out.stdout).ok()?.trim().to_string())
        });
        match (version, default.split_once("-apple-")) {
            (Some(v), Some((arch, _))) if !v.is_empty() => format!("{}-apple-macosx{}", arch, v),
            _ => default,
        }
    });
    inkwell::targets::TargetTriple::create(t)
}

/// Write the module as an object file for this machine.
/// This machine, described: the triple, the CPU and what it can do.
fn machine() -> Result<TargetMachine, String> {
    Target::initialize_native(&InitializationConfig::default()).map_err(|e| e.to_string())?;
    let triple = host_triple();
    let target = Target::from_triple(&triple).map_err(|e| e.to_string())?;
    target
        .create_target_machine(&triple, &TargetMachine::get_host_cpu_name().to_string(), &TargetMachine::get_host_cpu_features().to_string(), OptimizationLevel::Aggressive, RelocMode::PIC, CodeModel::Default)
        .ok_or_else(|| "this machine has no target description".to_string())
}

pub fn write_object(module: &Module, path: &Path) -> Result<(), String> {
    let machine = machine()?;
    module.set_triple(&host_triple());
    module.set_data_layout(&machine.get_target_data().get_data_layout());
    // LLVM's whole pipeline, as clang -O3 runs it (owner: optimised
    // everywhere). Without it the module went to the backend as written —
    // every local in memory, nothing inlined, no loop touched — and ran at
    // up to 5.6x C's time. Precog's proofs reach it as `nsw`/`nuw` and as
    // checks that are simply not there.
    module
        .run_passes("default<O3>", &machine, inkwell::passes::PassBuilderOptions::create())
        .map_err(|e| e.to_string())?;
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
