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

mod bits;
mod cabi;
mod clayout;
mod cstr;
mod deci;
mod heap;
pub mod bare;
mod par;
mod print;
pub mod size;
mod tasks;

/// Something the lowering cannot do yet, named so the writer knows what
/// rather than seeing a crash.
pub struct NotYet {
    pub what: String,
    pub span: Option<crate::source::Span>,
}

pub struct Lowered<'ctx> {
    pub module: Module<'ctx>,
    /// Every struct the program holds, laid out for the target: for the
    /// build report's Sizes (owner, 2026-10-19).
    pub structs: Vec<StructLayout>,
    /// Every kit-level `const` with a size the target gives it.
    pub consts: Vec<(String, String, u64)>,
}

/// One struct as the target lays it out.
pub struct StructLayout {
    pub name: String,
    pub size: u64,
    pub align: u32,
    /// Each field: its name, type, where it starts, its size, and the
    /// padding after it.
    pub fields: Vec<(String, String, u64, u64, u64)>,
}

/// What the object's writing did, for the build report.
#[derive(Default)]
pub struct ObjectFacts {
    /// Calls the recursive inliner inlined (`llvm-passes/`).
    pub recursion_inlined: u32,
    /// Functions whose optimised code works on vectors (SIMD), by symbol.
    pub vector_fns: Vec<String>,
    pub optimize_time: std::time::Duration,
    pub emit_time: std::time::Duration,
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
    lower_for(ctx, p, sources, name, assert_proofs, false)
}

/// `tests`: the binary `cca test` runs (owner, 2026-09-29) — every `test
/// fn`, its entry running the one its first argument numbers, in the order
/// written. Otherwise the tests are left out and the entry runs `MAIN`.
pub fn lower_for<'ctx>(ctx: &'ctx Context, p: &Program, sources: &SourceMap, name: &str, assert_proofs: bool, tests: bool) -> Result<Lowered<'ctx>, Vec<NotYet>> {
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
    let mut cg = Cg { ctx, module: &module, builder: ctx.create_builder(), p, sources, fns: HashMap::new(), env_fns: HashMap::new(), externs: HashMap::new(), problems: Vec::new(), strings: HashMap::new(), target, helpers: HashMap::new(), cur_flags: Vec::new(), cur_groups: Vec::new(), cur_outcomes: HashMap::new(), assert_proofs, here: (0, 0), ranges_trusted: None, tests, cur_fi: 0, const_arrays: HashMap::new(), op_sites: Vec::new(), par: crate::par::plan(p), par_child: None, nocheat: nocheat_blocks(p), bits: crate::mir::bits::bit_lists(p) };
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
        return Err(vec![NotYet { what: format!("LLVM refused the code kayo generated — this is a bug in kayo, not in the program:\n       {}", e.to_string().replace('\n', "\n       ")), span: None }]);
    }
    let structs = cg.struct_layouts();
    let consts = cg.const_sizes();
    Ok(Lowered { module, structs, consts })
}

struct Cg<'a, 'ctx> {
    /// Each const array's elements in the program, made once, by its value.
    const_arrays: HashMap<String, (PointerValue<'ctx>, (u64, u64))>,
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
    /// Whether any range in the program rests on someone's word
    /// (`guaranteed_range`), worked out once.
    ranges_trusted: Option<bool>,
    /// Building the test binary (`lower_for`).
    tests: bool,
    /// The function being written.
    cur_fi: usize,
    /// The check sites guarding the operation being lowered.
    op_sites: Vec<SiteId>,
    /// Parallel `for` (`crate::par`): the loops that run in slices.
    par: crate::par::Plan,
    /// The slice being written, when one is.
    par_child: Option<par::ParChild<'ctx>>,
    /// The blocks of `for !cheat` loops: (function, block).
    nocheat: std::collections::HashSet<(usize, usize)>,
    /// The `list[bool]` locals held as bits (`mir::bits`).
    bits: std::collections::HashSet<(usize, u32)>,
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
            // An exact answer is held in its limit's width and one more.
            Ty::Int(IntW::Fixed { signed, bits }) => Some(self.ctx.custom_width_int_type(std::num::NonZeroU32::new(crate::resolve::storage(*signed, *bits).1)?).ok()?.into()),
            // A width Precog left free reaches here only if nothing ever
            // pinned it; the machine's natural one is the rule
            // (`core-features.md`: a lone scalar takes the natural width).
            Ty::Int(IntW::Free(_)) => Some(self.ctx.i64_type().into()),
            Ty::Bool => Some(self.ctx.bool_type().into()),
            Ty::Bin(BinW::Fixed(16)) => Some(self.ctx.f16_type().into()),
            Ty::Brain => Some(self.ctx.bf16_type().into()),
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
                // A C union, or a C struct with a bit-field: C's own layout.
                if let Some(l) = self.c_layout(t) {
                    return Some(self.c_layout_shape(&l));
                }
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
        // A `no_cheat` copy of a function that captures takes its captures too.
        for (i, f) in self.p.functions.iter().enumerate() {
            if let Some(n) = f.nocheat_of.and_then(|o| self.env_fns.get(&(o.0 as usize)).copied()) {
                self.env_fns.insert(i, n);
            }
        }
        for (i, f) in self.p.functions.iter().enumerate() {
            // A normal build leaves the tests out (owner, 2026-09-29).
            if f.is_main || (f.is_test && !self.tests) {
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
            // The program is one module and only `main` is called from
            // outside it: its own functions are internal, which lets LLVM
            // inline one into its caller and see what it checked (the range
            // `read_n` in bench/sieve.cpls establishes). Their addresses
            // still go to C and to tasks as values.
            let v = self.module.add_function(&name, fty, Some(inkwell::module::Linkage::Internal));
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
                self.cannot(format!("the `extern.C fn {}` takes a callback that passes or returns a struct by value, which needs a thunk in C's calling convention, and that is not built yet", e.name), None);
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
            // C's `...`: the fixed parameters are the function's type, and
            // the extras travel as C's variadic ones do (owner, 2026-10-19:
            // on Apple silicon apart from the fixed ones).
            let fixed = e.var_from.unwrap_or(e.params.len());
            let vararg = e.variadic || e.var_from.is_some();
            for (_, t) in &e.params[..fixed] {
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
                self.cannot(format!("the `extern.C fn {}` takes something the generated code has no shape for yet", e.name), None);
                continue;
            }
            let fty = match &e.ret {
                Ty::Nothing => self.ctx.void_type().fn_type(&params, vararg),
                _ if sret.is_some() => self.ctx.void_type().fn_type(&params, vararg),
                r => match (self.ty(&r.base().clone()), self.c_pass(&r.base().clone(), true)) {
                    (Some(t), Some(cabi::CPass::Direct)) => t.fn_type(&params, vararg),
                    (Some(_), Some(cabi::CPass::Coerce(c))) => c.fn_type(&params, vararg),
                    _ => {
                        self.cannot(format!("the `extern.C fn {}` returns something the generated code has no shape for yet", e.name), None);
                        continue;
                    }
                },
            };
            // C's own name (`= C.printf`), declared once however many
            // names of ours it has.
            let symbol = e.c_name.clone().unwrap_or_else(|| e.name.clone());
            let v = match self.module.get_function(&symbol) {
                Some(f) if f.get_type() == fty => f,
                _ => self.module.add_function(&symbol, fty, Some(Linkage::External)),
            };
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
        if self.tests {
            self.emit_test_entry();
            return;
        }
        let Some(main_idx) = self.p.main else { return };
        let i32t = self.ctx.i32_type();
        let p = self.ctx.ptr_type(AddressSpace::default());
        let f = self.module.add_function("main", i32t.fn_type(&[i32t.into(), p.into()], false), None);
        let entry = self.ctx.append_basic_block(f, "entry");
        self.builder.position_at_end(entry);
        // C's arguments, kept where `MAIN` reads its parameters (owner,
        // 2026-09-29) — whether it runs directly or as a task.
        let (argc, argv) = self.main_args();
        let _ = self.builder.build_store(argc, f.get_nth_param(0).unwrap());
        let _ = self.builder.build_store(argv, f.get_nth_param(1).unwrap());
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

    /// Where C's `argc` and `argv` are kept for `MAIN`.
    fn main_args(&mut self) -> (PointerValue<'ctx>, PointerValue<'ctx>) {
        let i32t = self.ctx.i32_type();
        let p = self.ctx.ptr_type(AddressSpace::default());
        let argc = self.module.get_global("cy_argc").unwrap_or_else(|| {
            let g = self.module.add_global(i32t, None, "cy_argc");
            g.set_linkage(inkwell::module::Linkage::Internal);
            g.set_initializer(&i32t.const_zero());
            g
        });
        let argv = self.module.get_global("cy_argv").unwrap_or_else(|| {
            let g = self.module.add_global(p, None, "cy_argv");
            g.set_linkage(inkwell::module::Linkage::Internal);
            g.set_initializer(&p.const_null());
            g
        });
        (argc.as_pointer_value(), argv.as_pointer_value())
    }

    /// The test binary's `main`: the test its first argument numbers, run
    /// as `MAIN` would be (as a task when the program has any), then 0.
    /// A `must` that does not hold, or a check, ends it sooner.
    fn emit_test_entry(&mut self) {
        let i32t = self.ctx.i32_type();
        let i64t = self.ctx.i64_type();
        let p = self.ctx.ptr_type(AddressSpace::default());
        let tests: Vec<usize> = self.p.functions.iter().enumerate().filter(|(_, f)| f.is_test).map(|(i, _)| i).collect();
        // Which test, where the task-run entry can read it.
        let which = self.module.add_global(i64t, None, "cy_test_which");
        which.set_initializer(&i64t.const_zero());
        let run = self.module.add_function("cy_test_run", self.ctx.void_type().fn_type(&[], false), Some(inkwell::module::Linkage::Internal));
        let entry = self.ctx.append_basic_block(run, "entry");
        self.builder.position_at_end(entry);
        let n = self.builder.build_load(i64t, which.as_pointer_value(), "which").unwrap().into_int_value();
        let done = self.ctx.append_basic_block(run, "done");
        let mut cases = Vec::new();
        for (k, fi) in tests.iter().enumerate() {
            let b = self.ctx.append_basic_block(run, "test");
            cases.push((i64t.const_int(k as u64, false), b));
            self.builder.position_at_end(b);
            if let Some(f) = self.fns.get(fi).copied() {
                let _ = self.builder.build_call(f, &[], "");
            }
            let _ = self.builder.build_unconditional_branch(done);
        }
        self.builder.position_at_end(entry);
        let _ = self.builder.build_switch(n, done, &cases);
        self.builder.position_at_end(done);
        let _ = self.builder.build_return(None);
        let main = self.module.add_function("main", i32t.fn_type(&[i32t.into(), p.into()], false), None);
        let e = self.ctx.append_basic_block(main, "entry");
        self.builder.position_at_end(e);
        let index = self.module.get_function("cyborg_test_index").unwrap_or_else(|| self.module.add_function("cyborg_test_index", i64t.fn_type(&[i32t.into(), p.into()], false), None));
        let argc = main.get_nth_param(0).unwrap();
        let argv = main.get_nth_param(1).unwrap();
        let k = self.builder.build_call(index, &[argc.into(), argv.into()], "k").ok().and_then(basic_result).unwrap();
        let _ = self.builder.build_store(which.as_pointer_value(), k);
        if self.uses_tasks() {
            let task_run = self.module.get_function("cyborg_run").unwrap_or_else(|| self.module.add_function("cyborg_run", self.ctx.void_type().fn_type(&[p.into()], false), None));
            let _ = self.builder.build_call(task_run, &[run.as_global_value().as_pointer_value().into()], "");
        } else {
            let _ = self.builder.build_call(run, &[], "");
        }
        let _ = self.builder.build_return(Some(&i32t.const_zero()));
    }

    // ------------------------------------------------------------ bodies

    fn define(&mut self, i: usize) {
        let f = &self.p.functions[i];
        // The test binary runs the tests, never `MAIN`.
        if f.is_main && self.tests {
            return;
        }
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
        // A table Precog made (owner, 2026-10-17): the body is a lookup.
        // Never in the build that checks the proofs.
        if !self.assert_proofs && !self.env_fns.contains_key(&i) {
            if let Some(t) = self.p.tables.iter().find(|t| t.function == i) {
                if self.define_table(i, func, t).is_some() {
                    return;
                }
                self.cannot(format!("the table of `{}` has no shape in the generated code yet", f.name), None);
                return;
            }
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
                let slot = self.stack(self.ctx.ptr_type(AddressSpace::default()), &name).unwrap();
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
                    let slot = self.stack(t, &name).unwrap();
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
            let flag = self.stack(self.ctx.bool_type(), "alive").unwrap();
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
        // `MAIN ['argc': i32, 'argv': ptr[ptr[u8]]]`: its parameters
        // are what C's `main` kept.
        if f.is_main && f.params.len() == 2 {
            let (argc, argv) = self.main_args();
            let loaded = [
                self.builder.build_load(self.ctx.i32_type(), argc, "argc").ok(),
                self.builder.build_load(self.ctx.ptr_type(AddressSpace::default()), argv, "argv").ok(),
            ];
            for (p, v) in f.params.iter().zip(loaded) {
                if let (Some(slot), Some(v)) = (slots[p.0 as usize], v) {
                    let _ = self.builder.build_store(slot, v);
                }
            }
        }
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
        // A `no_cheat` copy's parameters are unknown to LLVM, and taking
        // them is a step it may not skip: its work is not folded to the
        // answer, dropped, or shared with a look-alike call (owner,
        // 2026-10-18).
        if f.nocheat {
            for p in &f.params {
                if !lent[p.0 as usize] {
                    self.no_cheat_barrier(i, *p, &slots);
                }
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
        // `for !cheat` (owner, 2026-10-01): each store to a local in the loop
        // passes through a barrier LLVM cannot see into, so no round's
        // work is folded away or dropped.
        let nocheat = self.nocheat.contains(&(fi, b));
        let mut barrier_due: Option<LocalId> = None;
        // Where a `no_cheat` statement starts: what it touches is unknown to
        // LLVM from there on.
        let entries: Vec<(u32, Vec<LocalId>)> = f.nocheat_entries.iter().filter(|e| e.0 as usize == b).map(|e| (e.1, e.2.clone())).collect();
        // A counted loop's step is how it counts, not its work: known to
        // LLVM under `no_cheat` too, so a `by 1` loop takes the plain form
        // (bench sum ran 1.56× slow with its step behind a barrier).
        let steps: std::collections::HashSet<LocalId> = f.fors.iter().filter_map(|fl| fl.step).collect();
        let starts_at = |this: &mut Self, si: usize| {
            for (at, locals) in &entries {
                if *at as usize == si {
                    for l in locals {
                        if !steps.contains(l) {
                            this.no_cheat_barrier(fi, *l, slots);
                        }
                    }
                }
            }
        };
        for (si, s) in stmts.iter().enumerate() {
            if let Some(l) = barrier_due.take() {
                self.no_cheat_barrier(fi, l, slots);
            }
            starts_at(self, si);
            if nocheat {
                if let StmtKind::Assign { place, .. } = &s.kind {
                    if place.proj.is_empty() && !steps.contains(&place.local) {
                        barrier_due = Some(place.local);
                    }
                }
            }
            self.here = (b as u32, si as u32);
            // In a slice: a fold's store and sites are left out, and its
            // add goes to the slice's own partial (`codegen::par`).
            if let Some(pc) = &self.par_child {
                if pc.skip.contains(&(b, si)) {
                    continue;
                }
                if let Some((which, x)) = pc.adds.get(&(b, si)).cloned() {
                    if self.par_fold_add(fi, which, &x, slots, func).is_none() {
                        self.cannot("this sum in a slice has no shape in the generated code yet", Some(f.span(s.span)));
                    }
                    continue;
                }
            }
            match &s.kind {
                StmtKind::Site(id) => match f.site(*id).kind {
                    // These stand on their own: they guard a value that is
                    // already there, not the instruction after them.
                    SiteKind::Bounds => self.bounds_check(fi, *id, slots, func),
                    SiteKind::SliceRange => self.slice_check(fi, *id, slots, func),
                    // Both ask only whether a number is below 0.
                    SiteKind::NegativeExponent | SiteKind::FillCount => self.negative_exponent_check(fi, *id, slots, func),
                    SiteKind::Position => self.position_check(fi, *id, slots, func),
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
                    // A borrow passed along is a read of another borrow; a
                    // read of a plain value (`change 'x' = 'x' x 2` over
                    // `lend.mut 'xs'`) writes *through* the borrow. Taking
                    // every read for a rebinding pointed 'x' at the
                    // temporary and left the list as it was (owner,
                    // 2026-09-30, `for 'x' in lend.mut`).
                    let binds_borrow = binds_borrow
                        && match rvalue {
                            Rvalue::Lend { .. } => true,
                            Rvalue::Use(o) => o.place().is_some_and(|p| p.proj.is_empty() && matches!(f.locals[p.local.0 as usize].kind, LocalKind::Lent { .. })),
                            _ => false,
                        };
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
                    } else if want.is_some() {
                        // Never drop a store quietly: the place would be read
                        // unset (a simd lane read did this).
                        self.cannot("this value has no shape in the generated code yet", Some(f.span(s.span)));
                    }
                }
                StmtKind::Op { dest, op, args } => {
                    // The sites guarding this operation (`simd.sum`'s).
                    self.op_sites = std::mem::take(&mut pending);
                    let dest_ty = dest.as_ref().and_then(|d| self.slot_of(fi, d, slots)).map(|(_, t)| t);
                    let as_bits = matches!(op, Op::ListFill) && dest.as_ref().is_some_and(|d| d.proj.is_empty() && self.bits.contains(&(fi, d.local.0)));
                    let done = if as_bits { dest_ty.and_then(|t| self.bit_fill(fi, args, &t, slots, func)).map(Some) } else { self.op(fi, op, args, slots, func, dest_ty) };
                    match done {
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
                StmtKind::Asm { text, ins, outs, lends, clobbers } => {
                    if self.inline_asm(fi, text, ins, outs, lends, clobbers, slots).is_none() {
                        self.cannot("this `inline.ASM` has no shape in the generated code yet", Some(f.span(s.span)));
                    }
                }
            }
        }
        if let Some(l) = barrier_due.take() {
            self.no_cheat_barrier(fi, l, slots);
        }
        starts_at(self, stmts.len());
        // The sites guarding a call that ends the block (`map.get`'s).
        self.op_sites = std::mem::take(&mut pending);
        let term = f.blocks[b].term.clone();
        self.here = (b as u32, f.blocks[b].stmts.len() as u32);
        match &term {
            Terminator::Goto(t) => {
                // Into a loop that may run in slices (`crate::par`).
                let planned = self.par.loops.get(&(fi, b)).filter(|pl| pl.fl.head == *t).cloned();
                match planned {
                    Some(pl) => {
                        if self.par_dispatch(fi, &pl, blocks, slots, func).is_none() {
                            self.cannot("this parallel `for` has no shape in the generated code yet", Some(f.span(pl.fl.span)));
                            let _ = self.builder.build_unconditional_branch(blocks[t.0 as usize]);
                        }
                    }
                    None => {
                        let to = back_to_body(f, b, *t, self.par_child.is_some()).unwrap_or(*t);
                        let br = self.builder.build_unconditional_branch(blocks[to.0 as usize]);
                        // The way round a loop that may run in slices, in the
                        // copy that runs in order: that copy only ever takes
                        // the short runs, and vectorising it hoisted its
                        // 128-bit trip arithmetic into the loop around it —
                        // once per outer round, the sieve at two thirds speed.
                        // Only a loop whose step the program works out: one
                        // counting by a number written is vectorised as any
                        // loop is (the in-order copy of every loop that may
                        // run in slices was kept scalar, two lists added
                        // element by element among them).
                        let back = self.par_child.is_none() && self.par.loops.iter().any(|((g, _), pl)| *g == fi && pl.fl.head == *t && pl.blocks.contains(&b) && pl.fl.step.is_some_and(|st| !step_is_const(f, st)));
                        if let (true, Ok(br)) = (back, br) {
                            self.no_vectorize(br);
                        }
                    }
                }
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
            Terminator::Call { callee: Callee::Op(op @ (Op::CStrMake | Op::CStrRead)), args, dest, ok, fail } if !fail.is_empty() => {
                if self.cstr_call(fi, op, args, dest, *ok, fail, blocks, slots, func).is_none() {
                    self.cannot("this C string has no shape in the generated code yet", Some(f.span(f.blocks[b].term_span)));
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
                    // Say whose call: a std one waits on the std Kit, not
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
                    let v = match callee {
                        Callee::Fn(idx) => {
                            let ret = self.p.functions[idx.0 as usize].ret.clone();
                            self.clamp_answer(v, &ret)
                        }
                        _ => v,
                    };
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
                    // A union's field, or a field of a struct with
                    // bit-fields: where C puts it (a bit-field's unit).
                    if let Some(l) = self.c_layout(&ty) {
                        slot = self.c_field_at(slot, l.fields[*i])?;
                        ty = crate::mir::build::projected_ty(&ty, std::slice::from_ref(pr), &self.p.items);
                        continue;
                    }
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
                    // One lane of a vector (`simd.md`: `'v'[1]`, 1-based):
                    // in memory a vector is its lanes one after another.
                    // Only a list's index was known here, and a lane read
                    // came back with nothing — the assignment was dropped
                    // and the variable read unset (check claude).
                    if let Ty::Simd { elem, .. } = ty.base().clone() {
                        if elem.base() == &Ty::Bool {
                            self.cannot("one lane of a mask (`simd[bool, count N]`) has no shape in the generated code yet", None);
                            return None;
                        }
                        let et = self.ty(&elem)?;
                        let w = self.ctx.i64_type();
                        let i = self.builder.build_int_cast_sign_flag(idx, w, false, "lane").ok()?;
                        let zb = self.builder.build_int_sub(i, w.const_int(1, false), "zb").ok()?;
                        slot = unsafe { self.builder.build_gep(et, slot, &[zb], "lane").ok()? };
                        ty = (*elem).clone();
                        continue;
                    }
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
            Operand::Read(p) | Operand::Move(p) if self.is_bit(fi, p) => self.bit_read(fi, p, slots),
            // A C bit-field: its bits.
            Operand::Read(p) | Operand::Move(p) if self.c_field(fi, p).is_some_and(|c| c.2.width.is_some()) => {
                let (outer, _, f, ft) = self.c_field(fi, p)?;
                let (base, _) = self.slot_of(fi, &outer, slots)?;
                let at = self.c_field_at(base, f)?;
                self.c_bits_read(at, f, &ft)
            }
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
            // it signed — `10000000000000000000` is a fine `u64`, and
            // built at 64 bits and then widened it would read as negative.
            Const::Int(v) if *v > i64::MAX as i128 || *v < i64::MIN as i128 => {
                let t = self.ctx.i128_type();
                let lo = *v as u128 as u64;
                let hi = ((*v as u128) >> 64) as u64;
                Some(t.const_int_arbitrary_precision(&[lo, hi]).into())
            }
            Const::Int(v) => Some(self.ctx.i64_type().const_int(*v as u64, true).into()),
            // Past `i128`: at the width its digits need, signed, for the
            // place to widen (owner, 2026-10-03).
            Const::Big(v) => {
                let b = crate::bigint::Big::parse(v)?;
                let t = self.ctx.custom_width_int_type(std::num::NonZeroU32::new(b.bits() as u32 + 1)?).ok()?;
                let digits = v.trim_start_matches('-');
                let m = t.const_int_from_string(digits, inkwell::types::StringRadix::Decimal)?;
                Some(if b.is_neg() { m.const_neg().into() } else { m.into() })
            }
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
            // walked an `i32` list 8 bytes at a time.
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
            // A const array as a value: a fresh array of its own (owner,
            // 2026-10-10: "every use is a fresh value"), copied from the
            // program's.
            Const::Bytes(_) | Const::Array { .. } => {
                let (g, n) = self.const_array(c)?;
                let i64t = self.ctx.i64_type();
                let bytes = i64t.const_int(n.1, false);
                let mem = self.alloc(bytes);
                if n.1 > 0 {
                    let _ = self.builder.build_memcpy(mem, 1, g, 1, bytes);
                }
                let h = self.handle(2);
                let v = self.builder.build_insert_value(h.get_undef(), mem, 0, "data").ok()?;
                let v = self.builder.build_insert_value(v, i64t.const_int(n.0, false), 1, "len").ok()?;
                Some(v.into_struct_value().into())
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
    /// No bin or brain16 result is ever `-0` (owner, 2026-10-07: "`a`"):
    /// an operation that could make one adds `+0`, which turns `-0` into `0`
    /// and leaves every other value as it is (LLVM drops it where it can
    /// tell the answer is no `-0`). The `.ieee` forms keep IEEE's own
    /// answer ("yea"), and are otherwise the plain operators.
    fn rvalue(&mut self, fi: usize, rv: &Rvalue, slots: &[Option<PointerValue<'ctx>>], sites: &[SiteId], func: FunctionValue<'ctx>, blocks: &[LlBlock<'ctx>], want: Option<BasicTypeEnum<'ctx>>, want_ty: Option<Ty>) -> Option<BasicValueEnum<'ctx>> {
        let plain;
        let (rv, zeroable) = match rv {
            Rvalue::Binary { op, lhs, rhs } if op.is_ieee() => {
                plain = Rvalue::Binary { op: op.plain(), lhs: lhs.clone(), rhs: rhs.clone() };
                (&plain, false)
            }
            Rvalue::Unary { op: UnOp::NegIeee, operand } => {
                plain = Rvalue::Unary { op: UnOp::Neg, operand: operand.clone() };
                (&plain, false)
            }
            Rvalue::Binary { op: BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div(_) | BinOp::Mod(_) | BinOp::Pow, .. } | Rvalue::Unary { op: UnOp::Neg, .. } => (rv, true),
            _ => (rv, false),
        };
        let v = self.rvalue_raw(fi, rv, slots, sites, func, blocks, want, want_ty)?;
        Some(if zeroable { self.no_negative_zero(v)? } else { v })
    }

    /// `x + 0`, for a bin or a vector of them: IEEE's `-0` becomes `0`.
    pub(super) fn no_negative_zero(&mut self, v: BasicValueEnum<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        Some(match v {
            BasicValueEnum::FloatValue(x) => self.builder.build_float_add(x, x.get_type().const_zero(), "no_negative_zero").ok()?.into(),
            BasicValueEnum::VectorValue(x) if x.get_type().get_element_type().is_float_type() => {
                self.builder.build_float_add(x, x.get_type().const_zero(), "no_negative_zero").ok()?.into()
            }
            other => other,
        })
    }

    fn rvalue_raw(&mut self, fi: usize, rv: &Rvalue, slots: &[Option<PointerValue<'ctx>>], sites: &[SiteId], func: FunctionValue<'ctx>, blocks: &[LlBlock<'ctx>], want: Option<BasicTypeEnum<'ctx>>, want_ty: Option<Ty>) -> Option<BasicValueEnum<'ctx>> {
        match rv {
            Rvalue::Use(o) => {
                // A bin literal landing in an f16 or f32 is written at
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
                    // A negative literal is written at its place's width, as
                    // a positive one is (a brain16 -0.3 was stored as a
                    // f64 into two bytes); anything else is cast there.
                    (UnOp::Neg, BasicValueEnum::FloatValue(x)) => {
                        if let (Operand::Const(Const::Bin(text)), Some(BasicTypeEnum::FloatType(t))) = (operand, want) {
                            let neg = format!("-{}", text.replace('_', ""));
                            return Some(unsafe { t.const_float_from_string(&neg) }.into());
                        }
                        let n = self.builder.build_float_neg(x, "neg").ok()?;
                        match want {
                            Some(BasicTypeEnum::FloatType(t)) if t != n.get_type() => Some(self.float_cast(n, t)?.into()),
                            _ => Some(n.into()),
                        }
                    }
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
                    let pa = self.stack(a.get_type(), "eq_a").ok()?;
                    let pb = self.stack(b.get_type(), "eq_b").ok()?;
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
                // numbers going into a `u64` is an unsigned 64-bit
                // multiply; checked as signed, 2 x 7726793855274730819 was
                // an overflow that never happens. (The ordinary build folds
                // it to its answer; `--assert-proofs` computes it.)
                // A shift of an exact answer works at the width of its typed
                // operands (owner, 2026-10-03), the type it lands in here:
                // the value has to fit that width before it is shifted, and
                // that is the shift's own site.
                if matches!(op, BinOp::Shl | BinOp::Shr) {
                    if let (BasicValueEnum::IntValue(x), Some(Ty::Int(IntW::Fixed { signed: ws, bits: wb }))) = (a, want_ty.as_ref().map(|t| t.base().clone())) {
                        let width = crate::resolve::storage(ws, wb).1;
                        if self.operand_cy_ty(fi, lhs).is_some_and(|t| t.is_exact()) && x.get_type().get_bit_width() != width {
                            let (lo, hi) = crate::bigint::type_range(ws, wb);
                            let cw = x.get_type().get_bit_width().max(width) + 1;
                            let ct = self.ctx.custom_width_int_type(std::num::NonZeroU32::new(cw)?).ok()?;
                            let xs = self.builder.build_int_cast_sign_flag(x, ct, true, "exact").ok()?;
                            let open = sites.iter().copied().find(|s| matches!(self.p.functions[fi].site(*s).kind, SiteKind::ShiftWidth | SiteKind::ShiftCount) && self.gets_check(&self.p.functions[fi].site(*s).verdict));
                            if let (Some(s), Some(lo), Some(hi)) = (open, const_big(ct, &lo), const_big(ct, &hi)) {
                                let below = self.builder.build_int_compare(IntPredicate::SLT, xs, lo, "below").ok()?;
                                let above = self.builder.build_int_compare(IntPredicate::SGT, xs, hi, "above").ok()?;
                                let out = self.builder.build_or(below, above, "too_big_to_shift").ok()?;
                                self.guard(fi, s, out, func)?;
                            }
                            let t = self.ctx.custom_width_int_type(std::num::NonZeroU32::new(width)?).ok()?;
                            let nx = self.builder.build_int_truncate_or_bit_cast(xs, t, "at_width").ok()?;
                            return self.binary(fi, op, nx.into(), b, sites, func, blocks, ws);
                        }
                    }
                }
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
                                // Inside both ranges, both promises hold,
                                // and LLVM needs the signed one to see a
                                // counter is never negative (owner,
                                // 2026-09-29: c): `'i' mod 1000` in
                                // bench/lists.cpls was a signed floored `mod`
                                // with only `nuw` on `'i' + 1`, 7% behind C.
                                if unsigned && hi <= i64::MAX as i128 && matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul) {
                                    if let Some(i) = r.as_instruction() {
                                        let _ = i.set_no_signed_wrap_flag(true);
                                    }
                                }
                                let t = self.ty(want_ty.as_ref()?)?.into_int_type();
                                return Some(self.builder.build_int_cast_sign_flag(r, t, !unsigned, "exact").ok()?.into());
                            }
                        }
                        let plain = matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow);
                        // Its limit in between: 64 bits, or the widest
                        // operand's (owner, 2026-10-03). `+ - x` work at a
                        // width no two values in the window can overflow;
                        // `div` and `mod` at the window's own.
                        let limit = want_ty.as_ref().and_then(|t| t.exact_limit()).unwrap_or(64);
                        let work_bits = if plain { (2 * (limit + 1)).max(128) } else { limit + 1 };
                        let work = self.ctx.custom_width_int_type(std::num::NonZeroU32::new(work_bits)?).ok()?;
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
                        // Past 128 bits a sum or product cannot overflow its
                        // work width, and LLVM's overflow intrinsics need a
                        // library call there: only the window is asked.
                        let own = if work_bits > 128 && !matches!(op, BinOp::Pow) && plain { &[][..] } else { sites };
                        let r = self.binary(fi, op, x.into(), y.into(), own, func, blocks, true)?.into_int_value();
                        if plain {
                            self.past_limit(fi, r, limit, sites, func);
                        }
                        let t = self.ty(want_ty.as_ref()?)?.into_int_type();
                        return Some(if r.get_type().get_bit_width() == t.get_bit_width() { r.into() } else { self.builder.build_int_truncate(r, t, "exact").ok()?.into() });
                    }
                }
                // A comparison is of the two numbers, whatever their types:
                // each side is extended by its *own* sign, one bit wider
                // when the signs differ, and compared signed. Extending both
                // by the left side's sign made a `u8` of 201 into -55,
                // and `198 > 201` came out true (check claude, seed 77).
                if op.is_comparison() {
                    if let (BasicValueEnum::IntValue(x), BasicValueEnum::IntValue(y)) = (a, b) {
                        // A literal is compared at its written value, never
                        // squeezed into the other side's width: `'v' > 200`
                        // on an `i8` is false, not a comparison with -56.
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
                        if la.is_some() != lb.is_some() || mixed {
                            // One bit past the widest side, at least 128:
                            // any width (owner, 2026-10-03).
                            let w = self.ctx.custom_width_int_type(std::num::NonZeroU32::new((widest + 1).max(128))?).ok()?;
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
                // 64 bits, as in the oracle: `'f' > 0.1` with an f32 0.1 is
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
            Rvalue::Convert { to, operand, useful, .. } => {
                let mut v = self.operand(fi, operand, slots)?;
                // A constant has its family, not a place's type: a const's
                // whole number into a deci went in as raw bits (check
                // claude, seed 48186: 1234 printed as 0.000…0001234).
                let from = match operand {
                    Operand::Const(Const::Int(_)) => Some(Ty::exact()),
                    Operand::Const(Const::Bin(_)) => Some(Ty::Bin(crate::types::BinW::Fixed(64))),
                    Operand::Const(Const::Deci { bits, .. }) => Some(Ty::Deci(*bits)),
                    _ => self.operand_cy_ty(fi, operand),
                };
                if let Some(from) = from {
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
                    return self.to_text_as(fi, operand, v, to, *useful);
                }
                let t = self.ty(&to.clone())?;
                // Widening extends by what the value *is*, not by what it
                // becomes: a `u8` of 250 widened to 128 bits is 250, and
                // sign-extending it made it -6.
                let from_signed = self.operand_signed(fi, operand).unwrap_or(true);
                // Past 128 bits into a bin or brain16: rounded once from its
                // digits (owner, 2026-10-03), as text into a bin is — LLVM
                // has no library call for the wide conversion. Past the
                // largest is the store's own site, before this.
                if let (BasicValueEnum::IntValue(i), BasicTypeEnum::FloatType(ft)) = (v, t) {
                    if i.get_type().get_bit_width() > 128 {
                        let code = match to.base() {
                            Ty::Bin(crate::types::BinW::Fixed(b)) => *b as u64,
                            Ty::Brain => crate::types::BRAIN_BITS as u64,
                            _ => 64,
                        };
                        let text = self.wide_int_text(i, from_signed)?;
                        let (p, n) = self.text_parts_pub(text)?;
                        let f64t = self.ctx.f64_type();
                        let out = self.stack(f64t, "parsed").ok()?;
                        let f = self.rt("cyborg_parse_bin");
                        let _ = self.builder.build_call(f, &[p.into(), n.into(), self.ctx.i32_type().const_int(code, false).into(), out.into()], "status");
                        let free = self.rt("cyborg_str_free");
                        let _ = self.builder.build_call(free, &[p.into(), n.into()], "");
                        let x = self.builder.build_load(f64t, out, "value").ok()?.into_float_value();
                        return Some(self.float_cast(x, ft)?.into());
                    }
                }
                match (v, t) {
                    (BasicValueEnum::IntValue(i), BasicTypeEnum::IntType(it)) => Some(self.builder.build_int_cast_sign_flag(i, it, from_signed, "conv").ok()?.into()),
                    (BasicValueEnum::IntValue(i), BasicTypeEnum::FloatType(ft)) if !from_signed => Some(self.builder.build_unsigned_int_to_float(i, ft, "conv").ok()?.into()),
                    (BasicValueEnum::IntValue(i), BasicTypeEnum::FloatType(ft)) => Some(self.builder.build_signed_int_to_float(i, ft, "conv").ok()?.into()),
                    (BasicValueEnum::FloatValue(x), BasicTypeEnum::FloatType(ft)) => Some(self.float_cast(x, ft)?.into()),
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
                if let Some(t) = want_ty.clone() {
                    if let Some(l) = self.c_layout(&t) {
                        return self.c_make(fi, &t, &l, fields, slots);
                    }
                }
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
                        let tmp = self.stack(shape, "copy").ok()?;
                        let _ = self.builder.build_call(copier, &[tmp.into(), src.into()], "");
                        self.builder.build_load(shape, tmp, "copied").ok()
                    }
                    None => self.operand(fi, &Operand::Read(p.clone()), slots),
                }
            }
            // Reading C's variable: a !check site, a plain load.
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
            // A shift works at the width of what it shifts; its count is a
            // count. A wider count is clamped to the width first, so one too
            // big still trips the count's check: widened instead, `u32
            // shl i64` shifted at 64 bits and lost bits went unseen.
            (BasicValueEnum::IntValue(x), BasicValueEnum::IntValue(y)) if matches!(op, BinOp::Shl | BinOp::Shr) && !x.is_const() && x.get_type().get_bit_width() != y.get_type().get_bit_width() => {
                let xt = x.get_type();
                let w = xt.get_bit_width();
                let y = if y.get_type().get_bit_width() > w {
                    let cap = y.get_type().const_int(w as u64, false);
                    let past = self.builder.build_int_compare(IntPredicate::UGT, y, cap, "count_past").ok()?;
                    let y = self.builder.build_select(past, cap, y, "count").ok()?.into_int_value();
                    self.builder.build_int_truncate(y, xt, "count").ok()?
                } else {
                    // Zero-extended: a negative count reads as a huge one,
                    // past the width, and trips the same check.
                    self.builder.build_int_z_extend(y, xt, "count").ok()?
                };
                (a, y.into())
            }
            (BasicValueEnum::IntValue(x), BasicValueEnum::IntValue(y)) if x.get_type().get_bit_width() != y.get_type().get_bit_width() => {
                // One side is a literal, which has no width of its own: it
                // takes the other's.
                // The literal is the one that moves, whichever is wider: a
                // `u64` times 10^19 is a `u64` multiply, and the
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
                // Each rounds as named (`operators.md`): `-7 div.floor 2 =
                // -4`, `-7 mod.floor 2 = 1`, `-7 div.trunc 2 = -3`.
                // The hardware's division (owner, 2026-10-03): the lowest
                // value by -1 is itself, which every rounding of it by 1
                // gives — so -1 is asked as 1 there, and nothing is undefined.
                BinOp::DivWrap(round) => {
                    if let Some(s) = divzero {
                        let zero = self.builder.build_int_compare(IntPredicate::EQ, y, y.get_type().const_zero(), "by_zero").ok()?;
                        self.guard(fi, s, zero, func)?;
                    }
                    let y = if signed {
                        let bits = x.get_type().get_bit_width();
                        let min = const_i128(x.get_type(), if bits >= 128 { i128::MIN } else { -(1i128 << (bits - 1)) });
                        let min = if bits > 128 { const_big(x.get_type(), &crate::bigint::Big::pow2(bits - 1).neg())? } else { min };
                        let is_min = self.builder.build_int_compare(IntPredicate::EQ, x, min, "is_min").ok()?;
                        let by_m1 = self.builder.build_int_compare(IntPredicate::EQ, y, y.get_type().const_all_ones(), "by_m1").ok()?;
                        let both = self.builder.build_and(is_min, by_m1, "min_by_m1").ok()?;
                        self.builder.build_select(both, y.get_type().const_int(1, false), y, "by").ok()?.into_int_value()
                    } else {
                        y
                    };
                    self.div_round(x, y, signed, false, *round)?.0
                }
                BinOp::Div(round) | BinOp::DivIeee(round) | BinOp::Mod(round) => {
                    // A zero divisor, and the lowest signed value divided by
                    // -1, are both undefined to LLVM: checked before the
                    // divide, never after.
                    if let Some(s) = divzero {
                        let zero = self.builder.build_int_compare(IntPredicate::EQ, y, y.get_type().const_zero(), "by_zero").ok()?;
                        self.guard(fi, s, zero, func)?;
                    }
                    if let (true, true, Some(s)) = (op.is_div(), signed, site) {
                        let bits = x.get_type().get_bit_width();
                        // The lowest value at this width, any width.
                        let min = const_i128(x.get_type(), if bits >= 128 { i128::MIN } else { -(1i128 << (bits - 1)) });
                        let is_min = self.builder.build_int_compare(IntPredicate::EQ, x, min, "is_min").ok()?;
                        let by_m1 = self.builder.build_int_compare(IntPredicate::EQ, y, y.get_type().const_all_ones(), "by_m1").ok()?;
                        let both = self.builder.build_and(is_min, by_m1, "min_by_m1").ok()?;
                        self.guard(fi, s, both, func)?;
                    }
                    let (v, stepped) = self.div_round(x, y, signed, op.is_mod(), *round)?;
                    // Unsigned, what a step up leaves is below 0.
                    if let (false, true, Some(s)) = (signed, op.is_mod(), site) {
                        self.guard(fi, s, stepped, func)?;
                    }
                    v
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
            // A bin never holds an infinity or a NaN (owner, 2026-09-29):
            // a zero divisor is asked first, then whether the answer is a
            // real number — `r - r` is 0 only for a finite `r`.
            if let (BinOp::Div(_) | BinOp::DivIeee(_) | BinOp::Mod(_), Some(s)) = (op, divzero) {
                let zero = self.builder.build_float_compare(FP::OEQ, y, y.get_type().const_zero(), "by_zero").ok()?;
                self.guard(fi, s, zero, func)?;
            }
            // A whole quotient, and what it leaves over (`div.near` is the
            // ordinary division, below).
            if let BinOp::Div(round) | BinOp::DivIeee(round) | BinOp::Mod(round) = op {
                if *round != crate::ast::Round::Near {
                    let v = self.bin_div_round(x, y, op.is_mod(), *round)?;
                    if let (Some(s), false) = (overflow, op.is_mod()) {
                        self.bin_finite_guard(fi, s, v, func)?;
                    }
                    return Some(v.into());
                }
            }
            if matches!(op, BinOp::Pow) {
                let id = inkwell::intrinsics::Intrinsic::find("llvm.pow")?;
                let f = id.get_declaration(self.module, &[x.get_type().into()])?;
                let r = basic_result(self.builder.build_call(f, &[x.into(), y.into()], "fpow").ok()?)?.into_float_value();
                if let Some(s) = overflow {
                    self.bin_finite_guard(fi, s, r, func)?;
                }
                return Some(r.into());
            }
            let r: BasicValueEnum = match op {
                BinOp::Add => self.builder.build_float_add(x, y, "fadd").ok()?.into(),
                BinOp::Sub => self.builder.build_float_sub(x, y, "fsub").ok()?.into(),
                BinOp::Mul => self.builder.build_float_mul(x, y, "fmul").ok()?.into(),
                BinOp::Div(_) | BinOp::DivIeee(_) => self.builder.build_float_div(x, y, "fdiv").ok()?.into(),
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
            if let (Some(s), BasicValueEnum::FloatValue(rv)) = (overflow, r) {
                if matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div(_) | BinOp::DivIeee(_)) {
                    self.bin_finite_guard(fi, s, rv, func)?;
                }
            }
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
        // With no operating system, a failed check freezes the CPU there,
        // on a trap instruction a debugger shows (owner, 2026-10-08: "yea,
        // freeze there"): LLVM's trap, `brk` on ARM.
        if crate::target::bare() {
            if let Some(trap) = inkwell::intrinsics::Intrinsic::find("llvm.trap").and_then(|i| i.get_declaration(self.module, &[])) {
                let _ = self.builder.build_call(trap, &[], "");
            }
            let _ = self.builder.build_unreachable();
            return;
        }
        let (msg, handler) = self.site_message(fi, site);
        let ptr = self.global_bytes(&msg);
        let len = self.ctx.i64_type().const_int(msg.len() as u64, false);
        if let Some(handler) = self.module.get_function(handler) {
            let args: Vec<BasicMetadataValueEnum> = vec![ptr.into(), len.into()];
            let _ = self.builder.build_call(handler, &args, "");
        }
        let _ = self.builder.build_unreachable();
    }

    /// A const array's elements in the program, once each, and its length
    /// and size in bytes: `incl.bytes`' file, or an array of literals.
    fn const_array(&mut self, c: &Const) -> Option<(PointerValue<'ctx>, (u64, u64))> {
        let key = format!("{:?}", c);
        if let Some(found) = self.const_arrays.get(&key) {
            return Some(*found);
        }
        let (init, n): (BasicValueEnum<'ctx>, u64) = match c {
            Const::Bytes(b) => (self.ctx.const_string(b, false).into(), b.len() as u64),
            Const::Array { elem, items } => {
                let et = self.ty(elem)?;
                let mut vals: Vec<BasicValueEnum<'ctx>> = Vec::new();
                for it in items {
                    let v = self.constant(0, it)?;
                    vals.push(self.fit(v, et));
                }
                let arr: BasicValueEnum<'ctx> = match et {
                    BasicTypeEnum::IntType(t) => t.const_array(&vals.iter().map(|v| v.into_int_value()).collect::<Vec<_>>()).into(),
                    BasicTypeEnum::FloatType(t) => t.const_array(&vals.iter().map(|v| v.into_float_value()).collect::<Vec<_>>()).into(),
                    _ => return None,
                };
                (arr, items.len() as u64)
            }
            _ => return None,
        };
        let size = self.target.get_store_size(&init.get_type());
        let g = self.module.add_global(init.get_type(), None, "const_array");
        g.set_initializer(&init);
        g.set_constant(true);
        g.set_linkage(Linkage::Private);
        let found = (g.as_pointer_value(), (n, size));
        self.const_arrays.insert(key, found);
        Some(found)
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
    /// In between, the limit is the widest operand's width, at least 64
    /// bits (owner, 2026-09-23, 2026-10-03): an exact answer below the
    /// signed type's lowest or above the unsigned one's highest stops
    /// here, unless Precog proved it never is.
    fn past_limit(&mut self, fi: usize, r: IntValue<'ctx>, limit: u32, sites: &[SiteId], func: FunctionValue<'ctx>) {
        let open = sites.iter().copied().find(|s| {
            let site = self.p.functions[fi].site(*s);
            matches!(site.kind, SiteKind::Overflow(_)) && self.gets_check(&site.verdict)
        });
        let Some(s) = open else { return };
        let t = r.get_type();
        let (lo, hi) = crate::bigint::type_range(true, crate::types::EXACT_FLAG | limit);
        let (Some(lo), Some(hi)) = (const_big(t, &lo), const_big(t, &hi)) else { return };
        let (Ok(low), Ok(high)) = (
            self.builder.build_int_compare(IntPredicate::SLT, r, lo, "below"),
            self.builder.build_int_compare(IntPredicate::SGT, r, hi, "above"),
        ) else { return };
        let Ok(out) = self.builder.build_or(low, high, "past_limit") else { return };
        let _ = self.guard(fi, s, out, func);
    }

    /// `div.X` or `mod.X` on whole numbers (`operators.md`): the hardware's
    /// truncated quotient and remainder, then one step where the rounding
    /// asks — up (`q + 1`, `r - y`) when the remainder and the divisor share
    /// a sign, down (`q - 1`, `r + y`) when not. Also whether it stepped,
    /// which an unsigned `mod.ceil` / `mod.near` needs: it is then below 0.
    /// A zero divisor and the lowest value by -1 are the caller's to stop
    /// first; a remainder never divides by -1 (`trunc_rem`).
    fn div_round(&mut self, x: IntValue<'ctx>, y: IntValue<'ctx>, signed: bool, want_mod: bool, round: crate::ast::Round) -> Option<(IntValue<'ctx>, IntValue<'ctx>)> {
        use crate::ast::Round;
        let t = x.get_type();
        let zero = t.const_zero();
        let one = t.const_int(1, false);
        let b = &self.builder;
        let (q, m) = if signed {
            // A remainder's quotient only for its parity: by -1 asked as
            // by 1 too, where the remainder is 0 and nothing steps.
            let q = if want_mod {
                let by_m1 = b.build_int_compare(IntPredicate::EQ, y, t.const_all_ones(), "by_m1").ok()?;
                let by = b.build_select(by_m1, one, y, "div_by").ok()?.into_int_value();
                b.build_int_signed_div(x, by, "q").ok()?
            } else {
                b.build_int_signed_div(x, y, "q").ok()?
            };
            (q, self.trunc_rem(x, y)?)
        } else {
            (b.build_int_unsigned_div(x, y, "q").ok()?, b.build_int_unsigned_rem(x, y, "r").ok()?)
        };
        let b = &self.builder;
        let no = self.ctx.bool_type().const_zero();
        if matches!(round, Round::Trunc) || (!signed && matches!(round, Round::Floor | Round::Bare)) {
            return Some((if want_mod { m } else { q }, no));
        }
        let nz = b.build_int_compare(IntPredicate::NE, m, zero, "inexact").ok()?;
        let up = if signed {
            let m_neg = b.build_int_compare(IntPredicate::SLT, m, zero, "rneg").ok()?;
            let y_neg = b.build_int_compare(IntPredicate::SLT, y, zero, "yneg").ok()?;
            b.build_not(b.build_xor(m_neg, y_neg, "differ").ok()?, "up").ok()?
        } else {
            self.ctx.bool_type().const_all_ones()
        };
        let step = match round {
            Round::Floor | Round::Bare => b.build_and(nz, b.build_not(up, "down").ok()?, "step").ok()?,
            Round::Ceil => b.build_and(nz, up, "step").ok()?,
            _ => {
                // The nearer whole number: |r| against |y| - |r|, unsigned,
                // a tie to the even quotient.
                let (am, ay) = if signed {
                    let neg = |v: IntValue<'ctx>, n: &str| -> Option<IntValue<'ctx>> {
                        let is_neg = b.build_int_compare(IntPredicate::SLT, v, zero, n).ok()?;
                        let flipped = b.build_int_sub(zero, v, n).ok()?;
                        Some(b.build_select(is_neg, flipped, v, n).ok()?.into_int_value())
                    };
                    (neg(m, "abs_r")?, neg(y, "abs_y")?)
                } else {
                    (m, y)
                };
                let rest = b.build_int_sub(ay, am, "rest").ok()?;
                let past = b.build_int_compare(IntPredicate::UGT, am, rest, "past_half").ok()?;
                let tie = b.build_int_compare(IntPredicate::EQ, am, rest, "tie").ok()?;
                let odd = b.build_int_truncate(q, self.ctx.bool_type(), "odd").ok()?;
                let tie_odd = b.build_and(tie, odd, "tie_odd").ok()?;
                b.build_and(nz, b.build_or(past, tie_odd, "nearer").ok()?, "step").ok()?
            }
        };
        let v = if want_mod {
            let lower = b.build_int_sub(m, y, "r_up").ok()?;
            let higher = b.build_int_add(m, y, "r_down").ok()?;
            let moved = b.build_select(up, lower, higher, "r_step").ok()?;
            b.build_select(step, moved, m.into(), "mod").ok()?.into_int_value()
        } else {
            let higher = b.build_int_add(q, one, "q_up").ok()?;
            let lower = b.build_int_sub(q, one, "q_down").ok()?;
            let moved = b.build_select(up, higher, lower, "q_step").ok()?;
            b.build_select(step, moved, q.into(), "div").ok()?.into_int_value()
        };
        Some((v, step))
    }

    /// `div.X` / `mod.X` on a bin or brain16, `X` not `near` (`operators.md`),
    /// as the oracle works them (0d25799): `fmod`, the quotient `(x - r) / y`
    /// rounded to a whole number, then one step for `floor` (`q - 1`, `r +
    /// y`, when the remainder's sign is not the divisor's) or `ceil` (`q +
    /// 1`, `r - y`, when it is): the true quotient, rounded as named. A
    /// remainder the step brings to the divisor's size is 0 (owner,
    /// 2026-10-02: b), so it stays below it; `mod.floor`'s 0 takes the
    /// divisor's sign, `mod.ceil`'s is `+0`, any other is `fmod`'s.
    /// A brain16 or f16 is worked at 32 bits, as its other arithmetic is,
    /// and its remainder asked again at its own width, where one more can
    /// round to the divisor.
    fn bin_div_round(&mut self, x: inkwell::values::FloatValue<'ctx>, y: inkwell::values::FloatValue<'ctx>, want_mod: bool, round: crate::ast::Round) -> Option<inkwell::values::FloatValue<'ctx>> {
        use crate::ast::Round;
        use inkwell::FloatPredicate as FP;
        let t = x.get_type();
        let wide = t == self.ctx.bf16_type() || t == self.ctx.f16_type();
        let wt = if wide { self.ctx.f32_type() } else { t };
        let (xw, yw) = if wide {
            (self.builder.build_float_ext(x, wt, "x32").ok()?, self.builder.build_float_ext(y, wt, "y32").ok()?)
        } else {
            (x, y)
        };
        let b = &self.builder;
        let zero = wt.const_zero();
        let m = b.build_float_rem(xw, yw, "frem").ok()?;
        // The step; a remainder it brings to the divisor's size is 0.
        let mut stepped = None;
        let m = if round == Round::Trunc {
            m
        } else {
            let nz = b.build_float_compare(FP::ONE, m, zero, "inexact").ok()?;
            let m_neg = b.build_float_compare(FP::OLT, m, zero, "rneg").ok()?;
            let y_neg = b.build_float_compare(FP::OLT, yw, zero, "yneg").ok()?;
            let differ = b.build_xor(m_neg, y_neg, "differ").ok()?;
            let (step, moved, edge) = if round == Round::Ceil {
                (b.build_and(nz, b.build_not(differ, "same").ok()?, "step").ok()?, b.build_float_sub(m, yw, "r_up").ok()?, b.build_float_neg(yw, "neg_y").ok()?)
            } else {
                (b.build_and(nz, differ, "step").ok()?, b.build_float_add(m, yw, "r_down").ok()?, yw)
            };
            let reached = b.build_float_compare(FP::OEQ, moved, edge, "reached_divisor").ok()?;
            let on_edge = b.build_and(step, reached, "on_edge").ok()?;
            let m = b.build_select(step, moved, m, "r_step").ok()?.into_float_value();
            let m = b.build_select(on_edge, zero, m, "below_divisor").ok()?.into_float_value();
            stepped = Some(step);
            m
        };
        if want_mod {
            let m = if wide { self.float_cast(m, t)? } else { m };
            if round == Round::Trunc {
                return Some(m);
            }
            // At the type's own width, and `mod.floor`'s 0 by the divisor.
            return self.remainder_rule_after(m, y, round);
        }
        let b = &self.builder;
        let fm = b.build_float_rem(xw, yw, "fmod").ok()?;
        let less = b.build_float_sub(xw, fm, "whole_part").ok()?;
        let q = b.build_float_div(less, yw, "q").ok()?;
        let even = inkwell::intrinsics::Intrinsic::find("llvm.roundeven")?.get_declaration(self.module, &[wt.into()])?;
        let q = basic_result(self.builder.build_call(even, &[q.into()], "q_whole").ok()?)?.into_float_value();
        let q = match stepped {
            Some(kept) => {
                let b = &self.builder;
                let one = wt.const_float(1.0);
                let moved = if round == Round::Ceil { b.build_float_add(q, one, "q_up").ok()? } else { b.build_float_sub(q, one, "q_down").ok()? };
                b.build_select(kept, moved, q, "q_step").ok()?.into_float_value()
            }
            None => q,
        };
        // A zero's sign is `rvalue`'s (owner, 2026-10-07).
        if wide {
            self.float_cast(q, t)
        } else {
            Some(q)
        }
    }

    /// A remainder at the divisor's size is 0: `+0` for `mod.ceil`, the
    /// divisor's sign for `mod.floor`, whose every 0 takes it.
    fn remainder_rule_after(&mut self, m: inkwell::values::FloatValue<'ctx>, y: inkwell::values::FloatValue<'ctx>, round: crate::ast::Round) -> Option<inkwell::values::FloatValue<'ctx>> {
        use crate::ast::Round;
        use inkwell::FloatPredicate as FP;
        let t = m.get_type();
        let tz = t.const_zero();
        let b = &self.builder;
        let reached = if round == Round::Ceil {
            let neg_y = b.build_float_neg(y, "neg_y").ok()?;
            b.build_float_compare(FP::OEQ, m, neg_y, "reached_divisor").ok()?
        } else {
            b.build_float_compare(FP::OEQ, m, y, "reached_divisor").ok()?
        };
        let m = b.build_select(reached, tz, m, "below_divisor").ok()?.into_float_value();
        if round == Round::Ceil {
            return Some(m);
        }
        let cs = inkwell::intrinsics::Intrinsic::find("llvm.copysign")?.get_declaration(self.module, &[t.into()])?;
        let signed_zero = basic_result(self.builder.build_call(cs, &[tz.into(), y.into()], "zero").ok()?)?.into_float_value();
        let is_zero = self.builder.build_float_compare(FP::OEQ, m, tz, "is_zero").ok()?;
        Some(self.builder.build_select(is_zero, signed_zero, m, "mod").ok()?.into_float_value())
    }

    /// The hardware's signed remainder, the dividend's sign (`mod.trunc`).
    /// The lowest value by -1 is undefined to LLVM, and its remainder is 0
    /// with no check site: any number by -1 leaves 0, so -1 is asked as 1.
    fn trunc_rem(&mut self, x: IntValue<'ctx>, y: IntValue<'ctx>) -> Option<IntValue<'ctx>> {
        let by_m1 = self.builder.build_int_compare(IntPredicate::EQ, y, y.get_type().const_all_ones(), "by_m1").ok()?;
        let y = self.builder.build_select(by_m1, y.get_type().const_int(1, false), y, "rem_by").ok()?.into_int_value();
        self.builder.build_int_signed_rem(x, y, "r").ok()
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
                BinOp::Div(crate::ast::Round::Near) => self.builder.build_float_div(x, y, "fdiv").ok()?.into(),
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
            // No lane holds an infinity or a NaN (owner, 2026-09-29): a
            // zero divisor in any lane, then any lane's answer not a real
            // number, stops it.
            if matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div(_) | BinOp::DivIeee(_)) {
                if let (BinOp::Div(_) | BinOp::DivIeee(_), Some(s)) = (op, open(self, &|k| matches!(k, SiteKind::DivZero))) {
                    let m = self.builder.build_float_compare(FP::OEQ, y, y.get_type().const_zero(), "by_zero").ok()?;
                    let bad = any(self, m)?;
                    self.guard(fi, s, bad, func)?;
                }
                if let Some(s) = open(self, &|k| matches!(k, SiteKind::Overflow(_))) {
                    self.bin_finite_guard(fi, s, r.into_vector_value(), func)?;
                }
            }
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
            BinOp::Div(round) | BinOp::DivIeee(round) | BinOp::Mod(round) => {
                if let Some(s) = open(self, &|k| matches!(k, SiteKind::DivZero | SiteKind::ModZero)) {
                    let z = self.builder.build_int_compare(IntPredicate::EQ, y, zero, "by_zero").ok()?;
                    let bad = any(self, z)?;
                    self.guard(fi, s, bad, func)?;
                }
                if signed && op.is_div() {
                    if let Some(s) = open(self, &|k| matches!(k, SiteKind::Overflow(BinOp::Div(_) | BinOp::DivIeee(_)))) {
                        let min = const_i128(vt.get_element_type().into_int_type(), -(1i128 << (bits - 1)));
                        let mins = splat(self, min.into())?;
                        let m1 = splat(self, vt.get_element_type().into_int_type().const_all_ones().into())?;
                        let is_min = self.builder.build_int_compare(IntPredicate::EQ, x, mins, "is_min").ok()?;
                        let by_m1 = self.builder.build_int_compare(IntPredicate::EQ, y, m1, "by_m1").ok()?;
                        let both = self.builder.build_and(is_min, by_m1, "min_by_m1").ok()?;
                        let bad = any(self, both)?;
                        self.guard(fi, s, bad, func)?;
                    }
                }
                // Lane by lane, the scalar rule: no machine divides a
                // vector of integers in one instruction anyway.
                let mut out = vt.get_undef();
                let mut stepped = self.ctx.bool_type().const_zero();
                for k in 0..vt.get_size() {
                    let at = self.ctx.i32_type().const_int(k as u64, false);
                    let xi = self.builder.build_extract_element(x, at, "x_lane").ok()?.into_int_value();
                    let yi = self.builder.build_extract_element(y, at, "y_lane").ok()?.into_int_value();
                    let (v, step) = self.div_round(xi, yi, signed, op.is_mod(), *round)?;
                    stepped = self.builder.build_or(stepped, step, "stepped").ok()?;
                    out = self.builder.build_insert_element(out, v, at, "lane").ok()?;
                }
                // Unsigned, what a step up leaves is below 0.
                if !signed && op.is_mod() {
                    if let Some(s) = open(self, &|k| matches!(k, SiteKind::Overflow(BinOp::Mod(_)))) {
                        self.guard(fi, s, stepped, func)?;
                    }
                }
                out.into()
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
                let tmp = self.stack(shape, "copy").ok()?;
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
    fn inline_asm(&mut self, fi: usize, text: &str, ins: &[(Place, String)], outs: &[(Place, String)], lends: &[(Place, bool, String)], clobbers: &[String], slots: &[Option<PointerValue<'ctx>>]) -> Option<()> {
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
        // A lend is an address, as at an extern call: the data of a text,
        // list or other handle, or the variable itself.
        for (p, _, r) in lends {
            constraints.push(format!("{{{}}}", abi(r)));
            let (slot, ty) = self.slot_of(fi, p, slots)?;
            let addr: PointerValue = if is_handle(&ty) {
                let st = match self.ty(&ty)? {
                    BasicTypeEnum::StructType(s) => s,
                    _ => return None,
                };
                let field = self.builder.build_struct_gep(st, slot, 0, "data").ok()?;
                self.builder.build_load(self.ctx.ptr_type(AddressSpace::default()), field, "p").ok()?.into_pointer_value()
            } else {
                slot
            };
            let t = self.ctx.i64_type();
            args.push(self.builder.build_ptr_to_int(addr, t, "addr").ok()?.into());
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
    /// A stack slot for the function being generated, always made in its
    /// entry block. Made where it is first needed, a slot is a *dynamic*
    /// allocation: inside a loop it grows the stack every trip, and LLVM
    /// will neither inline such a function nor promote its slots to
    /// registers (`bench/sieve.cpls`: `read_n` stayed a call, and the range
    /// it checks never reached the loop that needed it).
    pub(super) fn stack<T: inkwell::types::BasicType<'ctx>>(&self, t: T, name: &str) -> Result<PointerValue<'ctx>, inkwell::builder::BuilderError> {
        let Some(func) = self.builder.get_insert_block().and_then(|b| b.get_parent()) else { return self.builder.build_alloca(t, name) };
        let Some(entry) = func.get_first_basic_block() else { return self.builder.build_alloca(t, name) };
        let b = self.ctx.create_builder();
        match entry.get_first_instruction() {
            Some(i) => b.position_before(&i),
            None => b.position_at_end(entry),
        }
        b.build_alloca(t, name)
    }

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
    /// A call through a held function in `no_cheat` code (owner,
    /// 2026-10-19: "(a)") calls the `no_cheat` copy of the function held:
    /// which one is known only as it runs, so the code is compared with
    /// every function that has a copy. LLVM folds the comparisons away
    /// where it knows which function is held.
    fn nocheat_held(&mut self, code: PointerValue<'ctx>) -> Option<PointerValue<'ctx>> {
        let fi = self.cur_fi;
        if !self.p.functions[fi].nocheat && !self.nocheat.contains(&(fi, self.here.0 as usize)) {
            return Some(code);
        }
        let pairs: Vec<(usize, usize)> = self.p.functions.iter().enumerate().filter_map(|(c, g)| g.nocheat_of.map(|o| (o.0 as usize, c))).collect();
        let mut code = code;
        for (o, c) in pairs {
            let (Some(of), Some(cf)) = (self.fns.get(&o).copied(), self.fns.get(&c).copied()) else { continue };
            let of = of.as_global_value().as_pointer_value();
            let cf = cf.as_global_value().as_pointer_value();
            let is = self.builder.build_int_compare(IntPredicate::EQ, code, of, "held").ok()?;
            code = self.builder.build_select(is, cf, code, "held").ok()?.into_pointer_value();
        }
        Some(code)
    }

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
                let tmp = self.stack(v.get_type(), "lent_value").ok()?;
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
        let code = self.nocheat_held(code)?;
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

    /// `simd.sum` is a plain number (owner, 2026-10-07: "b"): the lanes are
    /// added at the width of the exact answer it lands as, where N of them
    /// cannot overflow, and checked only where it lands (`.lane` is that,
    /// narrowed to the lane's type).
    fn simd_sum(&mut self, v: inkwell::values::VectorValue<'ctx>, signed: bool, wide: Option<u32>) -> Option<BasicValueEnum<'ctx>> {
        let et = v.get_type().get_element_type().into_int_type();
        let bits = et.get_bit_width();
        let wide_bits = wide.unwrap_or(bits * 2).max(bits);
        let wide_t = self.ctx.custom_width_int_type(std::num::NonZeroU32::new(wide_bits)?).ok()?;
        let wide_v = wide_t.vec_type(v.get_type().get_size());
        let wide = if signed { self.builder.build_int_s_extend(v, wide_v, "wide").ok()? } else { self.builder.build_int_z_extend(v, wide_v, "wide").ok()? };
        let id = inkwell::intrinsics::Intrinsic::find("llvm.vector.reduce.add")?;
        let f = id.get_declaration(self.module, &[wide_v.into()])?;
        let total = basic_result(self.builder.build_call(f, &[wide.into()], "sum").ok()?)?.into_int_value();
        Some(total.into())
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
            // Through `float_cast`, so a value narrowed to nothing lands as
            // 0, never -0 (examples/zero_everywhere: `'w': f32 = 'e30'
            // times 'e30' times (-1.0)` printed -0).
            (BasicValueEnum::FloatValue(x), BasicTypeEnum::FloatType(t)) if x.get_type() != t => self.float_cast(x, t).map(|y| y.into()).unwrap_or(v),
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
        // `list[i64]` read as a `list[i32]` is garbage. What that should
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
                    self.cannot(format!("handing {} to a parameter that takes {} has no shape in the generated code yet: their elements differ in width", crate::types::a_type(&spell(&have)), crate::types::a_type(&spell(w))), None);
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
                        let tmp = self.stack(v.get_type(), "lent_value").ok()?;
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
        // C's extras get C's default promotions: a whole number narrower
        // than an `int` widens to one, a `float` to a `double`.
        if f.get_type().is_var_arg() {
            let i32t = self.ctx.i32_type();
            for i in want.len()..vals.len() {
                let Ok(v) = BasicValueEnum::try_from(vals[i]) else { continue };
                vals[i] = match v {
                    BasicValueEnum::IntValue(x) if x.get_type().get_bit_width() < 32 => {
                        let signed = x.get_type().get_bit_width() > 1 && signs.get(i).copied().unwrap_or(true);
                        self.builder.build_int_cast_sign_flag(x, i32t, signed, "promoted").ok()?.into()
                    }
                    BasicValueEnum::FloatValue(x) if x.get_type() != self.ctx.f64_type() => self.builder.build_float_ext(x, self.ctx.f64_type(), "promoted").ok()?.into(),
                    _ => vals[i],
                };
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
            // A const array read in place (owner, 2026-10-10): the
            // program's own copy, nothing copied, never freed.
            Op::ConstView => {
                let Some(Operand::Const(c)) = args.first() else { return None };
                let (g, (n, _)) = self.const_array(c)?;
                let h = self.handle(2);
                let i64t = self.ctx.i64_type();
                Some(Some(h.const_named_struct(&[g.into(), i64t.const_int(n, false).into()]).into()))
            }
            // `incl.bytes[…]` (owner, 2026-10-10): the file's bytes in the
            // program, copied once into the new array, which may change.
            Op::InclBytes => {
                let Some(Operand::Const(Const::Str(hex))) = args.first() else { return None };
                let bytes: Vec<u8> = (0..hex.len() / 2).filter_map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok()).collect();
                let i64t = self.ctx.i64_type();
                let n = i64t.const_int(bytes.len() as u64, false);
                let data = self.ctx.const_string(&bytes, false);
                let g = self.module.add_global(data.get_type(), None, "incl");
                g.set_initializer(&data);
                g.set_constant(true);
                g.set_linkage(Linkage::Private);
                let mem = self.alloc(n);
                if !bytes.is_empty() {
                    let _ = self.builder.build_memcpy(mem, 1, g.as_pointer_value(), 1, n);
                }
                let h = self.handle(2);
                let v = h.get_undef();
                let v = self.builder.build_insert_value(v, mem, 0, "data").ok()?;
                let v = self.builder.build_insert_value(v, n, 1, "len").ok()?;
                Some(Some(v.into_struct_value().into()))
            }
            // Of cord, the prints and `exit` (owner, 2026-10-09).
            Op::Std(name) if name == "exit" => {
                self.emit_exit(fi, args, slots)?;
                Some(None)
            }
            Op::Std(name) if crate::check::expr::BUILT_CORD.contains(&name.as_str()) => {
                self.emit_print(fi, name, args, slots)?;
                Some(None)
            }
            // `list.append` hands nothing back (`lists.md`), so it is
            // handled and has no value — which is not the same as not
            // being handled at all.
            Op::ListAppend => {
                self.list_append(fi, args, slots, func)?;
                Some(None)
            }
            Op::ListFill => Some(Some(self.list_fill(fi, args, &dest_ty?, slots, func)?)),
            // A literal with no 0 byte: `C.str.make` that cannot fail.
            Op::CStrMake => {
                let out = dest_ty.unwrap_or_else(|| Ty::list(Ty::int(false, 8)));
                Some(Some(self.cstr_make(fi, &Arg::Value(args.first()?.clone()), &out, slots)?.0))
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
            Op::SimdAll | Op::SimdAny | Op::SimdMax | Op::SimdMin | Op::SimdSum | Op::SimdSumWrap => {
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
                    let wide = dest_ty.as_ref().and_then(|t| self.ty(t)).filter(|t| t.is_int_type()).map(|t| t.into_int_type().get_bit_width());
                    return Some(Some(self.simd_sum(v, signed, wide)?));
                }
                let id = inkwell::intrinsics::Intrinsic::find(name)?;
                let f = id.get_declaration(self.module, &[v.get_type().into()])?;
                let call = if name == "llvm.vector.reduce.fadd" {
                    let zero = et.into_float_type().const_float(0.0);
                    self.builder.build_call(f, &[zero.into(), v.into()], "reduce").ok()?
                } else {
                    self.builder.build_call(f, &[v.into()], "reduce").ok()?
                };
                // `simd.max`/`min`/`sum` give `0`, never `-0` (design
                // claude's reading, 2026-10-07).
                let r = basic_result(call)?;
                Some(Some(if float { self.no_negative_zero(r)? } else { r }))
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
            // `u8`, a point's a `u32`, and the char already is it.
            Op::CharCode => {
                let v = self.operand(fi, args.first()?, slots)?.into_int_value();
                let want = match dest_ty.as_ref().and_then(|t| self.ty(t)) {
                    Some(BasicTypeEnum::IntType(t)) => t,
                    _ => v.get_type(),
                };
                Some(Some(self.builder.build_int_z_extend_or_bit_cast(v, want, "code").ok()?.into()))
            }
            // `on.machine.chip X` (owner, 2026-10-06): the runtime asks the
            // chip, as the oracle does.
            Op::MachineHas(f) => {
                let has = self.rt("cyborg_machine_has");
                let code = self.ctx.i32_type().const_int(*f as u64, false);
                let v = basic_result(self.builder.build_call(has, &[code.into()], "has").ok()?)?;
                Some(Some(v))
            }
            // `must` in a `test fn` (owner, 2026-09-29).
            Op::MustReport => {
                let v = self.operand(fi, args.first()?, slots)?;
                let (ptr, len) = self.text_parts_pub(v)?;
                let f = &self.p.functions[fi];
                let (b, si) = (self.here.0 as usize, self.here.1 as usize);
                let block = &f.blocks[b];
                let span = f.span(block.stmts.get(si).map(|s| s.span).unwrap_or(block.term_span));
                let at = format!("{}: ", self.sources.get(span.file).line_of(span.start));
                let i64t = self.ctx.i64_type();
                let p = self.ctx.ptr_type(AddressSpace::default());
                let rf = self.module.get_function("cyborg_must_report").unwrap_or_else(|| self.module.add_function("cyborg_must_report", self.ctx.void_type().fn_type(&[p.into(), i64t.into(), p.into(), i64t.into()], false), None));
                let at_len = at.len() as u64;
                let at_ptr = self.builder.build_global_string_ptr(&at, "must_at").ok()?.as_pointer_value();
                let _ = self.builder.build_call(rf, &[ptr.into(), len.into(), at_ptr.into(), i64t.const_int(at_len, false).into()], "");
                Some(None)
            }
            Op::MustStopFork => {
                let rf = self.module.get_function("cyborg_must_stop_fork").unwrap_or_else(|| self.module.add_function("cyborg_must_stop_fork", self.ctx.i32_type().fn_type(&[], false), None));
                let v = self.builder.build_call(rf, &[], "stop").ok().and_then(basic_result)?;
                Some(Some(v))
            }
            Op::MustStopDone => {
                let rf = self.module.get_function("cyborg_must_stop_done").unwrap_or_else(|| self.module.add_function("cyborg_must_stop_done", self.ctx.void_type().fn_type(&[], false), None));
                let _ = self.builder.build_call(rf, &[], "");
                Some(None)
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

    /// One float type to another. `bfloat` and `half` are both 16 bits, so
    /// LLVM casts neither way between them: through `float`, which holds
    /// every value of both exactly, so the one rounding is the last.
    pub(super) fn float_cast(&mut self, x: inkwell::values::FloatValue<'ctx>, to: inkwell::types::FloatType<'ctx>) -> Option<inkwell::values::FloatValue<'ctx>> {
        let (bf, h) = (self.ctx.bf16_type(), self.ctx.f16_type());
        let from = x.get_type();
        if from == to {
            return Some(x);
        }
        let r = if (from == bf && to == h) || (from == h && to == bf) {
            let mid = self.builder.build_float_ext(x, self.ctx.f32_type(), "via_float").ok()?;
            self.builder.build_float_trunc(mid, to, "conv").ok()?
        } else {
            self.builder.build_float_cast(x, to, "conv").ok()?
        };
        // Rounding where a value lands never makes a -0 of a number that was
        // not zero, and keeps one an `.ieee` form made (design claude's
        // reading, 2026-10-07). Only a narrower type can round to zero.
        let width = |t: inkwell::types::FloatType<'ctx>| if t == bf || t == h { 16 } else if t == self.ctx.f32_type() { 32 } else { 64 };
        if width(to) == 64 || (width(to) == 32 && width(from) == 16) {
            return Some(r);
        }
        let b = &self.builder;
        let was_zero = b.build_float_compare(inkwell::FloatPredicate::OEQ, x, from.const_zero(), "was_zero").ok()?;
        let unsigned = b.build_float_add(r, to.const_zero(), "no_negative_zero").ok()?;
        Some(b.build_select(was_zero, r, unsigned, "landed").ok()?.into_float_value())
    }

    /// A value LLVM must take as unknown: an empty `asm volatile` hands the
    /// local's value back through a register, so a loop's rounds can be
    /// neither folded to their answer nor dropped (`for !cheat`).
    pub(super) fn no_cheat_barrier(&mut self, fi: usize, l: LocalId, slots: &[Option<PointerValue<'ctx>>]) {
        let local = &self.p.functions[fi].locals[l.0 as usize];
        if matches!(local.kind, LocalKind::Lent { .. }) {
            return;
        }
        let Some(slot) = slots.get(l.0 as usize).copied().flatten() else { return };
        let ty = local.ty.clone();
        let Some(t) = self.ty(&ty) else { return };
        self.barrier_on(slot, t);
    }

    /// The barrier on whatever a slot holds: a local, or a slice's partial.
    pub(super) fn barrier_on(&mut self, slot: PointerValue<'ctx>, t: BasicTypeEnum<'ctx>) {
        let x86 = host_triple().as_str().to_string_lossy().starts_with("x86_64");
        let constraint = match t {
            BasicTypeEnum::IntType(i) if (8..=64).contains(&i.get_bit_width()) => "=r,0",
            BasicTypeEnum::FloatType(_) => if x86 { "=x,0" } else { "=w,0" },
            _ => return,
        };
        let fnty = t.fn_type(&[t.into()], false);
        let asm = self.ctx.create_inline_asm(fnty, String::new(), constraint.to_string(), true, false, None, false);
        let Ok(v) = self.builder.build_load(t, slot, "nocheat") else { return };
        let Ok(call) = self.builder.build_indirect_call(fnty, asm, &[v.into()], "nocheat") else { return };
        if let Some(r) = basic_result(call) {
            let _ = self.builder.build_store(slot, r);
        }
    }

    /// `llvm.loop` metadata on a loop's back edge saying not to vectorise
    /// it: a node naming itself first, as LLVM's loop IDs do.
    fn no_vectorize(&self, br: inkwell::values::InstructionValue<'ctx>) {
        use inkwell::context::AsContextRef;
        use inkwell::values::AsValueRef;
        use llvm_sys::core::*;
        unsafe {
            let ctx = self.ctx.as_ctx_ref();
            let key = "llvm.loop.vectorize.enable";
            let name = LLVMMDStringInContext2(ctx, key.as_ptr() as *const _, key.len());
            let off = LLVMValueAsMetadata(LLVMConstInt(LLVMInt1TypeInContext(ctx), 0, 0));
            let mut opt_ops = [name, off];
            let opt = LLVMMDNodeInContext2(ctx, opt_ops.as_mut_ptr(), 2);
            let tmp = llvm_sys::debuginfo::LLVMTemporaryMDNode(ctx, std::ptr::null_mut(), 0);
            let mut id_ops = [tmp, opt];
            let id = LLVMMDNodeInContext2(ctx, id_ops.as_mut_ptr(), 2);
            llvm_sys::debuginfo::LLVMMetadataReplaceAllUsesWith(tmp, id);
            let kind_name = "llvm.loop";
            let kind = LLVMGetMDKindIDInContext(ctx, kind_name.as_ptr() as *const _, kind_name.len() as u32);
            LLVMSetMetadata(br.as_value_ref(), kind, LLVMMetadataAsValue(ctx, id));
        }
    }

    /// Trap at `site` unless `r` is a real number: an infinity or a NaN is
    /// no bin (owner, 2026-09-29). `r - r` is 0 only for a finite `r`, lane
    /// by lane for a vector.
    fn bin_finite_guard<V: inkwell::values::FloatMathValue<'ctx> + Copy>(&mut self, fi: usize, site: SiteId, r: V, func: FunctionValue<'ctx>) -> Option<()> {
        let z = self.builder.build_float_sub(r, r, "rz").ok()?;
        let zero = z.as_basic_value_enum().get_type();
        let cmp: BasicValueEnum = match z.as_basic_value_enum() {
            BasicValueEnum::FloatValue(v) => self.builder.build_float_compare(inkwell::FloatPredicate::UNE, v, v.get_type().const_zero(), "not_finite").ok()?.into(),
            BasicValueEnum::VectorValue(v) => {
                let _ = zero;
                self.builder.build_float_compare(inkwell::FloatPredicate::UNE, v, v.get_type().const_zero(), "not_finite").ok()?.into()
            }
            _ => return None,
        };
        let bad = match cmp {
            BasicValueEnum::IntValue(b) => b,
            BasicValueEnum::VectorValue(m) => {
                let id = inkwell::intrinsics::Intrinsic::find("llvm.vector.reduce.or")?;
                let f = id.get_declaration(self.module, &[m.get_type().into()])?;
                basic_result(self.builder.build_call(f, &[m.into()], "any_lane").ok()?)?.into_int_value()
            }
            _ => return None,
        };
        self.guard(fi, site, bad, func)
    }

    /// The range a call's answer is known to be in, when its type is an
    /// integer range and nothing in the program could have put another
    /// number there (owner, 2026-09-29: c): no range narrowing taken on
    /// `nocheck`'s word, no C door trusted under `NO_FUCKING_CHECKS`, no C
    /// global of a range type (C writes those unasked). Checked or proven,
    /// never a claim.
    fn guaranteed_range(&mut self, t: &Ty) -> Option<(i128, i128)> {
        let Ty::Bounded { bound: crate::types::Bound::Int { lo, hi, .. }, .. } = t else { return None };
        let (lo, hi) = (*lo, *hi);
        if self.ranges_trusted.is_none() {
            let items = &self.p.items;
            let trusted = items.trusted_door
                || items.externs.iter().any(|e| e.is_let && matches!(e.ret, Ty::Bounded { .. }))
                || self.p.functions.iter().any(|f| {
                    f.sites.iter().any(|s| match (&s.kind, &s.verdict) {
                        (SiteKind::Narrow { to } | SiteKind::LengthBound { to }, Verdict::Trusted) => to.bound().is_some(),
                        _ => s.tier == Some(crate::ast::Tier::NoChecks),
                    })
                });
            self.ranges_trusted = Some(trusted);
        }
        if self.ranges_trusted == Some(true) {
            return None;
        }
        Some((lo, hi))
    }

    /// A call's answer, told its range: `min(max(v, lo), hi)`, which changes
    /// nothing on a value already inside, and is a range LLVM's loop
    /// analysis reads (`bench/sieve.cpls`: `n` came back from `read_n`
    /// through the failure outcome, and without it LLVM worked the inner
    /// loop's trip count out in 128 bits on every outer trip).
    pub(super) fn clamp_answer(&mut self, v: BasicValueEnum<'ctx>, ret: &Ty) -> BasicValueEnum<'ctx> {
        let BasicValueEnum::IntValue(x) = v else { return v };
        let Some((lo, hi)) = self.guaranteed_range(ret) else { return v };
        let signed = int_signed(ret).unwrap_or(true);
        let t = x.get_type();
        let (max, min) = if signed { ("llvm.smax", "llvm.smin") } else { ("llvm.umax", "llvm.umin") };
        let mut out = x;
        for (name, bound) in [(max, lo), (min, hi)] {
            let Some(id) = inkwell::intrinsics::Intrinsic::find(name) else { return v };
            let Some(f) = id.get_declaration(self.module, &[t.into()]) else { return v };
            let c = const_i128(t, bound);
            match self.builder.build_call(f, &[out.into(), c.into()], "in_range").ok().and_then(basic_result) {
                Some(r) => out = r.into_int_value(),
                None => return v,
            }
        }
        out.into()
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
        if let (Ty::Deci(bits), Some(crate::types::Bound::Bin { lo, hi, .. })) = (to.base(), to.bound()) {
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
        // A whole number into a bin or brain16: its size below where
        // rounding goes past the largest, compared as whole numbers.
        let bin_bits = match to.base() {
            Ty::Bin(crate::types::BinW::Fixed(b)) => Some(*b),
            Ty::Brain => Some(crate::types::BRAIN_BITS),
            _ => None,
        };
        if let (Some(b), Some(o)) = (bin_bits, site.operands.first()) {
            let o = match o.place() {
                Some(p) => Operand::Read(p.clone()),
                None => o.clone(),
            };
            if let Some(BasicValueEnum::IntValue(v)) = self.operand(fi, &o, slots) {
                let signed = self.operand_signed(fi, &o).unwrap_or(true);
                let limit = crate::bigint::bin_round_limit(b);
                let need = (v.get_type().get_bit_width() + 1).max(limit.bits() as u32 + 2);
                let Some(w) = std::num::NonZeroU32::new(need).and_then(|n| self.ctx.custom_width_int_type(n).ok()) else { return };
                let x = self.fit_as(v.into(), w.into(), signed).into_int_value();
                let (Some(hi), Some(lo)) = (const_big(w, &limit), const_big(w, &limit.neg())) else { return };
                let (Ok(above), Ok(below)) = (
                    self.builder.build_int_compare(IntPredicate::SGE, x, hi, "rounds_past"),
                    self.builder.build_int_compare(IntPredicate::SLE, x, lo, "rounds_past_low"),
                ) else { return };
                let Ok(bad) = self.builder.build_or(above, below, "past_largest") else { return };
                let _ = self.guard(fi, id, bad, func);
                return;
            }
        }
        if let Ty::Bin(crate::types::BinW::Fixed(bits @ (16 | 32))) = to.base() {
            return self.bin_narrow_check(fi, id, *bits, &site, slots, func);
        }
        if to.is_brain() {
            return self.bin_narrow_check(fi, id, crate::types::BRAIN_BITS, &site, slots, func);
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
        // The target's ends, at any size (owner, 2026-10-03).
        let ends = match (crate::precog::smt::int_bounds(to), to.base()) {
            (Some((lo, hi)), _) => (crate::bigint::Big::from_i128(lo), crate::bigint::Big::from_i128(hi)),
            (None, Ty::Int(IntW::Fixed { signed, bits })) if to.bound().is_none() => crate::bigint::type_range(*signed, *bits),
            _ => {
                if !matches!(to.base(), Ty::Bin(_)) {
                    self.cannot(format!("a narrowing into `{}` has no run-time check yet", crate::types::TySpelling { ty: to, structs: &self.p.items.structs, enums: &self.p.items.enums }), Some(self.p.functions[fi].span(site.span)));
                }
                return;
            }
        };
        let Some(o) = site.operands.first() else { return };
        let o = match o.place() {
            Some(p) => Operand::Read(p.clone()),
            None => o.clone(),
        };
        let signed = self.operand_signed(fi, &o).unwrap_or(true);
        let Some(BasicValueEnum::IntValue(v)) = self.operand(fi, &o, slots) else { return };
        // Compared at a width that holds the value, by its own sign, and
        // the target's ends, so no comparison can itself wrap.
        let need = (v.get_type().get_bit_width() + 1).max(ends.0.bits().max(ends.1.bits()) as u32 + 2).max(128);
        let Some(w) = std::num::NonZeroU32::new(need).and_then(|n| self.ctx.custom_width_int_type(n).ok()) else { return };
        let x = self.fit_as(v.into(), w.into(), signed).into_int_value();
        let (Some(lo), Some(hi)) = (const_big(w, &ends.0), const_big(w, &ends.1)) else { return };
        let (Ok(below), Ok(above)) = (
            self.builder.build_int_compare(IntPredicate::SLT, x, lo, "below"),
            self.builder.build_int_compare(IntPredicate::SGT, x, hi, "above"),
        ) else { return };
        let Ok(bad) = self.builder.build_or(below, above, "outside") else { return };
        let _ = self.guard(fi, id, bad, func);
    }

    /// A value going into an f16 or f32 that Precog could not prove fits:
    /// it fits when rounding it to the width gives a finite value — a finite
    /// value that rounds to infinity went past the largest one (65520 into a
    /// f16). Until now a bin narrowing generated no check at all.
    fn bin_narrow_check(&mut self, fi: usize, id: SiteId, bits: u16, site: &crate::mir::Site, slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) {
        let Some(o) = site.operands.first() else { return };
        let o = match o.place() {
            Some(p) => Operand::Read(p.clone()),
            None => o.clone(),
        };
        let target = match bits {
            16 => self.ctx.f16_type(),
            crate::types::BRAIN_BITS => self.ctx.bf16_type(),
            _ => self.ctx.f32_type(),
        };
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
        let Some(r) = self.float_cast(x, target) else { return };
        let Some(r) = self.float_cast(r, x.get_type()) else { return };
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
        // In 128 bits, each extended by its own sign, so a `u64`
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
                // Past 128 bits, by hand: the product at twice the width,
                // and whether the low half read back by its sign is all of
                // it. LLVM's overflow intrinsics need a library call there.
                Some(s) if t.get_bit_width() > 128 => {
                    let wide = this.ctx.custom_width_int_type(std::num::NonZeroU32::new(2 * t.get_bit_width())?).ok()?;
                    let a2 = this.builder.build_int_cast_sign_flag(a, wide, signed, "a2").ok()?;
                    let b2 = this.builder.build_int_cast_sign_flag(b, wide, signed, "b2").ok()?;
                    let p = this.builder.build_int_mul(a2, b2, "p2").ok()?;
                    let low = this.builder.build_int_truncate(p, t, "low").ok()?;
                    let back = this.builder.build_int_cast_sign_flag(low, wide, signed, "back").ok()?;
                    let o = this.builder.build_int_compare(IntPredicate::NE, back, p, "past").ok()?;
                    this.guard(fi, s, o, func)?;
                    Some(low)
                }
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

    /// A text position is 1 or more: below 1 (0 for an unsigned one) stops
    /// the program.
    fn position_check(&mut self, fi: usize, id: SiteId, slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) {
        let site = self.p.functions[fi].site(id).clone();
        if !self.gets_check(&site.verdict) {
            return;
        }
        let Some(o) = site.operands.first() else { return };
        let signed = self.operand_signed(fi, o) != Some(false);
        let read = match o.place() {
            Some(p) => Operand::Read(p.clone()),
            None => o.clone(),
        };
        let Some(BasicValueEnum::IntValue(e)) = self.operand(fi, &read, slots) else { return };
        let pred = if signed { IntPredicate::SLT } else { IntPredicate::ULT };
        let Ok(below) = self.builder.build_int_compare(pred, e, e.get_type().const_int(1, false), "no_position") else { return };
        let _ = self.guard(fi, id, below, func);
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
        if self.is_bit(fi, p) {
            let _ = self.bit_write(fi, p, v, slots);
            return;
        }
        if let Some((outer, _, f, _)) = self.c_field(fi, p).filter(|c| c.2.width.is_some()) {
            if let (Some((base, _)), BasicValueEnum::IntValue(x)) = (self.slot_of(fi, &outer, slots), v) {
                if let Some(at) = self.c_field_at(base, f) {
                    let _ = self.c_bits_write(at, f, x);
                }
            }
            return;
        }
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
/// A whole number at any size as a constant of `t`.
fn const_big<'ctx>(t: inkwell::types::IntType<'ctx>, v: &crate::bigint::Big) -> Option<IntValue<'ctx>> {
    if let Some(small) = v.to_i128() {
        return Some(const_i128(t, small));
    }
    let digits = v.to_string();
    let m = t.const_int_from_string(digits.trim_start_matches('-'), inkwell::types::StringRadix::Decimal)?;
    Some(if v.is_neg() { m.const_neg() } else { m })
}

fn const_i128<'ctx>(t: inkwell::types::IntType<'ctx>, v: i128) -> IntValue<'ctx> {
    let bits = v as u128;
    // Wider than 128 bits, a negative carries its sign up every word.
    let words = (t.get_bit_width() as usize).div_ceil(64).max(2);
    let fill = if v < 0 { u64::MAX } else { 0 };
    let mut w = vec![fill; words];
    w[0] = bits as u64;
    w[1] = (bits >> 64) as u64;
    t.const_int_arbitrary_precision(&w)
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
/// The macOS version a program is built for: the deployment target when
/// one is set, this Mac's otherwise. The object and the link both say it,
/// or the linker warns that one was built for a newer macOS.
fn macos_version() -> Option<String> {
    static VERSION: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    VERSION
        .get_or_init(|| {
            std::env::var("MACOSX_DEPLOYMENT_TARGET").ok().filter(|v| !v.is_empty()).or_else(|| {
                let out = std::process::Command::new("sw_vers").arg("-productVersion").output().ok()?;
                Some(String::from_utf8(out.stdout).ok()?.trim().to_string()).filter(|v| !v.is_empty())
            })
        })
        .clone()
}

/// The build's triple: this machine's, or LLVM's name for a no-OS
/// target (`aarch64-unknown-none`).
fn host_triple() -> inkwell::targets::TargetTriple {
    if crate::target::bare() {
        return inkwell::targets::TargetTriple::create(&crate::target::bare_triple());
    }
    static TRIPLE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let t = TRIPLE.get_or_init(|| {
        let default = TargetMachine::get_default_triple().as_str().to_string_lossy().into_owned();
        if !cfg!(target_os = "macos") {
            return default;
        }
        match (macos_version(), default.split_once("-apple-")) {
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
    // A bare chip is not this one: its baseline CPU, and code placed where
    // the linker puts it, with nothing to relocate it when it starts.
    if crate::target::bare() {
        Target::initialize_aarch64(&InitializationConfig::default());
        let target = Target::from_triple(&triple).map_err(|e| e.to_string())?;
        return target
            .create_target_machine(&triple, "generic", "", OptimizationLevel::Aggressive, RelocMode::Static, CodeModel::Small)
            .ok_or_else(|| format!("LLVM has no description of `{}`", crate::target::name()));
    }
    let target = Target::from_triple(&triple).map_err(|e| e.to_string())?;
    target
        .create_target_machine(&triple, &TargetMachine::get_host_cpu_name().to_string(), &TargetMachine::get_host_cpu_features().to_string(), OptimizationLevel::Aggressive, RelocMode::PIC, CodeModel::Default)
        .ok_or_else(|| "this machine has no target description".to_string())
}

extern "C" {
    /// `kayo/llvm-passes/pipeline.cpp`: `default<O3>` with kayo's own
    /// passes in it.
    fn kayo_optimize(module: llvm_sys::prelude::LLVMModuleRef, machine: llvm_sys::target_machine::LLVMTargetMachineRef, depth: u32, max_size: u32, inlined: *mut u32);
}

/// LLVM's whole pipeline, `default<O3>` as clang -O3 runs it, with
/// recursive inlining in it (owner, 2026-10-17): a function that calls
/// itself is inlined into itself `RECURSION_DEPTH` levels, while it stays
/// under `RECURSION_SIZE` instructions.
fn optimize(module: &Module<'_>, machine: &TargetMachine) -> u32 {
    // `KAYO_RECURSION_DEPTH=0` leaves recursion as written, to compare.
    let depth = std::env::var("KAYO_RECURSION_DEPTH").ok().and_then(|d| d.parse().ok()).unwrap_or(RECURSION_DEPTH);
    let mut inlined = 0u32;
    unsafe { kayo_optimize(module.as_mut_ptr(), machine.as_mut_ptr(), depth, RECURSION_SIZE, &mut inlined) };
    inlined
}

/// Levels a function is inlined into itself: past 3, bench fib gains
/// nothing (GCC's default is 8).
pub const RECURSION_DEPTH: u32 = 4;
/// Instructions a function may grow to by it, counted after the early
/// simplification (GCC's is 450 of its own).
pub const RECURSION_SIZE: u32 = 1000;

/// The passes `write_object` runs, as `opt -passes=…` spells them.
pub const PASSES: &str = "default<O3>";

thread_local! {
    /// Where `--emit-llvm-tv` writes (owner, 2026-10-15: "(a)").
    static TV_DIR: std::cell::RefCell<Option<std::path::PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// `--emit-llvm-tv <dir>`: the IR for translation validation (owner,
/// 2026-10-15: "(a)", check claude's Alive2), beside a build that is
/// otherwise the same.
pub fn set_tv_dir(dir: Option<std::path::PathBuf>) {
    TV_DIR.with(|d| *d.borrow_mut() = dir);
}

/// The module before and after KAYO\n's own passes with inlining off, for
/// Alive2 to check function by function: a copy, each function with a body
/// marked `noinline` and given weak linkage (so no pass changes a
/// function's signature, deletes it, or uses its body across a call), run through the same pipeline on the same machine with the same
/// facts from the prover, and the attributes the passes inferred on a
/// function taken off again. What is built is untouched.
fn write_tv(module: &Module, machine: &TargetMachine, dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot make {}: {}", dir.display(), e))?;
    let copy = module.clone();
    let noinline = inkwell::attributes::Attribute::get_named_enum_kind_id("noinline");
    let always = inkwell::attributes::Attribute::get_named_enum_kind_id("alwaysinline");
    for f in copy.get_functions() {
        if f.count_basic_blocks() == 0 {
            continue;
        }
        f.remove_enum_attribute(inkwell::attributes::AttributeLoc::Function, always);
        f.add_attribute(inkwell::attributes::AttributeLoc::Function, copy.get_context().create_enum_attribute(noinline, 0));
        // Weak, so no pass may take it as knowing every call, nor its body
        // as the one that runs: a constant carried in, an argument made
        // poison, a call replaced by what the body returns, a function
        // deleted would each change what is compared, function by function
        // (check claude: external still let `ret ptr %0` replace its
        // callers' calls). `main` stays as it is.
        if f.get_name().to_bytes() != b"main" {
            f.set_linkage(Linkage::WeakAny);
        }
    }
    let write = |name: &str, what: &str| -> Result<(), String> {
        let text = format!(
            "; {}\n; for translation validation (--emit-llvm-tv): every function with a body `noinline` and weak (`main` kept), the rest as built\n; passes: opt -passes='{}' (TargetMachine: {}, cpu {}, features {}, -O3, PIC)\n{}",
            what,
            PASSES,
            machine.get_triple().as_str().to_string_lossy(),
            machine.get_cpu().to_string(),
            machine.get_feature_string().to_string_lossy(),
            copy.print_to_string().to_string()
        );
        let file = dir.join(name);
        std::fs::write(&file, text).map_err(|e| format!("cannot write {}: {}", file.display(), e))
    };
    write("tv-before.ll", "as kayo hands it to LLVM, before any pass")?;
    // What each function says of itself before the passes, so what they
    // infer can be taken off again: attribute inference is about calls,
    // and some of what it writes (`nofree`) Alive2 cannot read (check
    // claude).
    use inkwell::attributes::AttributeLoc;
    let key = |a: &inkwell::attributes::Attribute| if a.is_string() { format!("s:{}", a.get_string_kind_id().to_string_lossy()) } else { format!("e:{}", a.get_enum_kind_id()) };
    let places = |f: &inkwell::values::FunctionValue| -> Vec<AttributeLoc> { std::iter::once(AttributeLoc::Function).chain(std::iter::once(AttributeLoc::Return)).chain((0..f.count_params()).map(AttributeLoc::Param)).collect() };
    let mut said: HashMap<(String, String), std::collections::HashSet<String>> = HashMap::new();
    for f in copy.get_functions().filter(|f| f.count_basic_blocks() > 0) {
        for loc in places(&f) {
            said.insert((f.get_name().to_string_lossy().into_owned(), format!("{:?}", loc)), f.attributes(loc).iter().map(key).collect());
        }
    }
    optimize(&copy, machine);
    for f in copy.get_functions().filter(|f| f.count_basic_blocks() > 0) {
        for loc in places(&f) {
            let before = said.get(&(f.get_name().to_string_lossy().into_owned(), format!("{:?}", loc))).cloned().unwrap_or_default();
            for a in f.attributes(loc) {
                if before.contains(&key(&a)) {
                    continue;
                }
                if a.is_string() {
                    f.remove_string_attribute(loc, &a.get_string_kind_id().to_string_lossy());
                } else {
                    f.remove_enum_attribute(loc, a.get_enum_kind_id());
                }
            }
        }
    }
    write("tv-after.ll", "after kayo's LLVM passes, inlining off, the attributes they inferred taken off")
}

/// With `emit`, the module is written there twice as text: `before.ll`, as
/// CCA hands it to LLVM (Precog's `nsw`/`nuw` and facts in it), and
/// `after.ll`, what LLVM's passes made of it and the backend gets.
pub fn write_object(module: &Module, path: &Path, emit: Option<&Path>) -> Result<ObjectFacts, String> {
    let mut facts = ObjectFacts::default();
    let machine = machine()?;
    module.set_triple(&host_triple());
    module.set_data_layout(&machine.get_target_data().get_data_layout());
    let write_ll = |name: &str, what: &str| -> Result<(), String> {
        let Some(dir) = emit else { return Ok(()) };
        std::fs::create_dir_all(dir).map_err(|e| format!("cannot make {}: {}", dir.display(), e))?;
        let text = format!(
            "; {}\n; passes: opt -passes='{}' (TargetMachine: {}, cpu {}, features {}, -O3, PIC)\n{}",
            what,
            PASSES,
            host_triple().as_str().to_string_lossy(),
            TargetMachine::get_host_cpu_name().to_string(),
            TargetMachine::get_host_cpu_features().to_string(),
            module.print_to_string().to_string()
        );
        let file = dir.join(name);
        std::fs::write(&file, text).map_err(|e| format!("cannot write {}: {}", file.display(), e))
    };
    write_ll("before.ll", "as kayo hands it to LLVM, before any pass")?;
    if let Some(dir) = TV_DIR.with(|d| d.borrow().clone()) {
        write_tv(module, &machine, &dir)?;
    }
    // LLVM's whole pipeline, as clang -O3 runs it (owner: optimised
    // everywhere). Without it the module went to the backend as written —
    // every local in memory, nothing inlined, no loop touched — and ran at
    // up to 5.6x C's time. Precog's proofs reach it as `nsw`/`nuw` and as
    // checks that are simply not there.
    let started = std::time::Instant::now();
    facts.recursion_inlined = optimize(module, &machine);
    facts.optimize_time = started.elapsed();
    facts.vector_fns = vector_fns(module);
    write_ll("after.ll", "after LLVM's passes, as the backend gets it")?;
    // With no OS, nothing will define what LLVM's code still calls; the
    // entry and the memory routines are the program's own.
    if crate::target::bare() {
        if let Some(name) = bare::missing(module).first() {
            return Err(bare::why_missing(name));
        }
        module.set_inline_assembly(&format!("{}\n{}", bare::ENTRY_ASM, bare::MEMORY_ASM));
    }
    let started = std::time::Instant::now();
    machine.write_to_file(module, FileType::Object, path).map_err(|e| e.to_string())?;
    facts.emit_time = started.elapsed();
    Ok(facts)
}

/// The functions whose optimised code computes on vectors: where LLVM's
/// loop and SLP vectorisers (or a `simd` value) put SIMD instructions.
fn vector_fns(module: &Module) -> Vec<String> {
    use inkwell::types::AnyTypeEnum;
    let mut out = Vec::new();
    let mut f = module.get_first_function();
    while let Some(func) = f {
        let vector = func.get_basic_block_iter().any(|b| {
            b.get_instructions().any(|i| matches!(i.get_type(), AnyTypeEnum::VectorType(_)) && !matches!(i.get_opcode(), inkwell::values::InstructionOpcode::Load | inkwell::values::InstructionOpcode::Store))
        });
        if vector {
            out.push(func.get_name().to_string_lossy().to_string());
        }
        f = func.get_next_function();
    }
    out
}

pub use inkwell;

extern "C" {
    /// lld through its library (`lld/shim.cpp`): the exit code, and what
    /// it said in `said`.
    fn cca_lld_link(argc: std::os::raw::c_int, argv: *const *const std::os::raw::c_char, said: *mut std::os::raw::c_char, said_len: usize) -> std::os::raw::c_int;
}

/// Link with lld, built into CCA (owner, 2026-10-08: "We'll use `lld`",
/// then "`a`": its library, Zig's way); CyborgPL never writes its own
/// linker. On macOS, the arguments clang gives the system linker: the
/// architecture, the platform and SDK versions, the SDK as the system
/// root, the runtime, and libSystem.
/// A final build's debug info, written beside the program and taken out
/// of it (owner, 2026-10-08: "`a`"; 2026-10-14: "(a)", "(a)";
/// `debug-info.md`): `hello.dSYM` on macOS, `hello.debug` (DWARF, with a
/// `.gnu_debuglink` to it) on Linux and with no OS. LLVM's own tools,
/// beside the LLVM Synapse is built with; the object file must still be
/// there, since macOS's debug map points into it. Gives back the file.
pub fn split_debug(out: &Path) -> Result<std::path::PathBuf, String> {
    let tool = |name: &str| Path::new(env!("KAYO_LLVM_PREFIX")).join("bin").join(name);
    let run = |name: &str, args: &[&std::ffi::OsStr]| -> Result<(), String> {
        let o = std::process::Command::new(tool(name)).args(args).output().map_err(|e| format!("`{}` would not start, and a final build writes its debug info with it: {}", name, e))?;
        if o.status.success() {
            Ok(())
        } else {
            Err(format!("`{}` could not write the debug info: {}", name, String::from_utf8_lossy(&o.stderr).trim()))
        }
    };
    let beside = |ending: &str| {
        let mut p = out.as_os_str().to_os_string();
        p.push(ending);
        std::path::PathBuf::from(p)
    };
    let target = crate::target::name();
    if target.ends_with("-macos") {
        let dsym = beside(".dSYM");
        let _ = std::fs::remove_dir_all(&dsym);
        run("dsymutil", &[out.as_os_str(), "-o".as_ref(), dsym.as_os_str()])?;
        run("llvm-strip", &["--strip-debug".as_ref(), out.as_os_str()])?;
        return Ok(dsym);
    }
    if target.ends_with("-linux") || target.ends_with("-none") {
        let debug = beside(".debug");
        run("llvm-objcopy", &["--only-keep-debug".as_ref(), out.as_os_str(), debug.as_os_str()])?;
        let link = std::ffi::OsString::from(format!("--add-gnu-debuglink={}", debug.display()));
        run("llvm-objcopy", &["--strip-debug".as_ref(), link.as_os_str(), out.as_os_str()])?;
        return Ok(debug);
    }
    Err(format!("not yet: a final build for `{}` writes its debug info beside the program, and kayo knows that file only for macOS (`.dSYM`), Linux and no OS (`.debug`) so far", target))
}

pub fn link(obj: &Path, out: &Path) -> Result<(), String> {
    if crate::target::bare() {
        let Some(script) = crate::target::layout() else {
            return Err(format!("a program for `{}` brings its own linker script, which says where it sits in the board's memory (owner, 2026-10-09: \"`c` now\"), and this one has none", crate::target::name()));
        };
        return bare::link(obj, &script.0, &script.1, out);
    }
    let runtime = runtime_path().ok_or_else(|| "the runtime is not built: `cargo build -p kayo-runtime --release`".to_string())?;
    if !cfg!(target_os = "macos") {
        return Err("linking with lld is built for macOS so far: this host's system libraries and start files are not found yet".into());
    }
    let xcrun = |what: &str| -> Result<String, String> {
        let o = std::process::Command::new("xcrun").arg(what).output().map_err(|e| format!("`xcrun {}` would not start, and lld needs the macOS SDK: {}", what, e))?;
        let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if !o.status.success() || s.is_empty() {
            return Err(format!("`xcrun {}` found no macOS SDK, which lld links against: install the Command Line Tools (`xcode-select --install`)", what));
        }
        Ok(s)
    };
    let sdk = xcrun("--show-sdk-path")?;
    let sdk_version = xcrun("--show-sdk-version")?;
    let min = macos_version().unwrap_or_else(|| sdk_version.clone());
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => other,
    };
    let mut args: Vec<String> = vec![
        "ld64.lld".into(),
        "-arch".into(),
        arch.into(),
        "-platform_version".into(),
        "macos".into(),
        min,
        sdk_version,
        "-syslibroot".into(),
        sdk,
        "-o".into(),
        out.to_string_lossy().into_owned(),
        obj.to_string_lossy().into_owned(),
        runtime.to_string_lossy().into_owned(),
        "-lSystem".into(),
    ];
    // The C libraries the kits list (owner, 2026-10-19), found in the
    // system's folders (the SDK) and Homebrew's.
    let libs = crate::target::c_libs();
    if !libs.is_empty() {
        for dir in ["/opt/homebrew/lib", "/usr/local/lib"] {
            if Path::new(dir).is_dir() {
                args.push(format!("-L{}", dir));
            }
        }
        args.extend(libs.iter().map(|l| format!("-l{}", l)));
    }
    let c: Vec<std::ffi::CString> = args.iter().map(|a| std::ffi::CString::new(a.as_str()).map_err(|_| format!("a path holds a NUL byte: {}", a))).collect::<Result<_, _>>()?;
    let argv: Vec<*const std::os::raw::c_char> = c.iter().map(|a| a.as_ptr()).collect();
    let mut said = vec![0 as std::os::raw::c_char; 16 * 1024];
    let code = unsafe { cca_lld_link(argv.len() as std::os::raw::c_int, argv.as_ptr(), said.as_mut_ptr(), said.len()) };
    if code == 0 {
        return Ok(());
    }
    let said = unsafe { std::ffi::CStr::from_ptr(said.as_ptr()) }.to_string_lossy().trim().to_string();
    Err(if said.is_empty() { "the linker (lld) refused the program".into() } else { format!("the linker (lld) refused the program:\n{}", said) })
}

fn runtime_path() -> Option<std::path::PathBuf> {
    // Beside the compiler, which is where cargo puts it.
    let exe = std::env::current_exe().ok()?;
    for dir in exe.parent()?.ancestors().take(4) {
        for profile in ["release", "debug"] {
            let p = dir.join(profile).join("libkayo_runtime.a");
            if p.is_file() {
                return Some(p);
            }
        }
        let p = dir.join("libkayo_runtime.a");
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Every block of a `no_cheat` loop, a `no_cheat` statement and a
/// `no_cheat` copy of a function, by function.
impl<'a, 'ctx> Cg<'a, 'ctx> {
    /// A function Precog made a table of: its answers in a constant array,
    /// and a body that reads the one its arguments name. Each argument is
    /// within its type's range, which is what the table covers.
    fn define_table(&mut self, i: usize, func: FunctionValue<'ctx>, t: &crate::precog::tables::Table) -> Option<()> {
        let f = &self.p.functions[i];
        let BasicTypeEnum::IntType(rt) = self.ty(&f.ret)? else { return None };
        let vals: Vec<IntValue<'ctx>> = t.entries.iter().map(|c| match c {
            Const::Int(v) => Some(const_i128(rt, *v)),
            Const::Bool(b) => Some(rt.const_int(*b as u64, false)),
            _ => None,
        }).collect::<Option<_>>()?;
        let arr = rt.const_array(&vals);
        let g = self.module.add_global(arr.get_type(), None, &format!("cy_table_{}", i));
        g.set_initializer(&arr);
        g.set_constant(true);
        g.set_linkage(Linkage::Private);
        g.set_unnamed_addr(true);
        let entry = self.ctx.append_basic_block(func, "table");
        self.builder.position_at_end(entry);
        let w = self.ctx.i64_type();
        let sizes: Vec<u64> = t.ranges.iter().map(|(lo, hi)| (hi - lo + 1) as u64).collect();
        let mut idx = w.const_zero();
        for (k, (p, (lo, _))) in f.params.iter().zip(&t.ranges).enumerate() {
            let pv = func.get_nth_param(k as u32)?.into_int_value();
            let signed = int_signed(&f.locals[p.0 as usize].ty).unwrap_or(true);
            let x = self.builder.build_int_cast_sign_flag(pv, w, signed, "arg").ok()?;
            let off = self.builder.build_int_sub(x, const_i128(w, *lo), "from_lowest").ok()?;
            let stride: u64 = sizes[k + 1..].iter().product();
            let part = self.builder.build_int_mul(off, w.const_int(stride, false), "part").ok()?;
            idx = self.builder.build_int_add(idx, part, "index").ok()?;
        }
        let at = unsafe { self.builder.build_in_bounds_gep(arr.get_type(), g.as_pointer_value(), &[w.const_zero(), idx], "answer").ok()? };
        let v = self.builder.build_load(rt, at, "answer").ok()?;
        self.builder.build_return(Some(&v)).ok()?;
        Some(())
    }

    /// Every struct a live function holds, as the target lays it out.
    fn struct_layouts(&mut self) -> Vec<StructLayout> {
        let mut seen: Vec<Ty> = Vec::new();
        let mut work: Vec<Ty> = Vec::new();
        for f in self.p.functions.iter().filter(|f| !f.dropped) {
            work.extend(f.locals.iter().map(|l| l.ty.clone()));
        }
        while let Some(t) = work.pop() {
            match t.base() {
                Ty::Struct { id, args } => {
                    if seen.contains(t.base()) {
                        continue;
                    }
                    let info = &self.p.items.structs[id.0 as usize];
                    if info.opaque {
                        continue;
                    }
                    work.extend(info.fields.iter().map(|f| f.ty.subst(args)));
                    seen.push(t.base().clone());
                }
                Ty::Seq { elem, .. } | Ty::Channel(elem) => work.push((**elem).clone()),
                Ty::Map { key, val, .. } => {
                    work.push((**key).clone());
                    work.push((**val).clone());
                }
                Ty::Bundle(ts) => work.extend(ts.iter().cloned()),
                _ => {}
            }
        }
        let spell = |p: &Program, t: &Ty| crate::types::TySpelling { ty: t, structs: &p.items.structs, enums: &p.items.enums }.to_string();
        let mut out = Vec::new();
        for t in seen {
            let Ty::Struct { id, args } = &t else { continue };
            // A C union or bit-field struct: where C puts each field (a
            // bit-field's unit; its type says its bits).
            if let Some(l) = self.c_layout(&t) {
                let info = self.p.items.structs[id.0 as usize].clone();
                let fields = info.fields.iter().zip(&l.fields).map(|(f, c)| (f.name.clone(), spell(self.p, &f.ty.subst(args)), c.byte, c.unit, 0)).collect();
                out.push(StructLayout { name: spell(self.p, &t), size: l.size, align: l.align as u32, fields });
                continue;
            }
            let Some(BasicTypeEnum::StructType(st)) = self.ty(&t) else { continue };
            let info = self.p.items.structs[id.0 as usize].clone();
            let size = self.target.get_abi_size(&st);
            let align = self.target.get_abi_alignment(&st);
            let mut fields = Vec::new();
            for (i, fi) in info.fields.iter().enumerate() {
                let ft = fi.ty.subst(args);
                let at = self.target.offset_of_element(&st, i as u32).unwrap_or(0);
                let fsize = st.get_field_type_at_index(i as u32).map(|f| self.target.get_abi_size(&f)).unwrap_or(0);
                let next = if i + 1 < info.fields.len() { self.target.offset_of_element(&st, i as u32 + 1).unwrap_or(size) } else { size };
                fields.push((fi.name.clone(), spell(self.p, &ft), at, fsize, next.saturating_sub(at + fsize)));
            }
            out.push(StructLayout { name: spell(self.p, &t), size, align, fields });
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    /// Every kit-level `const` whose type the target gives a size: its
    /// name, type and bytes. An `incl.…` file's are the file's.
    fn const_sizes(&mut self) -> Vec<(String, String, u64)> {
        let mut out = Vec::new();
        for c in self.p.items.consts.clone() {
            let Some(t) = &c.ty else { continue };
            let mut spelled = crate::types::TySpelling { ty: t, structs: &self.p.items.structs, enums: &self.p.items.enums }.to_string();
            match t.base() {
                Ty::Int(IntW::Free(_)) => spelled.push_str(" (built as i64)"),
                Ty::Bin(BinW::Free(_)) => spelled.push_str(" (built as f64)"),
                _ => {}
            }
            if let crate::ast::ExprKind::Incl { path, .. } = &c.value.kind {
                let dir = self.p.items.neuron_dirs.get(&self.p.items.kit_root(c.module)).cloned().unwrap_or_default();
                let bytes = std::fs::metadata(dir.join(path)).map(|m| m.len()).unwrap_or(0);
                out.push((c.name.clone(), format!("{}, from `{}`", spelled, path), bytes));
                continue;
            }
            if let Some(size) = self.size_of_ty(t) {
                out.push((c.name.clone(), spelled, size));
            }
        }
        out
    }
}

fn nocheat_blocks(p: &crate::mir::Program) -> std::collections::HashSet<(usize, usize)> {
    let mut out = std::collections::HashSet::new();
    for (fi, f) in p.functions.iter().enumerate() {
        if f.nocheat {
            out.extend((0..f.blocks.len()).map(|b| (fi, b)));
        }
        out.extend(f.nocheat_blocks.iter().map(|b| (fi, *b as usize)));
        for fl in f.fors.iter().filter(|fl| fl.nocheat) {
            for b in crate::par::loop_blocks(f, fl) {
                out.insert((fi, b));
            }
        }
    }
    out
}

/// Where a counting loop's way round may go instead of its head: straight
/// into its body. The head asks whether the counter is still within the
/// stop, and coming round the answer is already known — the step was taken
/// only because there was room for it below the stop (above it, counting
/// down). Asking again cost the sieve's inner loop a second compare on
/// every trip (2026-10-15). Only a head that asks nothing else: no checks,
/// no `!cheat` barrier. In a slice the stop is the slice's last trip, and
/// the room has to be measured to it, not to a constant the stop was folded
/// to — which is the loop's own stop.
fn back_to_body(f: &Function, b: usize, head: BlockId, in_slice: bool) -> Option<BlockId> {
    let fl = f.fors.iter().find(|fl| fl.head == head && fl.pre.0 as usize != b && !fl.nocheat)?;
    // A step written as a number LLVM already folds the two tests into one,
    // and does better with the head's than with the room's alone (bench
    // lists: 3% slower).
    if fl.step.is_none_or(|st| step_is_const(f, st)) {
        return None;
    }
    let room_reads_stop = || {
        f.blocks.iter().flat_map(|blk| &blk.stmts).any(|s| matches!(&s.kind, StmtKind::Assign { rvalue: Rvalue::Convert { operand: Operand::Read(p), .. }, .. } if p.local == fl.stop && p.proj.is_empty()))
    };
    if in_slice && !room_reads_stop() {
        return None;
    }
    let only_temps = |blk: BlockId| {
        f.blocks[blk.0 as usize].stmts.iter().all(|s| match &s.kind {
            StmtKind::Assign { place, rvalue: Rvalue::Binary { .. } | Rvalue::Use(Operand::Const(_)) } => place.proj.is_empty() && f.locals[place.local.0 as usize].name.is_none(),
            _ => false,
        })
    };
    let Terminator::If { then: up, else_: down, .. } = &f.blocks[head.0 as usize].term else { return None };
    let Terminator::If { then: body, else_: out_up, .. } = &f.blocks[up.0 as usize].term else { return None };
    let Terminator::If { then: body_down, else_: out_down, .. } = &f.blocks[down.0 as usize].term else { return None };
    (body == body_down && *out_up == fl.exit && *out_down == fl.exit && only_temps(head) && only_temps(*up) && only_temps(*down)).then_some(*body)
}

/// A counting loop's step when it is a number written in the program.
pub(super) fn step_const(f: &Function, step: LocalId) -> Option<i128> {
    let mut given = f.blocks.iter().flat_map(|b| &b.stmts).filter_map(|s| match &s.kind {
        StmtKind::Assign { place, rvalue } if place.local == step && place.proj.is_empty() => Some(rvalue),
        _ => None,
    });
    match (given.next(), given.next()) {
        (Some(Rvalue::Use(Operand::Const(Const::Int(v)))), None) => Some(*v),
        _ => None,
    }
}

/// Whether a counting loop's step is a number written in the program: the
/// one place it is given a value is a constant.
fn step_is_const(f: &Function, step: LocalId) -> bool {
    let mut given = f.blocks.iter().flat_map(|b| &b.stmts).filter_map(|s| match &s.kind {
        StmtKind::Assign { place, rvalue } if place.local == step && place.proj.is_empty() => Some(rvalue),
        _ => None,
    });
    matches!((given.next(), given.next()), (Some(Rvalue::Use(Operand::Const(_))), None))
}
