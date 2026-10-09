//! Tree-walking evaluation. Slow, small, obvious — the point is to be right.
//!
//! What it enforces beyond the parser:
//!   - the typed form's ranges and kinds (`let 'x': int8 = 300;` is an error)
//!   - no implicit conversion between numeric families; a bare numeric
//!     *literal* is untyped and takes the family of its context
//!   - ownership: a bare `'t'` where an owned value would be *stored* is an
//!     error; `give` kills the source; `lend`/`lent` cannot be given away or
//!     assigned to; `lend.mut` writes through
//!   - the default check tier: every integer operation is overflow-checked,
//!     every division checks for zero — there is no Cortex here to remove them
//!   - `if`/`while` conditions must be bool

use crate::ast::*;
use crate::value::{Ptr, FuncVal, Chan, Timer};
use crate::value::{exact_decimal, int_range, ty_name, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;

#[derive(Debug)]
pub struct RuntimeError {
    pub line: usize,
    pub msg: String,
    /// `std::exit` unwinds through here.
    pub exit: Option<u8>,
    /// a safety check fired (overflow, range, fit, zero) — the thing the tiers govern
    pub site: bool,
    /// not an error: a control-flow exit (`otherwise return …`, `otherwise fail`) that
    /// has to climb out of an expression to the enclosing statement
    pub leave: Option<Flow>,
}

thread_local! { static SHOWN_FILE: std::cell::RefCell<String> = std::cell::RefCell::new(String::new()); }

/// The file a message's second places name, without its folders (`shop.nrv`).
pub fn set_shown_file(name: &str) { SHOWN_FILE.with(|f| *f.borrow_mut() = name.to_string()); }

/// A second place in a message gives its file, always (owner, 2026-10-08: "`a`"): `line 6 of shop.nrv`.
/// A Neuron's places all name the program's root file: the oracle keeps no file per line.
pub fn line_of(line: impl std::borrow::Borrow<usize>) -> String {
    let line = *line.borrow();
    SHOWN_FILE.with(|f| { let f = f.borrow(); if f.is_empty() { format!("line {line}") } else { format!("line {line} of {f}") } })
}

fn rt<T>(line: usize, msg: impl Into<String>) -> Result<T, RuntimeError> {
    Err(RuntimeError { line, msg: msg.into(), exit: None, site: false, leave: None })
}

/// A failed safety check at a check site.
fn site<T>(line: usize, msg: impl Into<String>) -> Result<T, RuntimeError> {
    Err(RuntimeError { line, msg: msg.into(), exit: None, site: true, leave: None })
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum SlotKind { Owned, Lent, LentMut, Const, Extern, ExternMut }

#[derive(Clone)]
struct Slot {
    cell: Rc<RefCell<Value>>,
    ty: Option<Ty>,
    kind: SlotKind,
    alive: bool,     // false after `give`
    immut: bool,     // stated `immut`: assignment is an error
    sized: bool,     // the list's length is known here (a literal, a bound, a bundle) — `...'xs'` may unpack it
    deps: Vec<usize>, // a borrow a call handed back under `@x`: every cell in the group (owner, 2026-09-29: they live and die together)
    since: u64,      // a lent slot: the write count when it was bound; a later write to what it borrows makes it stale
    sealed: bool,    // `!$`: the name cannot be shadowed (owner, 2026-09-25); a loop's counter is too
}

#[derive(Debug)]
pub enum Flow { Normal, Break, Continue, Return(Value), Fail(String, Option<Value>), Give(Value) }

/// A started, not-yet-run task (design/concurrency.md).
struct Task {
    body: Vec<Stmt>,
    captures: Vec<(String, Rc<RefCell<Value>>, bool)>,
    handler: Option<Handler>,
    fails: Vec<FailCase>,
    module: Vec<String>,
    /// the scope depth of the block that started it: that block waits for it at its end
    depth: usize,
    detached: bool,
    line: usize,
}

/// What an evaluated expression *is*, for ownership and typing purposes.
enum Prov {
    Literal,                             // a bare numeric literal: untyped, adopts its context's family
    Raw,                                 // a whole-number literal in a Neuron without ASP: its bits land as they are
    Plain,                               // a plain value: copies freely
    Fresh,                               // a newly made owned value (literal, call result): storable
    Given,                               // `give 'x'`: ownership transferred here
    Copied,                              // `copy 'x'`: an independent duplicate
    Borrowed(Rc<RefCell<Value>>, bool),  // lend / lend.mut of a variable (bool = mutable)
    Bare(String),                        // a bare reference to an owned variable: readable, not storable
}

/// One `for 'x' in lend.mut 'xs'` loop in progress.
struct MutWalk {
    list: Rc<RefCell<Value>>,
    elem: Rc<RefCell<Value>>,
    name: String,
    elem_name: String,
    pos: std::cell::Cell<usize>,
    /// the line where this trip first read the list, if it has
    read_at: std::cell::Cell<Option<usize>>,
}

pub struct Interp<'p> {
    prog: &'p Program,
    /// Writes to borrowed things (check claude, 2026-09-29): each owner's cell → (write count, line, what).
    /// A lent slot made before the latest write is stale, and using it is refused: a borrow never
    /// aliases a mutation and never outlives its owner (core-features.md).
    writes: HashMap<usize, (u64, usize, &'static str, String)>,
    write_count: u64,
    /// the group cells of the last returned `-> lent@x` borrow, until a declaration takes them
    call_deps: Vec<usize>,
    pending_deps: Vec<usize>,
    /// the line of the `return` that ended the last call
    ret_line: usize,
    /// inside `must.fail`: a failure passes up to it whatever the test's own `fail[…]`
    must_fail_depth: u32,
    /// the running function's return type: what a `return` lands in, whatever surrounds it
    cur_ret: Option<Ty>,
    /// the command line, program name first, as C's argv has it
    args: Vec<String>,
    /// the matched case's data type, for the name an arm binds
    arm_ty: Option<Ty>,
    /// Neuron-level `const`s by canonical key (`module/name`): never assignable, every read a fresh value.
    consts: HashMap<String, Slot>,
    /// the module the running code was written in (for bare names and visibility)
    module: Vec<String>,
    /// the ways the running function may fail (empty in MAIN)
    fails: Vec<FailCase>,
    /// the call being evaluated is a statement (its value is unused), so an
    /// `otherwise { … }` arm may fall through
    call_is_stmt: bool,
    /// the type the value being evaluated is being stored into, when there is one:
    /// a bare literal takes its width from its context (owner, 2026-09-23)
    expected: Option<Ty>,
    /// a conversion's target, which a bare literal in its `otherwise` takes as its family (`types.md`:
    /// a literal takes its context's; check claude, 2026-09-30)
    fallback_ty: Option<Ty>,
    /// whether the last value evaluated is statically a list (false) or an array (true), and
    /// the same for the value a `return` handed back: a list never becomes an array by itself
    /// ("nothing converts by itself", 2026-09-19; owner, 2026-09-23: `to.array[…]`, `to.list[…]`)
    last_kind: Option<bool>,
    /// the digits of the last value evaluated, when it was a bin literal: a literal is rounded
    /// once, from its digits, to a bin16/bin32 (compiler session, 2026-09-24)
    last_lit: Option<String>,
    ret_kind: Option<bool>,
    /// a `return lend 'x'` in flight: the lent cell and whether it is mutable (owner, 2026-09-29)
    ret_borrow: Option<(Rc<RefCell<Value>>, bool)>,
    /// the borrow the last call handed back, for the expression that called it
    call_borrow: Option<(Rc<RefCell<Value>>, bool)>,
    /// tasks started by `concur` and not yet run (design/concurrency.md). The oracle's
    /// schedule: a task runs when something waits on it (a `take`, a `whichever`, a full
    /// `send`) or when the block that started it ends — in source order. One valid schedule.
    tasks: Vec<Task>,
    next_chan_id: usize,
    /// how many `nocheck` statements enclose the running code (extern calls and
    /// pointer reads/writes need at least one — design/ffi.md)
    nocheck_depth: usize,
    /// inside `NO_FUCKING_CHECKS`: C's numbers are trusted, no `out_of_range` (owner, 2026-09-27)
    trust_c_depth: usize,
    /// type variables (`<T>`) of the running generic function, bound to a value's
    /// shape as arguments arrive (design/generics.md); the oracle checks
    /// consistency by kind, the compiler specialises per type
    tvars: std::cell::RefCell<HashMap<String, String>>,
    /// `lend.mut 'x'.field` hands out a temporary cell; at the end of the statement
    /// its value is written back into the field (struct cell, field name, temp cell)
    field_borrows: Vec<(Rc<RefCell<Value>>, String, Rc<RefCell<Value>>)>,
    /// `for 'x' in lend.mut 'xs'` loops running now: the list may be read while 'x' is finished
    /// for the trip, and sees the new values (owner, 2026-09-30: a, Synapse's liveness rule)
    mut_walks: Vec<MutWalk>,
    scopes: Vec<HashMap<String, Slot>>,
    out: Box<dyn Write>,
    /// each `pub` thing's `bare` mark, by `kind:module.name` (owner, 2026-10-06)
    bare_of: HashMap<String, crate::ast::Bare>,
}

impl<'p> Interp<'p> {
    pub fn new(prog: &'p Program, out: Box<dyn Write>) -> Self {
        Interp { prog, writes: HashMap::new(), write_count: 0, call_deps: Vec::new(), pending_deps: Vec::new(), ret_line: 0, arm_ty: None, must_fail_depth: 0, cur_ret: None, args: Vec::new(), consts: HashMap::new(), module: Vec::new(), fails: Vec::new(), call_is_stmt: false, expected: None, fallback_ty: None, last_kind: None, last_lit: None, ret_kind: None, ret_borrow: None, call_borrow: None, tasks: Vec::new(), next_chan_id: 0, nocheck_depth: 0, trust_c_depth: 0, tvars: std::cell::RefCell::new(HashMap::new()), field_borrows: Vec::new(), mut_walks: Vec::new(), scopes: vec![HashMap::new()], out, bare_of: HashMap::new() }
    }

    /// Ok(code): the program ran; Err: it failed.
    pub fn set_args(&mut self, args: Vec<String>) { self.args = args; }
    pub fn set_bare(&mut self, b: HashMap<String, crate::ast::Bare>) { self.bare_of = b; }

    /// Whether this code's Neuron has IEG, whose rule puts raw memory and unproven code under `!check`;
    /// Brainstem alone marks nothing (owner, 2026-10-06: "Brainstem marks nothing", then "yes" to
    /// `inline.ASM` and the rest needing `!check` only where IEG is)
    fn marks_needed(&self) -> bool {
        let root: Vec<String> = self.module.first().filter(|r| r.starts_with("::")).cloned().into_iter().collect();
        !crate::parser::LACKS.with(|l| l.borrow().iter().any(|(r, lacks)| *r == root && lacks.iter().any(|c| c == "IEG")))
    }

    /// Another Neuron's `pub` functions reached bare by `written`, from a call in `from` (owner, 2026-10-06):
    /// those marked `bare.optional` or `bare.only`, in a Neuron other than the caller's.
    fn bare_fns_elsewhere(&self, from: &[String], written: &[String], op: bool) -> Vec<&'p Func> {
        let mine = from.first().filter(|r| r.starts_with("::")).cloned();
        let prog: &'p Program = self.prog;
        prog.funcs.iter().filter(|f| {
            let Some(root) = f.module.first().filter(|r| r.starts_with("::")) else { return false };
            if Some(root) == mine.as_ref() || f.vis != crate::ast::Vis::Pub || f.is_op != op { return false; }
            let rel: Vec<&str> = f.module[1..].iter().map(String::as_str).chain(std::iter::once(f.name.as_str())).collect();
            rel == written.iter().map(String::as_str).collect::<Vec<_>>()
                && matches!(self.bare_of.get(&bare_key(if op { "op" } else { "fn" }, &f.module, &f.name)), Some(crate::ast::Bare::Optional | crate::ast::Bare::Only))
        }).collect()
    }

    pub fn run(&mut self) -> Result<u8, RuntimeError> {
        self.prepare()?;
        let main = match &self.prog.main {
            Some(m) => m,
            None => return rt(0, "this Neuron has no MAIN, so it is a library, not a program"),
        };
        // `MAIN ['argc': int32, 'argv': ptr[ptr[uint8]]]`: C's own view of the command line — the
        // program's name first, each word NUL-ended, and a null pointer after the last (owner, 2026-09-29)
        self.push();
        if let Some((argc, argv)) = &self.prog.main_args {
            let words: Vec<Value> = self.args.iter()
                .map(|w| Value::Ptr(Ptr { cell: Some(Rc::new(RefCell::new(Value::Str(w.clone())))), index: 0, mutable: false }))
                .chain(std::iter::once(Value::Ptr(Ptr { cell: None, index: 0, mutable: false })))
                .collect();
            let n = Value::Int(self.args.len() as i128);
            let vec = Value::Ptr(Ptr { cell: Some(Rc::new(RefCell::new(Value::List(words)))), index: 0, mutable: false });
            let int32 = Ty::Int(crate::ast::IntWidth::W32);
            let pp = Ty::Ptr(Box::new(Ty::Ptr(Box::new(Ty::Uint(crate::ast::IntWidth::W8)), false)), false);
            self.declare(argc, Slot { cell: Rc::new(RefCell::new(n)), ty: Some(int32), kind: SlotKind::Owned, alive: true, immut: true, sized: false, sealed: false, since: 0, deps: Vec::new() }, 0)?;
            self.declare(argv, Slot { cell: Rc::new(RefCell::new(vec)), ty: Some(pp), kind: SlotKind::Owned, alive: true, immut: true, sized: false, sealed: false, since: 0, deps: Vec::new() }, 0)?;
        }
        let r = self.block(main);
        self.pop();
        match r {
            Ok(Flow::Normal) => Ok(0),
            Ok(Flow::Return(_)) => rt(0, "`return` inside MAIN: MAIN is a block, not a function; use cord::exit[…] for a code"),
            Ok(Flow::Fail(n, _)) => rt(0, format!("MAIN cannot fail (`{n}`); it exits — handle it, or cord::exit[…]")),
            Ok(Flow::Break) | Ok(Flow::Continue) => rt(0, "`break`/`continue` outside a loop"),
            Ok(Flow::Give(_)) => rt(0, "`giveS` outside a case"),
            Err(RuntimeError { exit: Some(code), .. }) => Ok(code),
            Err(e) => Err(e),
        }
    }

    /// `synapse test` (owner, 2026-09-29): every `test fn`, in the order written, each on its own;
    /// a test passes unless a `must` does not hold or something else stops it. Exit 0 when all pass.
    pub fn run_tests(&mut self) -> Result<u8, RuntimeError> {
        self.prepare()?;
        let tests: Vec<Func> = self.prog.funcs.iter().filter(|f| f.is_test).cloned().collect();
        let (mut passed, mut failed) = (0, 0);
        for t in &tests {
            let r = self.run_body(t, Vec::new());
            let line = match r {
                Ok(_) => { passed += 1; format!("test {} ... ok", t.name) }
                Err(e) => { failed += 1; format!("test {} ... FAILED: {}: {}", t.name, line_of(e.line), if e.msg.is_empty() { "it left early".to_string() } else { e.msg.clone() }) }
            };
            writeln!(self.out, "{line}").map_err(|e| RuntimeError { line: 0, msg: e.to_string(), exit: None, site: false, leave: None })?;
        }
        writeln!(self.out, "{} tests: {passed} passed, {failed} failed", tests.len()).map_err(|e| RuntimeError { line: 0, msg: e.to_string(), exit: None, site: false, leave: None })?;
        Ok(if failed == 0 { 0 } else { 1 })
    }

    /// The checks before anything runs, then the consts and C's variables.
    fn prepare(&mut self) -> Result<(), RuntimeError> {
        // Consts first, in order: each may use the ones above it. Cortex would
        // fold these at compile time; the oracle evaluates them before MAIN.
        self.check_duplicates()?;
        self.check_returned_borrows()?;
        self.check_borrow_order()?;
        self.check_fails()?;
        self.check_claims()?;
        self.check_boundary()?;
        for c in &self.prog.consts {
            let key = const_key(&c.module, &c.name);
            if self.consts.contains_key(&key) { return rt(c.line, format!("const '{}' is declared twice", c.name)); }
            self.module = c.module.clone();
            if let Some(call) = first_std_call(&c.value) {
                return rt(c.line, format!("a const's value must be known at compile time; `cord::{call}` is an effect, not a value"));
            }
            if c.value.values.is_empty() { return rt(c.line, format!("const '{}' needs a value", c.name)); }
            let (v, prov) = self.bracket(&c.value, c.line)?;
            if matches!(prov, Prov::Borrowed(..)) { return rt(c.line, format!("const '{}' cannot hold a lend", c.name)); }
            let v = self.fit(v, prov, c.ty.as_ref(), c.line)?;
            self.consts.insert(key, Slot { cell: Rc::new(RefCell::new(v)), ty: c.ty.clone(), kind: SlotKind::Const, alive: true, immut: true, sized: false, sealed: false , since: 0, deps: Vec::new() });
        }
        // extern variables live beside the consts: C's memory, emulated as zero until written
        for x in &self.prog.externs {
            let key = const_key(&x.module, &x.name);
            if self.consts.contains_key(&key) { return rt(x.line, format!("'{}' is declared twice", x.name)); }
            let zero = match &x.ty { Ty::Bin(_) => Value::Bin(0.0), Ty::Bool => Value::Bool(false), Ty::Ptr(..) => Value::Ptr(Ptr { cell: None, index: 0, mutable: x.mutable }), _ => Value::Int(0) };
            self.consts.insert(key, Slot { cell: Rc::new(RefCell::new(zero)), ty: Some(x.ty.clone()), kind: if x.mutable { SlotKind::ExternMut } else { SlotKind::Extern }, alive: true, immut: !x.mutable, sized: false, sealed: false , since: 0, deps: Vec::new() });
        }
        self.module = Vec::new();
        Ok(())
    }

    // ---- scopes -----------------------------------------------------------

    fn push(&mut self) { self.scopes.push(HashMap::new()); }
    fn pop(&mut self) { self.scopes.pop(); }
    fn lookup(&self, name: &str) -> Option<&Slot> { self.scopes.iter().rev().find_map(|s| s.get(name)).or_else(|| self.consts.get(&const_key(&self.module, name))) }
    fn lookup_mut(&mut self, name: &str) -> Option<&mut Slot> { self.scopes.iter_mut().rev().find_map(|s| s.get_mut(name)) }
    fn is_const(&self, name: &str) -> bool { self.consts.contains_key(&const_key(&self.module, name)) && !self.scopes.iter().any(|s| s.contains_key(name)) }

    /// Two items with one canonical name, or a module declared twice (a file and a block).
    fn check_duplicates(&self) -> Result<(), RuntimeError> {
        let mut seen: HashMap<String, usize> = HashMap::new();
        for f in self.prog.funcs.iter().filter(|f| !f.is_op) {
            let k = format!("fn {}", join_path(&f.module, &f.name));
            if let Some(l) = seen.insert(k.clone(), f.line) { return rt(f.line, format!("`{}` is defined twice (first at {})", &k[3..], line_of(l))); }
        }
        // An operator may be defined for several types (2026-09-20), generic or not (owner,
        // 2026-09-29), but two meanings for the same types are an error (2026-09-23), and so is a
        // specific one a generic one already covers (owner, 2026-09-29: a — no quiet "specific wins").
        let ops: Vec<&Func> = self.prog.funcs.iter().filter(|f| f.is_op).collect();
        let generic = |f: &Func| -> Option<Option<String>> { match f.params.first().and_then(|p| p.ty.as_ref()) {
            Some(Ty::Var(v)) => Some(f.bounds.iter().find(|(n, _)| n == v).map(|(_, i)| i.clone())), _ => None } };
        let shapes = |f: &Func| -> Vec<String> { f.params.iter().map(|p| p.ty.as_ref().map(ty_shape).unwrap_or_default()).collect() };
        let root = |f: &Func| f.module.first().filter(|r| r.starts_with("::")).cloned();
        let word = |n: &str| n.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.');
        for (j, g) in ops.iter().enumerate() {
            for f in &ops[..j] {
                if f.name != g.name { continue; }
                // two Neurons may each define one word operator for the same types: the path tells them
                // apart (owner, 2026-10-02), and a bare use of it is refused where it is written; a glyph
                // operator has no path, so two of them stay an error
                if root(f) != root(g) && word(&f.name) { continue; }
                let (later, earlier) = (*g, *f);
                match (generic(earlier), generic(later)) {
                    (None, None) => if shapes(earlier) == shapes(later) {
                        return rt(later.line, format!("`{}` is defined twice for {} (first at {}): two meanings of one operator for the same types", later.name, shapes(later).join(", "), line_of(earlier.line)));
                    },
                    (Some(bound), None) | (None, Some(bound)) => {
                        let (gen, spec) = if generic(earlier).is_some() { (earlier, later) } else { (later, earlier) };
                        let shape = shapes(spec).first().cloned().unwrap_or_default();
                        let covered = match &bound { None => true, Some(i) => self.satisfies(&shape, i, gen.line)?.is_ok() };
                        let tname = spec.params.first().and_then(|p| p.ty.as_ref()).map(ty_name).unwrap_or_default();
                        if covered { return rt(later.line, format!("`{}` for {tname} ({}) and the generic `{}` ({}) both cover {tname}: two meanings of one operator for the same types; narrow the generic one's bound so {tname} does not keep it, or drop one", gen.name, line_of(spec.line), gen.name, line_of(gen.line))); }
                    }
                    // the oracle's reading: two generic ones may always meet on some type
                    (Some(_), Some(_)) => return rt(later.line, format!("two generic `{}` (lines {} and {}) could both cover one type", later.name, earlier.line, later.line)),
                }
            }
        }
        for d in &self.prog.structs {
            let k = format!("struct {}", d.name);
            if let Some(l) = seen.insert(k.clone(), d.line) { return rt(d.line, format!("struct `{}` is defined twice (first at {})", d.name, line_of(l))); }
        }
        for e in &self.prog.enums {
            let k = format!("struct {}", e.name);   // structs and enums share the type namespace
            if let Some(l) = seen.insert(k.clone(), e.line) { return rt(e.line, format!("type `{}` is defined twice (first at {})", e.name, line_of(l))); }
        }
        for m in &self.prog.modules {
            let k = format!("module {}", m.path.join("."));
            if let Some(l) = seen.insert(k.clone(), m.line) { return rt(m.line, format!("module `{}` is defined twice (first at {}) — a file and a block cannot both be it", m.path.join("."), line_of(l))); }
            // One name, one thing, per Neuron (decided 2026-09-21): a module and a type cannot share a name.
            let path = m.path.join(".");
            let clash = self.prog.structs.iter().map(|d| (&d.name, d.line, "struct")).chain(self.prog.enums.iter().map(|e| (&e.name, e.line, "enum"))).chain(self.prog.interfaces.iter().map(|i| (&i.name, i.line, "interface"))).find(|(n, _, _)| **n == path);
            if let Some((_, l, what)) = clash {
                let where_ = if m.line == 0 { format!("the file `{path}.nrv` and a {what} at {}", line_of(l)) } else { format!("a module block at {} and a {what} at {}", line_of(m.line), line_of(l)) };
                return rt(l, format!("`{path}` is both a module and a {what} ({where_}); one name is one thing in a Neuron — rename one"));
            }
        }
        Ok(())
    }

    /// design/ffi.md: only C-shaped types cross to C; a struct that crosses is `layout.C`.
    fn c_shaped(&self, ty: &Ty) -> Result<(), String> {
        match ty {
            Ty::Int(_) | Ty::Uint(_) | Ty::Bin(_) | Ty::Bool => Ok(()),
            Ty::Ptr(inner, _) => self.c_shaped(inner),
            Ty::Struct(n) => match self.prog.structs.iter().find(|d| &d.name == n) {
                Some(d) if d.layout_c => Ok(()),
                Some(_) => Err(format!("struct `{n}` is not `layout.C`; its layout is Cortex's, not C's")),
                None => match self.prog.enums.iter().find(|e| &e.name == n) {
                    Some(e) if e.c_values.is_some() => Ok(()),
                    Some(_) => Err(format!("enum `{n}` is not `layout.C`; a C enum is `layout.C enum {n} {{ a = 0 }}`")),
                    None => Err(format!("no struct or enum named `{n}`")),
                },
            },
            Ty::Var(_) | Ty::Inst(..) => Err("a generic type cannot cross to C".into()),
            Ty::Ranged(..) => Err("a range type cannot cross to C; C has no ranges — use the plain type".into()),
            // a callback (design/ffi.md, 2026-09-21): C-shaped through and through, by value, no failures
            Ty::Fn(ps, ret, fails) => {
                if !fails.is_empty() { return Err("a callback cannot fail; C has no failures".into()); }
                for (k, t) in ps {
                    if *k != ParamKind::Owned { return Err("a callback's parameters are by value; C passes pointers for the rest".into()); }
                    // C calls a callback, so a bad number has no `otherwise` to fail to: its
                    // parameters are plain, converted inside (owner, 2026-09-27)
                    if let Ty::Struct(n) = t {
                        let checked = self.prog.enums.iter().any(|e| &e.name == n && e.c_values.is_some())
                            || self.prog.structs.iter().any(|d| &d.name == n && d.fields.iter().any(|f| matches!(f.ty, Ty::Ranged(..))));
                        if checked { return Err(format!("a callback's parameter cannot be `{n}`: C calls it, so a bad number would have nowhere to fail to — take the plain number and convert it inside: `to.{n}['n'] otherwise …`")); }
                    }
                    self.c_shaped(t)?;
                }
                if let Some(r) = ret { self.c_shaped(r)?; }
                Ok(())
            }
            other => Err(format!("{} is not C-shaped (numbers, bool, ptr, layout.C structs are)", ty_name(other))),
        }
    }

    fn check_boundary(&self) -> Result<(), RuntimeError> {
        for d in self.prog.structs.iter().filter(|d| d.layout_c) {
            for f in &d.fields {
                // a field may be a range type (owner, 2026-09-27: a): C sees the plain number, and
                // a struct that comes from C is checked at the door, `out_of_range`
                let laid = match &f.ty { Ty::Ranged(b, _, _) => &**b, t => t };
                if let Err(m) = self.c_shaped(laid) { return rt(d.line, format!("`layout.C struct {}`: field `{}` — {m}", d.name, f.name)); }
            }
        }
        // `extern let` (decided 2026-09-21): C-shaped, like an extern parameter
        for x in &self.prog.externs {
            if let Err(m) = self.c_shaped(&x.ty) { return rt(x.line, format!("`extern.C '{}'`: {m}", x.name)); }
            // a C variable is read by its name, and a read never fails, so it holds C's plain
            // number; convert it with `to.Key[…] otherwise …` (owner, 2026-09-28: b)
            if let Some(n) = self.c_checked(&x.ty) { return rt(x.line, format!("`extern.C '{}'`: a C variable cannot be `{n}` — reading a variable never fails, so it holds C's plain number: declare it `int32` and convert, `to.{n}['{}'] otherwise …`", x.name, x.name)); }
        }
        for f in self.prog.funcs.iter().filter(|f| f.is_extern) {
            for p in &f.params {
                match &p.ty {
                    None if !p.variadic => return rt(f.line, format!("`extern.C fn {}`: parameter '{}' needs a type; C has no inference", f.name, p.name)),
                    Some(t) => if let Err(m) = self.c_shaped(t) { return rt(f.line, format!("`extern.C fn {}`: parameter '{}' — {m}", f.name, p.name)); },
                    None => {}
                }
                if p.kind != ParamKind::Owned { return rt(f.line, format!("`extern.C fn {}`: C receives values and pointers, not `lent`; a lend is written at the call, against a `ptr` parameter", f.name)); }
            }
            // A ranged return (`-> uint64[0, 1_000_000]`) is a contract checked at the door: the call
            // fails `out_of_range` when C returns outside it (decided 2026-09-22). The base must be C-shaped.
            if let Some(t) = &f.ret {
                let base = match t { Ty::Ranged(b, _, _) => &**b, t => t };
                if let Err(m) = self.c_shaped(base) { return rt(f.line, format!("`extern.C fn {}`: return type — {m}", f.name)); }
            }
        }
        Ok(())
    }

    /// `fail name;` only names a reason its function lists; MAIN cannot fail (decided 2026-09-19).
    fn check_fails(&self) -> Result<(), RuntimeError> {
        fn walk(stmts: &[Stmt], fails: &[FailCase], who: &str) -> Result<(), RuntimeError> {
            for s in stmts {
                match s {
                    Stmt::Fail { name, payload, line } => {
                        if who == "MAIN" { return rt(*line, "MAIN cannot fail; it exits (`cord::exit[…]`) or handles the failure"); }
                        match fails.iter().find(|f| &f.name == name) {
                            None if fails.is_empty() => return rt(*line, format!("`{who}` cannot fail: its signature has no `fail[…]`; add `-> … or fail[{name}]`")),
                            None => return rt(*line, format!("`{who}` does not list `{name}` in its `fail[…]` (it lists {})", fails.iter().map(|f| f.name.as_str()).collect::<Vec<_>>().join(", "))),
                            Some(f) => {
                                if f.ty.is_some() && payload.is_none() { return rt(*line, format!("`{name}` carries a {}; `fail {name}[…]` needs it", ty_name(f.ty.as_ref().unwrap()))); }
                                if f.ty.is_none() && payload.is_some() { return rt(*line, format!("`{name}` carries nothing; write `fail {name};`")); }
                            }
                        }
                    }
                    Stmt::If { arms, otherwise, .. } => { for (_, b) in arms { walk(b, fails, who)?; } if let Some(b) = otherwise { walk(b, fails, who)?; } }
                    Stmt::Loop { body, .. } | Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::Range { body, .. } | Stmt::Block { body, .. } => walk(body, fails, who)?,
                    Stmt::Tier { body, .. } => walk(std::slice::from_ref(body), fails, who)?,
                    _ => {}
                }
            }
            Ok(())
        }
        for f in &self.prog.funcs { walk(&f.body, &f.fails, &f.name)?; }
        if let Some(m) = &self.prog.main { walk(m, &[], "MAIN")?; }
        Ok(())
    }

    /// Visibility (decided 2026-09-19): an item declared in module `owner` with `vis`
    /// is reachable from module `from` if it is `pub` / `pub.neuron` (one Neuron here, so
    /// both count) or `from` is `owner` or a descendant of it. Every module on the
    /// way is checked the same way.
    fn reachable(&self, from: &[String], owner: &[String], vis: Vis) -> bool {
        // across a Neuron's edge only `pub` passes: `pub.neuron` stays home (limbs.md)
        if limb_of(from) != limb_of(owner) { return vis == Vis::Pub; }
        vis != Vis::Private || from.starts_with(owner)
    }

    fn check_path(&self, from: &[String], module: &[String], line: usize) -> Result<(), RuntimeError> {
        for i in 1..=module.len() {
            let path = &module[..i];
            let Some(m) = self.prog.modules.iter().find(|m| m.path == path) else {
                return rt(line, format!("no module `{}` in {}", path.iter().map(|p| p.trim_start_matches("::")).collect::<Vec<_>>().join("."), if limb_of(path).is_empty() { "this Neuron".to_string() } else { format!("the Neuron `{}`", &limb_of(path)[2..]) }));
            };
            if !self.reachable(from, &module[..i - 1], m.vis) {
                return rt(line, format!("module `{}` is private to `{}`", path.join("."), root_or(&module[..i - 1])));
            }
        }
        Ok(())
    }

    fn declare(&mut self, name: &str, slot: Slot, line: usize) -> Result<(), RuntimeError> {
        self.declare_shadow(name, slot, false, line)
    }

    /// Decided 2026-09-14: a visible name may only be redeclared with `$:=` /
    /// `$=`, and never if the visible one is `immut`.
    fn declare_shadow(&mut self, name: &str, mut slot: Slot, shadow: bool, line: usize) -> Result<(), RuntimeError> {
        if matches!(slot.kind, SlotKind::Lent | SlotKind::LentMut) { slot.since = self.write_count; slot.deps = std::mem::take(&mut self.pending_deps); }
        match (self.lookup(name), shadow) {
            (Some(_), false) => return rt(line, format!("'{name}' is already declared in an enclosing scope; to shadow it deliberately write `shadow '{name}' := …;` (owner, 2026-10-08)")),
            (None, true) => return rt(line, format!("`shadow` declares over an existing '{name}', but none is visible here")),
            (Some(existing), true) if existing.kind == SlotKind::Const => return rt(line, format!("'{name}' is a const and cannot be shadowed")),
            // `immut` only forbids `change` now; `!$` forbids shadowing (owner, 2026-09-25)
            (Some(existing), true) if existing.sealed => return rt(line, format!("'{name}' cannot be shadowed: it is `no_shadow` (or a loop's own counter)")),
            _ => {}
        }
        self.scopes.last_mut().unwrap().insert(name.to_string(), slot);
        Ok(())
    }

    // ---- statements -------------------------------------------------------

    fn block(&mut self, stmts: &[Stmt]) -> Result<Flow, RuntimeError> {
        self.push();
        let depth = self.scopes.len();
        let r = self.stmts(stmts);
        // structured concurrency: the block waits for every task it started (decided 2026-09-22)
        let r = match r { Ok(flow) => self.drain_tasks(depth).map(|_| flow), e => e };
        self.pop();
        r
    }

    /// Run every pending task started at `depth` or deeper, in order (detached tasks are
    /// nobody's to wait for; they run only on demand).
    fn drain_tasks(&mut self, depth: usize) -> Result<(), RuntimeError> {
        loop {
            let Some(i) = self.tasks.iter().position(|t| !t.detached && t.depth >= depth) else { return Ok(()) };
            let task = self.tasks.remove(i);
            self.run_task(task)?;
        }
    }

    /// Run one pending task — any, in order — because something is waiting. False if none.
    fn run_any_task(&mut self) -> Result<bool, RuntimeError> {
        if self.tasks.is_empty() { return Ok(false); }
        let task = self.tasks.remove(0);
        self.run_task(task)?;
        Ok(true)
    }

    fn run_task(&mut self, task: Task) -> Result<(), RuntimeError> {
        let saved = std::mem::replace(&mut self.scopes, vec![HashMap::new()]);
        let saved_module = std::mem::replace(&mut self.module, task.module.clone());
        let saved_fails = std::mem::replace(&mut self.fails, task.fails.clone());
        let saved_stmt = self.call_is_stmt;
        // A task's body has its own scope stack, so the tasks waiting in the *outer* one
        // are not its to run: their handlers belong to the scope their `concur` was
        // written in, and running them here would look that scope up in this stack.
        // (Found by the compiler session, 2026-09-23: two failing tasks in one block.)
        let saved_tasks = std::mem::take(&mut self.tasks);
        let flow = (|| {
            for (n, cell, mutable) in &task.captures {
                self.declare(n, Slot { cell: cell.clone(), ty: None, kind: if *mutable { SlotKind::Owned } else { SlotKind::Lent }, alive: true, immut: false, sized: false, sealed: false , since: 0, deps: Vec::new() }, task.line)?;
            }
            self.block(&task.body)
        })();
        self.scopes = saved; self.module = saved_module; self.fails = saved_fails; self.call_is_stmt = saved_stmt;
        let mut leftover = std::mem::replace(&mut self.tasks, saved_tasks);   // detached ones the task started
        self.tasks.append(&mut leftover);
        match flow {
            Ok(Flow::Fail(name, payload)) => {
                // the task's failure lands where its `concur` was written (owner, 2026-09-22)
                let Some(h) = task.handler.as_ref() else { return rt(task.line, format!("the task failed (`{name}`) and its `concur` has no `otherwise`")) };
                self.call_is_stmt = true;
                let r = self.handle_failure(h, task.line, "concur", name, payload, false);
                self.call_is_stmt = saved_stmt;
                match r { Ok(_) => Ok(()), Err(e) => Err(e) }
            }
            Ok(Flow::Return(_)) => rt(task.line, "`return` inside a task: a task is a block, not a function"),
            Ok(Flow::Break) | Ok(Flow::Continue) => rt(task.line, "`break`/`continue` inside a task reach no loop"),
            Ok(Flow::Give(_)) => rt(task.line, "`giveS` outside a case"),
            Ok(Flow::Normal) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// The failure names a task body can produce: its own `fail` statements and what it passes on.
    fn task_fails(&self, body: &[Stmt]) -> Vec<FailCase> {
        fn walk(stmts: &[Stmt], out: &mut Vec<FailCase>, prog: &Program) {
            for s in stmts {
                match s {
                    Stmt::Fail { name, payload, .. } => if !out.iter().any(|f| &f.name == name) { out.push(FailCase { name: name.clone(), ty: if payload.is_some() { Some(Ty::Bool) } else { None } }); },
                    Stmt::Call { call, .. } => if matches!(call.handler, Some(Handler::Propagate)) {
                        if let Some(f) = prog.funcs.iter().find(|f| f.name == *call.path.last().unwrap()) { for fc in &f.fails { if !out.iter().any(|x| x.name == fc.name) { out.push(fc.clone()); } } }
                    },
                    Stmt::Decl { value, .. } | Stmt::Assign { value, .. } => {
                        for v in &value.values { if let crate::ast::Value::Expr(Expr::Call(c)) = v { if matches!(c.handler, Some(Handler::Propagate)) {
                            if let Some(f) = prog.funcs.iter().find(|f| f.name == *c.path.last().unwrap()) { for fc in &f.fails { if !out.iter().any(|x| x.name == fc.name) { out.push(fc.clone()); } } }
                        } } }
                    }
                    Stmt::If { arms, otherwise, .. } => { for (_, b) in arms { walk(b, out, prog); } if let Some(b) = otherwise { walk(b, out, prog); } }
                    Stmt::Loop { body, .. } | Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::Range { body, .. } | Stmt::Block { body, .. } => walk(body, out, prog),
                    Stmt::Tier { body, .. } => walk(std::slice::from_ref(body), out, prog),
                    _ => {}
                }
            }
        }
        let mut out = Vec::new();
        walk(body, &mut out, self.prog);
        out
    }

    fn stmts(&mut self, stmts: &[Stmt]) -> Result<Flow, RuntimeError> {
        for s in stmts {
            match self.stmt(s)? {
                Flow::Normal => {}
                other => return Ok(other),
            }
        }
        Ok(Flow::Normal)
    }

    fn stmt(&mut self, s: &Stmt) -> Result<Flow, RuntimeError> {
        self.pending_deps.clear();
        let r = match self.stmt_inner(s) {
            Err(RuntimeError { leave: Some(f), .. }) => Ok(f),
            other => other,
        };
        // write mutable field lends back (a statement is the unit of a lend here)
        let borrows = std::mem::take(&mut self.field_borrows);
        for (owner, field, tmp) in borrows {
            if let Value::Struct(_, fields) = &mut *owner.borrow_mut() {
                if let Some((_, slot)) = fields.iter_mut().find(|(k, _)| k == &field) { *slot = tmp.borrow().clone(); }
            }
        }
        r
    }

    fn stmt_inner(&mut self, s: &Stmt) -> Result<Flow, RuntimeError> {
        match s {
            // a test's assertions (owner, 2026-09-29)
            Stmt::Must { cond, line } => {
                let (v, _) = self.eval(cond)?;
                match v {
                    Value::Bool(true) => Ok(Flow::Normal),
                    Value::Bool(false) => {
                        // a comparison shows both sides (worked out again for the message: the oracle's reading)
                        if let Expr::Binary(op, l, r, _) = cond {
                            if matches!(op.as_str(), "==" | "!==" | "<" | ">" | "</==" | ">/==") {
                                let (a, _) = self.eval(l)?;
                                let (b, _) = self.eval(r)?;
                                return rt(*line, format!("`must` does not hold: the left side is {}, and it must be `{}` {}", a.show_useful(), crate::parser::op_shown(op), b.show_useful()));
                            }
                        }
                        rt(*line, "`must` does not hold")
                    }
                    other => rt(*line, format!("`must` takes a condition, a bool; this is {}", other.kind())),
                }
            }
            Stmt::MustFail { expr, reason, line } => {
                self.must_fail_depth += 1;
                let r = self.eval(expr);
                self.must_fail_depth -= 1;
                match r {
                    Ok(_) => rt(*line, "`must.fail`: it did not fail"),
                    Err(RuntimeError { leave: Some(Flow::Fail(got, _)), .. }) => match reason {
                        Some(want) if *want != got => rt(*line, format!("`must.fail.{want}`: it failed with `{got}`")),
                        _ => Ok(Flow::Normal),
                    },
                    Err(e) => Err(e),
                }
            }
            Stmt::MustStop { expr, line } => match self.eval(expr) {
                Ok(_) => rt(*line, "`must.stop`: no check fired"),
                Err(e) if e.site => Ok(Flow::Normal),
                Err(e) => Err(e),
            },
            Stmt::Decl { name, mutability, ty, value, shadow, sealed, line } => {
                let (v, prov) = if value.values.is_empty() {
                    match ty {
                        Some(Ty::List(_)) => (Value::List(Vec::new()), Prov::Fresh),
                        Some(Ty::Array(_)) => return rt(*line, format!("an empty `array` '{name}' could never be filled; make it a `list`")),
                        Some(t) => return rt(*line, format!("`[]` is an empty list, but '{name}' is {}", ty_name(t))),
                        None => return rt(*line, format!("`[]` with `:=` says nothing about what '{name}' is; write `list.empty`")),
                    }
                } else {
                    let saved = std::mem::replace(&mut self.expected, ty.clone());
                    let r = self.bracket(value, *line);
                    self.expected = saved;
                    r?
                };
                let immut = *mutability == Some(Mutability::Immut);
                let slot = match prov {
                    Prov::Bare(n) => return rt(*line, format!("declaring '{name}' from bare '{n}', which owns a {}: say `give '{n}'`, `copy '{n}'`, or `lend '{n}'`", v.kind())),
                    Prov::Borrowed(cell, is_mut) => {
                        if ty.is_some() { return rt(*line, format!("'{name}' would hold a lend; a lent variable takes no explicit type here (use `:=`)")); }
                        Slot { cell, ty: None, kind: if is_mut { SlotKind::LentMut } else { SlotKind::Lent }, alive: true, immut, sized: false, sealed: false , since: 0, deps: Vec::new() }
                    }
                    _ => {
                        let v = self.fit(v, prov, ty.as_ref(), *line)?;
                        // `:=` takes the value's own type where the oracle knows it (only the length
                        // operations so far: `uint64`, decided 2026-09-21) — so a later assignment is checked.
                        let ty = ty.clone().or_else(|| self.known_ty(value));
                        // its length is known here when it came from a literal list, `list.empty`, or a bound
                        let sized = Self::sized_from(value, ty.as_ref());
                        Slot { cell: Rc::new(RefCell::new(v)), ty, kind: SlotKind::Owned, alive: true, immut, sized, sealed: *sealed , since: 0, deps: Vec::new() }
                    }
                };
                self.declare_shadow(name, slot, *shadow, *line)?;
                Ok(Flow::Normal)
            }
            Stmt::Assign { name, value, line } => {
                let (kind, ty, immut) = match self.lookup(name) {
                    Some(s) => (s.kind, s.ty.clone(), s.immut),
                    None => return rt(*line, format!("'{name}' is not declared")),
                };
                if kind == SlotKind::Const { return rt(*line, format!("'{name}' is a const and cannot be assigned to")); }
                if kind == SlotKind::Extern { return rt(*line, format!("'{name}' is `extern let`, read-only; declare it `extern let mut` to write C's variable")); }
                if kind == SlotKind::ExternMut && self.nocheck_depth == 0 && self.marks_needed() { return rt(*line, format!("'{name}' is extern: writing C's variable is a no_check site: `no_check '{name}' = …;`")); }
                if immut { return rt(*line, format!("'{name}' cannot be changed: it is `immut` (or a loop's counter or element, which belong to the loop)")); }
                if kind == SlotKind::Lent { return rt(*line, format!("'{name}' is lent (shared); it cannot be assigned to — a `lent.mut` could")); }
                self.stale(name, self.lookup(name).unwrap(), *line)?;
                let (v, prov) = {
                    let saved = std::mem::replace(&mut self.expected, ty.clone());
                    let r = self.bracket(value, *line);
                    self.expected = saved;
                    r?
                };
                match prov {
                    Prov::Bare(n) => return rt(*line, format!("assigning bare '{n}' to '{name}': '{n}' owns a {} — say `give '{n}'`, `copy '{n}'`, or `lend '{n}'`", v.kind())),
                    Prov::Borrowed(..) => return rt(*line, format!("assigning a lend to '{name}': a lend is temporary and cannot be stored in an owned variable (`copy` it or `give` it)")),
                    _ => {}
                }
                let v = self.fit(v, prov, ty.as_ref(), *line)?;
                let sized = Self::sized_from(value, ty.as_ref());
                if kind == SlotKind::ExternMut {
                    // C's variable: the cell lives beside the consts
                    *self.consts[&const_key(&self.module, name)].cell.borrow_mut() = v;
                    return Ok(Flow::Normal);
                }
                self.wrote(name, *line, "changed");
                let slot = self.lookup_mut(name).unwrap();
                slot.alive = true; // assigning to a given-away variable revives it
                slot.sized = sized;
                *slot.cell.borrow_mut() = v;
                Ok(Flow::Normal)
            }
            Stmt::AssignElem { name, index, value, line } => {
                let (kind, ty, immut, cell) = match self.lookup(name) {
                    Some(s) => (s.kind, s.ty.clone(), s.immut, s.cell.clone()),
                    None => return rt(*line, format!("'{name}' is not declared")),
                };
                if immut { return rt(*line, format!("'{name}' is `immut`; its elements cannot be assigned to")); }
                if kind == SlotKind::Lent { return rt(*line, format!("'{name}' is lent (shared); its elements cannot be assigned to — a `lent.mut` could")); }
                self.stale(name, self.lookup(name).unwrap(), *line)?;
                self.wrote(name, *line, "changed");
                let (i, _) = self.eval(index)?;
                let i = match i { Value::Int(i) => i, other => return rt(*line, format!("a position is an integer, got {}", other.kind())) };
                let (v, prov) = self.bracket(value, *line)?;
                match prov {
                    Prov::Bare(n) => return rt(*line, format!("storing bare '{n}' into '{name}': say `give '{n}'` or `copy '{n}'`")),
                    Prov::Borrowed(..) => return rt(*line, "a list owns its elements; store a `copy` or `give`, not a lend"),
                    _ => {}
                }
                let elem_ty = match &ty {
                    Some(Ty::List(t)) | Some(Ty::Array(t)) => Some((**t).clone()),
                    // one lane of a vector (owner, 2026-09-29: a)
                    Some(Ty::Simd(t, _)) => Some((**t).clone()),
                    Some(Ty::Ranged(b, _, _)) => match &**b { Ty::List(t) | Ty::Array(t) => Some((**t).clone()), _ => None },
                    // An untyped list still has an element family: what is in it. Storing a bin
                    // into a list of integers is the cross-family store `types.md` refuses
                    // (found by the compiler session, 2026-09-22 — the oracle had this hole).
                    None => match &*cell.borrow() {
                        Value::List(items) => items.first().map(|f| match f {
                            Value::Int(_) => Some(Ty::Int(IntWidth::W64)), Value::Bin(_) => Some(Ty::Bin(64)), _ => None,
                        }).flatten(),
                        _ => None,
                    },
                    _ => None,
                };
                let v = self.fit(v, prov, elem_ty.as_ref(), *line)?;
                let mut cur = cell.borrow_mut();
                let items = match &mut *cur { Value::List(items) | Value::Simd(items) => items, other => return rt(*line, format!("'{name}' is {}, not a list, array or vector", other.kind())) };
                if i < 1 || i as usize > items.len() {
                    return site(*line, format!("position {i} is out of range for '{name}', which has {} element(s) (1 to {})", items.len(), items.len()));
                }
                items[(i - 1) as usize] = v;
                Ok(Flow::Normal)
            }
            Stmt::AssignField { name, field, value, line } => {
                let (kind, immut, cell) = match self.lookup(name) {
                    Some(s) => (s.kind, s.immut, s.cell.clone()),
                    None => return rt(*line, format!("'{name}' is not declared")),
                };
                if !self.lookup(name).unwrap().alive { return rt(*line, format!("'{name}' was given away and no longer owns anything")); }
                if immut { return rt(*line, format!("'{name}' is `immut`; its fields cannot be assigned to")); }
                if kind == SlotKind::Lent { return rt(*line, format!("'{name}' is lent (shared); its fields cannot be assigned to — a `lent.mut` could")); }
                self.stale(name, self.lookup(name).unwrap(), *line)?;
                self.wrote(name, *line, "changed");
                let sname = match &*cell.borrow() { Value::Struct(n, _) => n.clone(), other => return rt(*line, format!("'{name}' is {}, not a struct; it has no field `{field}`", other.kind())) };
                let def = self.prog.structs.iter().find(|d| d.name == sname).cloned().unwrap();
                let fdef = match def.fields.iter().find(|f| &f.name == field) { Some(f) => f.clone(), None => return rt(*line, format!("{sname} has no field `{field}`")) };
                if fdef.immut { return rt(*line, format!("'{field}' is an `immut` field of {sname}; it cannot be assigned to")); }
                let (v, prov) = self.bracket(value, *line)?;
                match prov {
                    Prov::Bare(n) => return rt(*line, format!("storing bare '{n}' into '{name}'.{field}: say `give '{n}'` or `copy '{n}'`")),
                    Prov::Borrowed(..) => return rt(*line, "a struct owns its fields; store a `copy` or `give`, not a lend"),
                    _ => {}
                }
                let v = self.fit(v, prov, Some(&fdef.ty), *line)?;
                if let Value::Struct(_, fields) = &mut *cell.borrow_mut() {
                    for (k, slot) in fields.iter_mut() { if k == field { *slot = v; break; } }
                }
                Ok(Flow::Normal)
            }
            Stmt::If { arms, otherwise, line } => {
                for (cond, body) in arms {
                    if self.cond(cond, *line)? { return self.block(body); }
                }
                if let Some(b) = otherwise { return self.block(b); }
                Ok(Flow::Normal)
            }
            Stmt::Loop { body, .. } => loop {
                match self.block(body)? {
                    Flow::Break => return Ok(Flow::Normal),
                    f @ (Flow::Return(_) | Flow::Fail(..) | Flow::Give(_)) => return Ok(f),
                    Flow::Normal | Flow::Continue => {}
                }
            },
            Stmt::While { cond, body, line } => {
                while self.cond(cond, *line)? {
                    match self.block(body)? {
                        Flow::Break => break,
                        f @ (Flow::Return(_) | Flow::Fail(..) | Flow::Give(_)) => return Ok(f),
                        Flow::Normal | Flow::Continue => {}
                    }
                }
                Ok(Flow::Normal)
            }
            Stmt::For { name, index, perm, over, body, line, .. } => {
                let (items, kind) = self.for_source(over, *line)?;
                // a loop over a lend borrows the list while it runs: changing the list inside is refused
                // (check claude's borrow findings, 2026-09-29)
                let walked = match over.values.first() {
                    Some(crate::ast::Value::Expr(Expr::Owned(Verb::Lend | Verb::LendMut, n, _))) => self.lookup(n).map(|s| (n.clone(), Rc::as_ptr(&s.cell) as *const () as usize)),
                    _ => None,
                };
                let started = self.write_count;
                // over `lend.mut 'xs'`, the element is the list's own: `change 'x'` writes into 'xs'
                // (owner, 2026-09-30: "the program looks legal to me. `mut`'s already there")
                let mut_list = match over.values.first() {
                    Some(crate::ast::Value::Expr(Expr::Owned(Verb::LendMut, n, _))) => self.lookup(n).map(|s| s.cell.clone()),
                    _ => None,
                };
                // the item has the list's element type (check claude, 2026-09-29: `'x' +~ 10` had no width)
                let elem_ty = match over.values.first() {
                    Some(crate::ast::Value::Expr(Expr::Owned(_, n, _) | Expr::Var(n, _))) => match self.lookup(n).and_then(|s| s.ty.clone()) {
                        Some(Ty::List(t)) | Some(Ty::Array(t)) => Some(*t),
                        Some(Ty::Ranged(b, _, _)) => match *b { Ty::List(t) | Ty::Array(t) => Some(*t), _ => None },
                        _ => None,
                    },
                    _ => None,
                };
                // perm: the variable outlives the loop, holding whatever the loop last gave it.
                let cell = Rc::new(RefCell::new(Value::Nothing));
                let pos_cell = Rc::new(RefCell::new(Value::Int(0)));
                if let (Some(list), Some(crate::ast::Value::Expr(Expr::Owned(_, n, _)))) = (&mut_list, over.values.first()) {
                    self.mut_walks.push(MutWalk { list: list.clone(), elem: cell.clone(), name: n.clone(), elem_name: name.clone(), pos: std::cell::Cell::new(0), read_at: std::cell::Cell::new(None) });
                }
                let mut ran = false;
                let trips = (|| -> Result<Option<Flow>, RuntimeError> {
                    for (k, item) in items.into_iter().enumerate() {
                        ran = true;
                        *cell.borrow_mut() = item;
                        *pos_cell.borrow_mut() = Value::Int(k as i128 + 1);
                        if mut_list.is_some() { if let Some(w) = self.mut_walks.last() { w.pos.set(k); w.read_at.set(None); } }
                        self.push();
                        let r = match index {
                            Some(ix) => self.declare(ix, Slot { cell: pos_cell.clone(), ty: Some(Ty::Uint(crate::ast::IntWidth::W64)), kind: SlotKind::Owned, alive: true, immut: true, sized: false, sealed: true , since: 0, deps: Vec::new() }, *line),
                            None => Ok(()),
                        }
                        .and_then(|_| self.declare(name, Slot { cell: cell.clone(), ty: elem_ty.clone(), kind, alive: true, immut: mut_list.is_none(), sized: false, sealed: true , since: 0, deps: Vec::new() }, *line))
                        .and_then(|_| self.stmts(body));
                        self.pop();
                        if let Some(list) = &mut_list {
                            if let Value::List(xs) = &mut *list.borrow_mut() { if let Some(slot) = xs.get_mut(k) { *slot = cell.borrow().clone(); } }
                        }
                        let r = r?;
                        if let Some((n, key)) = &walked {
                            if let Some((w, at, what, _)) = self.writes.get(key) {
                                // the error is at the change (owner, 2026-09-29: a)
                                if *w > started { return rt(*at, format!("'{n}' is {what} here while the loop on {} is going through it: a loop over `lend '{n}'` borrows it until the loop ends", line_of(line))); }
                            }
                        }
                        match r {
                            Flow::Break => break,
                            f @ (Flow::Return(_) | Flow::Fail(..) | Flow::Give(_)) => return Ok(Some(f)),
                            Flow::Normal | Flow::Continue => {}
                        }
                    }
                    Ok(None)
                })();
                if mut_list.is_some() { self.mut_walks.pop(); }
                if let Some(f) = trips? { return Ok(f); }
                if *perm {
                    if !ran { return rt(*line, format!("`perm` '{name}' would outlive a loop over an empty list, but was never given a value")); }
                    self.declare(name, Slot { cell, ty: None, kind: SlotKind::Owned, alive: true, immut: false, sized: false, sealed: false , since: 0, deps: Vec::new() }, *line)?;
                    if let Some(ix) = index { self.declare(ix, Slot { cell: pos_cell, ty: Some(Ty::Uint(crate::ast::IntWidth::W64)), kind: SlotKind::Owned, alive: true, immut: false, sized: false, sealed: false , since: 0, deps: Vec::new() }, *line)?; }
                }
                Ok(Flow::Normal)
            }
            Stmt::Range { name, perm, ty, bounds, body, line, .. } => {
                let mut nums = Vec::new();
                for (k, v) in bounds.values.iter().enumerate() {
                    let (val, prov) = self.value(v)?;
                    // The step is its own signed number, not a value the counter takes
                    // (owner, 2026-09-23): `for 'i': uint64 in range[3, 1, -1]` is legal.
                    let val = if k < 2 { self.fit(val, prov, ty.as_ref(), *line)? } else { val };
                    match val {
                        Value::Int(i) => nums.push(i),
                        other => return rt(*line, format!("a range bound must be an integer, got {}", other.kind())),
                    }
                }
                let (start, stop) = (nums[0], nums[1]);
                let step = if nums.len() == 3 { nums[2] } else { 1 };
                if step == 0 { return rt(*line, "a range step of 0 would never end"); }
                // Inclusive stop (decided 2026-09-17): 'i' reaches the stop value.
                let cell = Rc::new(RefCell::new(Value::Int(start)));
                let mut i = start;
                let mut broke = false;
                while (step > 0 && i <= stop) || (step < 0 && i >= stop) {
                    if let Some(t) = ty { self.check_type(&Value::Int(i), t, *line)?; }
                    *cell.borrow_mut() = Value::Int(i);
                    self.push();
                    let r = self.declare(name, Slot { cell: cell.clone(), ty: ty.clone(), kind: SlotKind::Owned, alive: true, immut: true, sized: false, sealed: true , since: 0, deps: Vec::new() }, *line)
                        .and_then(|_| self.stmts(body));
                    self.pop();
                    match r? {
                        Flow::Break => { broke = true; break; }
                        f @ (Flow::Return(_) | Flow::Fail(..) | Flow::Give(_)) => return Ok(f),
                        Flow::Normal | Flow::Continue => {}
                    }
                    match i.checked_add(step) { Some(n) => i = n, None => break }
                }
                let _ = broke;
                if *perm {
                    // 'i' keeps whatever the loop last gave it: the last value the body ran
                    // with, or the start if the body never ran.
                    self.declare(name, Slot { cell, ty: ty.clone(), kind: SlotKind::Owned, alive: true, immut: false, sized: false, sealed: false , since: 0, deps: Vec::new() }, *line)?;
                }
                Ok(Flow::Normal)
            }
            Stmt::Break { .. } => Ok(Flow::Break),
            Stmt::Continue { .. } => Ok(Flow::Continue),
            Stmt::Return { value, line } => {
                self.ret_line = *line;
                let v = match value {
                    None => Value::Nothing,
                    Some(b) => {
                        // a `return` lands in the function's return type, never in a declaration it sits
                        // inside (a `match` arm's; check claude, seed 6868)
                        let ctx = self.cur_ret.clone();
                        let saved = std::mem::replace(&mut self.expected, ctx);
                        let r = self.bracket(b, *line);
                        self.expected = saved;
                        let (v, prov) = r?;
                        let (v, prov) = match &self.cur_ret { Some(t) => raw_land(v, prov, t, *line)?, None => (v, prov) };
                        self.ret_kind = self.last_kind.take();
                        self.ret_line = *line;
                        match prov {
                            Prov::Bare(n) => return rt(*line, format!("returning bare '{n}', which owns a {}: say `give '{n}'` (or `copy`)", v.kind())),
                            // a borrow handed back (owner, 2026-09-29): checked against `-> lent@x` by the caller
                            Prov::Borrowed(cell, m) => { self.ret_borrow = Some((cell, m)); v }
                            _ => v,
                        }
                    }
                };
                Ok(Flow::Return(v))
            }
            Stmt::Call { call, .. } => {
                self.call_is_stmt = true;
                let r = self.call(call);
                self.call_is_stmt = false;
                r?;
                Ok(Flow::Normal)
            }
            Stmt::Concur { detached, caps, body, handler, line } => {
                // captures, each with its verb — as an anonymous fn's (design/functions-as-values.md)
                let mut captures = Vec::new();
                for (verb, v) in caps {
                    let (kind, alive, cell, val) = match self.lookup(v) { Some(s) => (s.kind, s.alive, s.cell.clone(), s.cell.borrow().clone()), None => return rt(*line, format!("'{v}' is not declared")) };
                    if !alive { return rt(*line, format!("'{v}' was already given away")); }
                    match verb {
                        Verb::Lend | Verb::LendMut => {
                            if *detached { return rt(*line, format!("`concur.detached` may not lend '{v}': no scope protects a detached task — `give` or `copy` it")); }
                            if *verb == Verb::LendMut && kind == SlotKind::Lent { return rt(*line, format!("'{v}' is lent (shared); it cannot be lent mutably")); }
                            captures.push((v.clone(), cell, *verb == Verb::LendMut));
                        }
                        Verb::Copy => captures.push((v.clone(), Rc::new(RefCell::new(val)), true)),
                        Verb::Give => {
                            if kind != SlotKind::Owned { return rt(*line, format!("'{v}' is lent, not owned; you cannot give away what you do not own")); }
                            if val.is_owned() { self.lookup_mut(v).unwrap().alive = false; }
                            captures.push((v.clone(), Rc::new(RefCell::new(val)), true));
                        }
                    }
                }
                let fails = self.task_fails(body);
                self.call_is_stmt = true;
                let ch = self.check_handler(handler.as_ref(), &fails, "concur", *line);
                self.call_is_stmt = false;
                ch?;
                self.tasks.push(Task { body: body.clone(), captures, handler: handler.clone(), fails, module: self.module.clone(), depth: self.scopes.len(), detached: *detached, line: *line });
                Ok(Flow::Normal)
            }
            Stmt::Whichever { arms, handler, line } => {
                let fails = vec![FailCase { name: "closed".into(), ty: None }];
                self.call_is_stmt = true;
                let ch = self.check_handler(handler.as_ref(), &fails, "whichever", *line);
                self.call_is_stmt = false;
                ch?;
                // the channels, each lent
                let mut chans = Vec::new();
                for arm in arms {
                    let (v, prov) = self.single(&arm.chan, arm.line)?;
                    match (v, prov) {
                        (Value::Chan(c), Prov::Borrowed(..)) | (Value::Chan(c), Prov::Plain) | (Value::Chan(c), Prov::Fresh) => chans.push(c),
                        (Value::Chan(_), Prov::Bare(n)) => return rt(arm.line, format!("a `whichever` arm lends its channel: `from lend '{n}'`")),
                        (other, _) => return rt(arm.line, format!("a `whichever` arm waits on a channel, got {}", other.kind())),
                    }
                }
                loop {
                    // a real value first, in arm order
                    for (i, c) in chans.iter().enumerate() {
                        if let Some(v) = self.chan_ready(c, false) {
                            return self.run_whichever_arm(&arms[i], v);
                        }
                    }
                    // nothing ready: let a task run
                    if self.run_any_task()? { continue; }
                    // nothing can run: time may pass — a timer channel fires
                    for (i, c) in chans.iter().enumerate() {
                        if let Some(v) = self.chan_ready(c, true) {
                            return self.run_whichever_arm(&arms[i], v);
                        }
                    }
                    // every channel is closed
                    self.call_is_stmt = true;
                    let r = self.handle_failure(handler.as_ref().unwrap(), *line, "whichever", "closed".into(), None, false);
                    self.call_is_stmt = false;
                    return r.map(|_| Flow::Normal);
                }
            }
            Stmt::MapFailStmt { op, args, handler, line } => {
                self.map_fail(op, args, handler.as_ref(), *line, false)?;
                Ok(Flow::Normal)
            }
            Stmt::CallValue { name, args, handler, line } => {
                if self.as_index(name, args, handler.is_some(), *line)?.is_some() {
                    return rt(*line, format!("'{name}'[…] on its own reads an item and drops it; to change it, `change '{name}'[…] = …;`"));
                }
                self.call_is_stmt = true;
                let r = self.call_value(name, args, handler.as_ref(), *line);
                self.call_is_stmt = false;
                r?;
                Ok(Flow::Normal)
            }
            Stmt::Match { value, arms, line } => {
                let (v, _) = self.single(value, *line)?;
                let (arm, data, dty) = self.pick_arm(v, arms, *line)?;
                self.arm_ty = dty;
                if arm.value.is_some() && !matches!(arm.value, Some(Expr::Match(..))) { return rt(arm.line, format!("case `{}` gives a value, but this `match` is not used as one", arm.name)); }
                match self.run_arm(arm, data, true)? {
                    Flow::Give(_) => rt(arm.line, format!("case `{}` gives a value, but this `match` is not used as one", arm.name)),
                    f => Ok(f),
                }
            }
            Stmt::Gives { value, line } => {
                let (v, _) = self.bracket(value, *line)?;
                Ok(Flow::Give(v))
            }
            Stmt::Asm { text, binds, line } => {
                if self.nocheck_depth == 0 && self.marks_needed() { return rt(*line, "an `asm` block is a no_check site: the compiler cannot see inside it"); }
                for b in binds {
                    match b {
                        AsmBind::In(v, _) | AsmBind::InOut(v, _) => { if self.lookup(v).is_none() { return rt(*line, format!("'{v}' is not declared")); } }
                        AsmBind::Lend(v, _, mutable) => {
                            let Some(s) = self.lookup(v) else { return rt(*line, format!("'{v}' is not declared")) };
                            if *mutable && s.kind == SlotKind::Lent { return rt(*line, format!("'{v}' is lent (shared); it cannot be lent mutably")); }
                            if *mutable && s.immut { return rt(*line, format!("'{v}' is `immut`, so `lend.mut '{v}'` cannot hand the assembly its address to write through")); }
                        }
                        _ => {}
                    }
                }
                let n = text.lines().filter(|l| !l.trim().is_empty()).count();
                rt(*line, format!("the oracle cannot run assembly ({n} instruction line(s), {} binding(s)); the compiler hands it to LLVM's assembler", binds.len()))
            }
            Stmt::Fail { name, payload, line } => {
                let case = match self.fails.iter().find(|f| &f.name == name) { Some(f) => f.clone(), None => return rt(*line, format!("`{name}` is not a listed failure here")) };
                let v = match payload {
                    None => None,
                    Some(b) => {
                        let (v, prov) = self.bracket(b, *line)?;
                        match prov {
                            Prov::Bare(n) => return rt(*line, format!("a failure carries bare '{n}', which owns a {}: say `give '{n}'` or `copy '{n}'`", v.kind())),
                            Prov::Borrowed(..) => return rt(*line, "a failure cannot carry a lend"),
                            _ => {}
                        }
                        Some(self.fit(v, prov, case.ty.as_ref(), *line)?)
                    }
                };
                Ok(Flow::Fail(name.clone(), v))
            }
            Stmt::BuiltinStmt { head, op, args, line } => { self.builtin(head, op, args, *line)?; Ok(Flow::Normal) }
            Stmt::Block { body, .. } => self.block(body),
            // The tiers (decided 2026-09-19). The oracle has no Cortex, so it cannot
            // prove; it runs every check and reports what the compiler *would* have
            // found: a `check` whose site fires is the missing guardrail, shown with
            // the concrete value (the compiler's counterexample); a `nocheck` whose
            // site fires is a lie — the compiled program would be undefined there.
            Stmt::Tier { tier, body, line } => {
                let off = matches!(tier, Tier::NoCheck | Tier::NoChecksAtAll);
                let trust = *tier == Tier::NoChecksAtAll;
                if off { self.nocheck_depth += 1; }
                if trust { self.trust_c_depth += 1; }
                let r = self.stmt(body);
                if off { self.nocheck_depth -= 1; }
                if trust { self.trust_c_depth -= 1; }
                match r {
                    Err(e) if e.site => {
                        let msg = match tier {
                            Tier::Check => format!("`check` at {} fails: {} — the guardrail is not enough (Cortex would refuse to compile; this value is the counterexample)", line_of(line), e.msg),
                            Tier::NoCheck => format!("`no_check` at {} lied: {} — the compiled program would be undefined here", line_of(line), e.msg),
                            Tier::NoChecksAtAll => format!("`NO_FUCKING_CHECKS` at {} lied: {} — the compiled program would be undefined here", line_of(line), e.msg),
                        };
                        Err(RuntimeError { line: e.line, msg, exit: None, site: false, leave: None })
                    }
                    other => other,
                }
            }
        }
    }

    fn cond(&mut self, b: &Bracket, line: usize) -> Result<bool, RuntimeError> {
        let (v, _) = self.single(b, line)?;
        match v {
            Value::Bool(t) => Ok(t),
            other => rt(line, format!("a condition must be a bool, this is {} — there is no truthiness", other.kind())),
        }
    }

    fn for_source(&mut self, b: &Bracket, line: usize) -> Result<(Vec<Value>, SlotKind), RuntimeError> {
        if b.values.len() != 1 { return rt(line, "`for … in` takes one list"); }
        let expr = match &b.values[0] { crate::ast::Value::Expr(e) => e };
        let (v, prov) = self.eval(expr)?;
        let items = match v {
            Value::List(items) => items,
            other => return rt(line, format!("`for` needs a list, this is {}", other.kind())),
        };
        let kind = match prov {
            Prov::Bare(n) => return rt(line, format!("'{n}' is owned: say how the loop takes it — `lend '{n}'`, `copy '{n}'`, or `give '{n}'`")),
            Prov::Borrowed(_, false) => SlotKind::Lent,
            Prov::Borrowed(_, true) => SlotKind::LentMut, // written back after each trip, and on a read of the list (the `for` arm)
            _ => SlotKind::Owned,
        };
        Ok((items, kind))
    }

    // ---- typing -----------------------------------------------------------

    /// Fit a value into a declared type (or leave it as is for `:=`).
    /// A bare numeric literal adopts the family of the declared type.
    fn fit(&mut self, v: Value, prov: Prov, ty: Option<&Ty>, line: usize) -> Result<Value, RuntimeError> {
        let have = self.last_kind.take();
        let lit = self.last_lit.take();
        let Some(t) = ty else { return Ok(v) };
        let (v, prov) = raw_land(v, prov, t, line)?;
        let written = v.clone();
        // a bin literal landing in a bin16/bin32 is rounded once, from its digits
        let v = match (&v, &prov, &lit, bin_w(t)) {
            (Value::Bin(_), Prov::Literal, Some(d), Some(w @ (16 | 32))) => Value::Bin(round_decimal(d, w)),
            _ => v,
        };
        let shown = |a: f64| lit.clone().filter(|_| matches!(prov, Prov::Literal)).unwrap_or_else(|| format!("{a}"));
        if matches!(prov, Prov::Given | Prov::Copied | Prov::Fresh) { self.kind_clash(have, t, line)?; }
        let v = match (t, &v, &prov) {
            // a literal landing in a brain16 is rounded to the nearest one (owner, 2026-10-02: a)
            (Ty::Brain, Value::Int(i), Prov::Literal) => Value::Brain(crate::value::round_brain(*i as f64)),
            (Ty::Brain, Value::Bin(x), Prov::Literal) => {
                let exact = lit.as_ref().and_then(|d| d.parse::<f64>().ok()).unwrap_or(*x);
                let b = crate::value::round_brain(exact);
                if b.is_infinite() { return site(line, format!("{} does not fit brain16: it rounds past the largest value", shown(exact))); }
                Value::Brain(b)
            }
            (Ty::Bin(_), Value::Int(i), Prov::Literal) => Value::Bin(*i as f64),
            (Ty::Ranged(b, _, _), Value::Int(i), Prov::Literal) if matches!(**b, Ty::Bin(_)) => Value::Bin(*i as f64),
            // a char literal takes the declared unit (`let 'g': char.grapheme = `é`;` is a grapheme, not a point)
            (Ty::Char(unit), Value::Char(_, c), Prov::Plain | Prov::Fresh) => Value::Char(unit.clone(), c.clone()),
            // `simd.fill[0]` takes its lane count from the declared type (design/simd.md)
            (Ty::Simd(_, n), Value::Simd(lanes), _) if lanes.len() != *n && lanes.iter().all(|l| *l == lanes[0]) => Value::Simd(vec![lanes[0].clone(); *n]),
            _ => v,
        };
        // A bin16/bin32 holds what its width holds: the value is rounded to it where it lands
        // (DECISIONS 2026-09-14: a bin32 0.1 is 0.100000001490116119384765625).
        let before = written;
        let v = round_to_width(t, v);
        if let (Value::Bin(a), Value::Bin(b)) = (&before, &v) {
            if a.is_finite() && b.is_infinite() { return site(line, format!("{} does not fit {}: it rounds past the largest value", shown(*a), ty_name(t))); }
        }
        self.check_type(&v, t, line)?;
        Ok(v)
    }

    fn check_type(&self, v: &Value, ty: &Ty, line: usize) -> Result<(), RuntimeError> {
        let name = ty_name(ty);
        match (ty, v) {
            (Ty::Int(_) | Ty::Uint(_), Value::Int(_) | Value::Big(_)) => {
                // at any width (owner, 2026-10-03); the standard ones stay on i128
                if let (Some((lo, hi)), Value::Int(i)) = (int_range(ty), v) {
                    if *i < lo || *i > hi { return site(line, format!("{i} does not fit {name} (range {lo} to {hi})")); }
                    return Ok(());
                }
                let (lo, hi) = crate::value::big_range(ty).unwrap();
                let b = as_big(v).unwrap();
                if b.cmp(&lo).is_lt() || b.cmp(&hi).is_gt() { return site(line, format!("{b} does not fit {name} (range {lo} to {hi})")); }
                Ok(())
            }
            (Ty::Int(_) | Ty::Uint(_), Value::Bin(b)) => rt(line, format!("{} is a bin, not {name}; there is no implicit conversion between numeric families", exact_decimal(*b))),
            (Ty::Bin(w), Value::Bin(b)) => {
                match w {
                    16 => if b.abs() > 65504.0 { return site(line, format!("{b} does not fit bin16 (max 65504)")); },
                    32 => if b.abs() > f32::MAX as f64 { return site(line, format!("{b} does not fit bin32")); },
                    128 => return rt(line, "bin128 is not implemented in the oracle"),
                    _ => {}
                }
                Ok(())
            }
            (Ty::Bin(_), Value::Int(i)) => rt(line, format!("{i} is an integer, not {name}; there is no implicit conversion between numeric families")),
            // the brain float is a family of its own (owner, 2026-10-02: a): it never mixes without `to.`
            (Ty::Brain, Value::Brain(_)) => Ok(()),
            (Ty::Brain, v) => rt(line, format!("{} is {}, not brain16; there is no implicit conversion between numeric families: `to.brain16[…]`", v.show_stored(), v.kind())),
            (Ty::Bin(_) | Ty::Int(_) | Ty::Uint(_), Value::Brain(b)) => rt(line, format!("{} is a brain16, not {name}; there is no implicit conversion between numeric families: `to.{name}[…]`", exact_decimal(*b))),
            (Ty::Deci(w), Value::Deci(d)) => if d.width == *w { Ok(()) } else { rt(line, format!("a deci{} is not {name}; widths do not change by themselves: `to.{name}[…]`", d.width)) },
            (Ty::Deci(_), v) => rt(line, format!("{} is {}, not {name}; there is no implicit conversion between numeric families", v.show_stored(), v.kind())),
            (Ty::Str(enc), Value::Str(s)) => {
                if *enc == Encoding::Ascii && !s.is_ascii() { return rt(line, "this text is not ASCII, so it cannot be str.ascii"); }
                Ok(())
            }
            (Ty::Bool, Value::Bool(_)) => Ok(()),
            (Ty::List(inner) | Ty::Array(inner), Value::List(items)) => {
                for it in items { self.check_type(it, inner, line)?; }
                Ok(())
            }
            (Ty::Duration, Value::Duration(_)) => Ok(()),
            (Ty::Channel(_), Value::Chan(_)) => Ok(()),
            (Ty::Simd(t, n), Value::Simd(lanes)) => {
                if lanes.len() != *n { return rt(line, format!("a {} has {n} lanes, this one has {}", name, lanes.len())); }
                for l in lanes { self.check_type(l, t, line)?; }
                Ok(())
            }
            (Ty::Map(kt, vt, _), Value::Map(ps)) => {
                for (k, v) in ps { self.check_type(k, kt, line)?; self.check_type(v, vt, line)?; }
                Ok(())
            }
            (Ty::TextRanged(enc, lo, hi, unit), Value::Str(s)) => {
                self.check_type(v, &Ty::Str(enc.clone()), line)?;
                let n = match unit { CharUnit::Byte => s.len(), CharUnit::Point => s.chars().count(), CharUnit::Grapheme => grapheme_count(s) } as i128;
                if n < *lo || n > *hi { return site(line, format!("a text of {n} {} does not fit {name}", crate::value::unit_word(unit))); }
                Ok(())
            }
            (Ty::Ranged(base, lo, hi), v) => {
                self.check_type(v, base, line)?;
                // `min` / `max` mean the base type's own limit (owner, 2026-09-23)
                let lo = &resolve_bound(base, lo);
                let hi = &resolve_bound(base, hi);
                let inside = match (v, lo, hi) {
                    (Value::Int(i), Bound::Int(a), Bound::Int(b)) => a <= i && i <= b,
                    (Value::Bin(x), Bound::Bin(a), Bound::Bin(b)) => a <= x && x <= b,
                    // a deci range (owner, 2026-09-29: a); the oracle holds the bounds as bins
                    (Value::Deci(d), Bound::Bin(a), Bound::Bin(b)) => { let x = d.to_f64(); *a <= x && x <= *b }
                    // a length bound on a list (decided 2026-09-21)
                    (Value::List(items), Bound::Int(a), Bound::Int(b)) => {
                        let n = items.len() as i128;
                        if !(*a <= n && n <= *b) { return site(line, format!("a list of {n} does not fit {name}")); }
                        true
                    }
                    (Value::Chan(_), _, _) => true,
                    (Value::Map(ps), Bound::Int(a), Bound::Int(b)) => {
                        let n = ps.len() as i128;
                        if !(*a <= n && n <= *b) { return site(line, format!("a map of {n} does not fit {name}")); }
                        true
                    }
                    _ => true,
                };
                if inside { Ok(()) } else { site(line, format!("{} does not fit {name}", v.show_useful())) }
            }
            (Ty::Fn(ps, ret, fails), Value::Func(fv)) => {
                let f = &fv.def;
                if f.name == "op -" && ps.len() == 1 { return rt(line, format!("expected {name}: `op -` subtracts two values; negation as a value is `op neg` (owner, 2026-09-29)")); }
                if f.params.len() != ps.len() { return rt(line, format!("expected {name}, got a function taking {} argument(s)", f.params.len())); }
                for (fp, (kind, t)) in f.params.iter().zip(ps) {
                    if fp.kind != *kind { return rt(line, format!("expected {name}: parameter '{}' is {} where {} is needed", fp.name, kind_word(fp.kind), kind_word(*kind))); }
                    if let Some(ft) = &fp.ty { if !matches!(t, Ty::Var(_)) && !matches!(ft, Ty::Var(_)) && ty_name(ft) != ty_name(t) { return rt(line, format!("expected {name}: parameter '{}' is {}, not {}", fp.name, ty_name(ft), ty_name(t))); } }
                }
                match (&f.ret, ret) {
                    (Some(a), Some(b)) => if !matches!(**b, Ty::Var(_)) && !matches!(a, Ty::Var(_)) && ty_name(a) != ty_name(b) { return rt(line, format!("expected {name}: returns {}, not {}", ty_name(a), ty_name(b))); },
                    (None, Some(b)) => return rt(line, format!("expected {name}: the function returns nothing, {} is needed", ty_name(b))),
                    (Some(a), None) => return rt(line, format!("expected {name}: the function returns {}, nothing is needed", ty_name(a))),
                    (None, None) => {}
                }
                let fnames: Vec<&str> = f.fails.iter().map(|c| c.name.as_str()).collect();
                if fnames.iter().any(|n| !fails.iter().any(|x| x == n)) || fails.iter().any(|x| !fnames.contains(&x.as_str())) {
                    return rt(line, format!("expected {name}: the function fails with [{}]", fnames.join(", ")));
                }
                Ok(())
            }
            (Ty::Fn(..), v) => rt(line, format!("expected {name}, got {}", v.kind())),
            // the element type of `to.array[…]` whose source's type the oracle did not know: unchecked
            (Ty::Var(n), _) if n.is_empty() => Ok(()),
            (Ty::Var(n), v) => {
                // a blank: bound by the first value that reaches it, checked by every later one
                let shape = shape_of(v);
                let mut vars = self.tvars.borrow_mut();
                match vars.get(n) {
                    Some(s) if *s == shape => Ok(()),
                    Some(s) => rt(line, format!("`<{n}>` was filled with {s} earlier in this call, now {shape} — one blank, one type")),
                    None => { vars.insert(n.clone(), shape); Ok(()) }
                }
            }
            (Ty::Inst(want, args), Value::Struct(have, fields)) => {
                if want != have { return rt(line, format!("expected a {name}, got a {have}")); }
                let def = match self.prog.structs.iter().find(|d| &d.name == want) { Some(d) => d.clone(), None => return rt(line, format!("no struct named `{want}`")) };
                if def.params.len() != args.len() { return rt(line, format!("`{want}` has {} blank(s) ({}), given {}", def.params.len(), def.params.iter().map(|p| format!("<{p}>")).collect::<Vec<_>>().join(", "), args.len())); }
                let map: HashMap<String, Ty> = def.params.iter().cloned().zip(args.iter().cloned()).collect();
                // what each blank accepts (owner, 2026-10-02)
                for (var, iface) in &def.bounds {
                    if let Some(t) = map.get(var) { if let Err(m) = self.satisfies(&ty_shape(t), iface, line)? { return rt(line, format!("`{want}` needs `<{var}>: {iface}`, but {m}")); } }
                }
                for f in &def.fields {
                    if let Some((_, fv)) = fields.iter().find(|(k, _)| k == &f.name) { self.check_type(fv, &subst(&f.ty, &map), line)?; }
                }
                Ok(())
            }
            (Ty::Inst(want, args), Value::Enum(have, case, data)) => {
                if want != have { return rt(line, format!("expected a {name}, got a {have}")); }
                let def = match self.prog.enums.iter().find(|d| &d.name == want) { Some(d) => d.clone(), None => return rt(line, format!("no enum named `{want}`")) };
                if def.params.len() != args.len() { return rt(line, format!("`{want}` has {} blank(s), given {}", def.params.len(), args.len())); }
                let map: HashMap<String, Ty> = def.params.iter().cloned().zip(args.iter().cloned()).collect();
                for (var, iface) in &def.bounds {
                    if let Some(t) = map.get(var) { if let Err(m) = self.satisfies(&ty_shape(t), iface, line)? { return rt(line, format!("`{want}` needs `<{var}>: {iface}`, but {m}")); } }
                }
                if let (Some(c), Some(d)) = (def.cases.iter().find(|c| c.name == *case), data) { if let Some(t) = &c.ty { self.check_type(d, &subst(t, &map), line)?; } }
                Ok(())
            }
            (Ty::Inst(want, _), v) => rt(line, format!("expected a {want}[…], got {}", v.kind())),
            (Ty::Struct(want), Value::Struct(have, _)) | (Ty::Struct(want), Value::Enum(have, _, _)) => {
                if want == have { Ok(()) } else { rt(line, format!("expected a {want}, got a {have}")) }
            }
            (Ty::Ptr(_, want_mut), Value::Ptr(p)) => {
                if *want_mut && !p.mutable { return rt(line, "a `ptr.mut` is needed, but this pointer is read-only (lend it with `lend.mut`)"); }
                Ok(())
            }
            (Ty::Ptr(..), v) => rt(line, format!("expected a pointer ({name}); got {} — a pointer is a `lend` of a list/array/str, or comes from C", v.kind())),
            (Ty::Struct(want), v) => {
                if !self.prog.structs.iter().any(|sd| &sd.name == want) && !self.prog.enums.iter().any(|e| &e.name == want) { return rt(line, format!("no struct or enum named `{want}` is declared")); }
                rt(line, format!("expected a {want}, got {}", v.kind()))
            }
            (Ty::Char(unit), Value::Char(_, s)) => {
                match unit {
                    CharUnit::Byte => if s.len() != 1 { return rt(line, format!("`{s}` is {} bytes, not one; it cannot be char.byte", s.len())); },
                    CharUnit::Point => if s.chars().count() != 1 { return rt(line, format!("`{s}` is {} code points, not one; it cannot be char.point (char.grapheme?)", s.chars().count())); },
                    CharUnit::Grapheme => if grapheme_count(s) != 1 { return rt(line, format!("`{s}` is {} graphemes, not one", grapheme_count(s))); },
                }
                Ok(())
            }
            (_, v) => rt(line, format!("expected {name}, got {}", v.kind())),
        }
    }

    // ---- values -----------------------------------------------------------

    fn single(&mut self, b: &Bracket, line: usize) -> Result<(Value, Prov), RuntimeError> {
        if b.values.len() != 1 { return rt(line, format!("expected one value, this bracket holds {}", b.values.len())); }
        self.value(&b.values[0])
    }

    /// One value, or a list if there are several.
    fn bracket(&mut self, b: &Bracket, line: usize) -> Result<(Value, Prov), RuntimeError> {
        match b.values.len() {
            0 => rt(line, "an empty `[ ]` is not a value"),
            1 if !b.one_list => self.value(&b.values[0]),
            _ => {
                // each item lands in an element slot, so the element type is its context, as a
                // declaration's type is (owner, 2026-09-29: a): `[0 -~ 1,]` in a list[uint8] wraps at 8 bits
                let elem = match &self.expected {
                    Some(Ty::List(t)) | Some(Ty::Array(t)) => Some((**t).clone()),
                    Some(Ty::Ranged(b, _, _)) => match &**b { Ty::List(t) | Ty::Array(t) => Some((**t).clone()), _ => None },
                    _ => None,
                };
                let saved = std::mem::replace(&mut self.expected, elem.clone());
                let mut items = Vec::new();
                for v in &b.values {
                    let r = self.value(v);
                    let (val, prov) = match r { Ok(x) => x, Err(e) => { self.expected = saved; return Err(e); } };
                    let val = elem_literal(elem.as_ref(), val, &prov);
                    match prov {
                        Prov::Bare(name) => { self.expected = saved; return rt(line, format!("bare '{name}' inside a list: say `give`, `copy`, or `lend`")); }
                        Prov::Borrowed(..) => { self.expected = saved; return rt(line, "a lend inside a list: a list owns its elements (`copy` or `give`)"); }
                        _ => {}
                    }
                    items.push((val, matches!(prov, Prov::Literal)));
                }
                // with no declared element type, a bare number takes its family from the other
                // items (owner, 2026-09-30: a): `[1, 2.5]` is bins, `[1, 'q']` with 'q' a deci is decis
                if elem.is_none() {
                    let deci = items.iter().find_map(|(v, _)| if let Value::Deci(d) = v { Some(d.width) } else { None });
                    let bin = items.iter().any(|(v, _)| matches!(v, Value::Bin(_)));
                    if let Some(w) = deci {
                        self.expected = Some(Ty::Deci(w));
                        for (k, v) in b.values.iter().enumerate() {
                            if items[k].1 && !matches!(items[k].0, Value::Deci(_)) {
                                match self.value(v) { Ok((val, _)) => items[k].0 = val, Err(e) => { self.expected = saved; return Err(e); } }
                            }
                        }
                    } else if bin {
                        for it in items.iter_mut() { if it.1 { if let Value::Int(i) = it.0 { it.0 = Value::Bin(i as f64); } } }
                    }
                }
                self.expected = saved;
                Ok((Value::List(items.into_iter().map(|(v, _)| v).collect()), Prov::Fresh))
            }
        }
    }

    fn value(&mut self, v: &crate::ast::Value) -> Result<(Value, Prov), RuntimeError> {
        self.value_mode(v, true)
    }

    fn value_mode(&mut self, v: &crate::ast::Value, _stored: bool) -> Result<(Value, Prov), RuntimeError> {
        match v {
            crate::ast::Value::Expr(e) => self.eval(e),
        }
    }

    /// The type of an expression where it can be worked out — a declared name, a
    /// conversion, either side of an operator — falling back to the type the value is
    /// being stored into. A shift's width comes from any of these (owner, 2026-09-23);
    /// only a bare literal with no context has none.
    fn expr_ty(&self, e: &Expr) -> Option<Ty> {
        // the expression's own type first, both sides of it; where it goes only when it has none
        // (check claude, 2026-10-05: a bare left side took the context's type, and the right side's
        // `uint16` was never asked)
        let t = self.expr_ty_own(e).or_else(|| self.expected.clone());
        match t { Some(Ty::Ranged(b, _, _)) => Some(*b), t => t }
    }

    fn expr_ty_own(&self, e: &Expr) -> Option<Ty> {
        match e {
            Expr::Var(name, _) => self.lookup(name).and_then(|s| s.ty.clone()),
            Expr::Paren(inner, _) => self.expr_ty_own(inner),
            Expr::Convert(t, _, _, _, _) => Some(t.clone()),
            Expr::Unary(_, inner, _) => self.expr_ty_own(inner),
            Expr::Binary(_, l, r, _) => self.expr_ty_own(l).or_else(|| self.expr_ty_own(r)),
            // a length is a uint64, and an item has its list's element type (check claude, seeds 5018, 5054)
            Expr::Builtin(h, op, _, _) if (h == "list" || h == "array") && op == "length" => Some(Ty::Uint(IntWidth::W64)),
            Expr::Index(name, _, _) | Expr::CallValue(name, _, _, _) => match self.lookup(name).and_then(|s| s.ty.clone()) {
                Some(Ty::List(t)) | Some(Ty::Array(t)) => Some(*t),
                Some(Ty::Ranged(b, _, _)) => match *b { Ty::List(t) | Ty::Array(t) => Some(*t), _ => None },
                _ => None,
            },
            // a vector folded to one lane answers with the lane's type (check claude, seed 4094)
            Expr::Builtin(h, op, args, _) if h == "simd" && matches!(op.as_str(), "max" | "min" | "sum" | "sum.lane" | "sum.wrap") => {
                match args.values.first() {
                    Some(crate::ast::Value::Expr(e)) => match self.expr_ty_own(e) { Some(Ty::Simd(lane, _)) => Some(*lane), _ => None },
                    _ => None,
                }
            }
            // a call's return type answers for it (operators.md, 2026-09-23 a: any operand whose type
            // is known; check claude, seeds 55 and 123, after `shl` came to need a width)
            Expr::Call(c) if c.limb.is_none() => {
                let (module, name) = c.path.split_at(c.path.len().max(1) - 1);
                self.prog.funcs.iter().find(|f| f.module == module && name.first() == Some(&f.name) && !f.is_op)
                    .and_then(|f| f.ret.clone()).filter(|t| !matches!(t, Ty::Var(_) | Ty::Inst(..)))
            }
            _ => None,
        }
    }

    /// the deci width a literal here would take, if a deci goes here
    fn deci_ctx(&self) -> Option<u8> {
        match &self.expected { Some(Ty::Deci(w)) => Some(*w), Some(Ty::Ranged(b, _, _)) => match **b { Ty::Deci(w) => Some(w), _ => None }, _ => None }
    }
    /// a literal's digits as a deci: exact, or an error when they do not fit (owner, 2026-09-29: b)
    fn deci_literal(&self, digits: &str, w: u8, line: usize) -> Result<crate::deci::Deci, RuntimeError> {
        match crate::deci::Deci::parse(digits, w) {
            Ok(Some(d)) => Ok(d),
            Ok(None) => rt(line, format!("`{digits}` is not a number")),
            Err(m) => rt(line, format!("`{digits}` {m}: write fewer digits, or use a wider deci")),
        }
    }

    /// `return lend 'x';` is checked where it is written, called or not (check claude, 2026-09-29):
    /// a borrow of an owned local or an owned parameter dies at the return, one outside the
    /// signature's group is refused, and one returned without `-> lent` must say so. A borrow
    /// that comes from a call or another lend is followed when the function runs.
    fn check_returned_borrows(&self) -> Result<(), RuntimeError> {
        fn walk<'a>(b: &'a [Stmt], out: &mut Vec<(&'a str, bool, usize)>, owned_locals: &mut Vec<&'a str>, calls: &mut Vec<(&'a crate::ast::Call, usize)>) {
            for s in b {
                match s {
                    Stmt::Return { value: Some(v), line } => match v.values.as_slice() {
                        [crate::ast::Value::Expr(Expr::Owned(verb @ (Verb::Lend | Verb::LendMut), n, _))] => out.push((n.as_str(), *verb == Verb::LendMut, *line)),
                        // `return b1[lend 'c', …]`: checked on every path too, not only when it runs
                        // (check claude's finding, 2026-10-02)
                        [crate::ast::Value::Expr(Expr::Call(c))] => calls.push((c, *line)),
                        _ => {}
                    },
                    Stmt::Decl { name, value, .. } => {
                        let borrowish = matches!(value.values.as_slice(), [crate::ast::Value::Expr(Expr::Owned(Verb::Lend | Verb::LendMut, ..) | Expr::Call(_) | Expr::CallValue(..) | Expr::Var(..))]);
                        if !borrowish { owned_locals.push(name.as_str()); }
                    }
                    Stmt::If { arms, otherwise, .. } => { for (_, b) in arms { walk(b, out, owned_locals, calls); } if let Some(b) = otherwise { walk(b, out, owned_locals, calls); } }
                    Stmt::Loop { body, .. } | Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::Range { body, .. } | Stmt::Block { body, .. } => walk(body, out, owned_locals, calls),
                    Stmt::Match { arms, .. } => for a in arms { walk(&a.body, out, owned_locals, calls); },
                    Stmt::Tier { body, .. } => walk(std::slice::from_ref(body), out, owned_locals, calls),
                    _ => {}
                }
            }
        }
        for f in &self.prog.funcs {
            let (mut rets, mut locals, mut calls) = (Vec::new(), Vec::new(), Vec::new());
            walk(&f.body, &mut rets, &mut locals, &mut calls);
            if let Some((_, Some(l))) = &f.ret_lent {
                for (c, line) in &calls {
                    for n in self.call_froms(c, 0) {
                        if let Some(p) = f.params.iter().find(|p| p.name == n) {
                            if p.life.as_ref() != Some(l) { return rt(*line, format!("`{}` returns a borrow of '{n}', which is not in @{l}: say `lent@{l} '{n}'`", f.name)); }
                        }
                    }
                }
            }
            for (n, _, line) in rets {
                let param = f.params.iter().find(|p| p.name == n);
                let lent = param.map_or(false, |p| matches!(p.kind, ParamKind::Lent | ParamKind::LentMut));
                match &f.ret_lent {
                    None => if lent || locals.contains(&n) || param.is_some() { return rt(line, format!("`{}` returns a borrow, so its signature must say so: `-> lent@x …`", f.name)); },
                    Some((_, life)) => {
                        if (param.is_some() && !lent) || (param.is_none() && locals.contains(&n)) {
                            return rt(line, format!("`{}` hands back a borrow of something that dies when it returns; a borrow it returns must come from a `lent` parameter", f.name));
                        }
                        if let (Some(p), Some(l)) = (param, life) {
                            if p.life.as_ref() != Some(l) { return rt(line, format!("`{}` returns a borrow of '{n}', which is not in @{l}: say `lent@{l} '{n}'`", f.name)); }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// The parameters a `-> lent` function may hand back a borrow of, on any path: `return lend 'p'`,
    /// a local that borrows 'p', or a call passing `lend 'p'` where that call may hand it back.
    fn may_return(&self, f: &Func, depth: usize) -> Vec<String> {
        fn walk<'a>(b: &'a [Stmt], rets: &mut Vec<&'a crate::ast::Value>, locals: &mut Vec<(&'a str, &'a crate::ast::Value)>) {
            for s in b {
                match s {
                    Stmt::Return { value: Some(v), .. } => if let [x] = v.values.as_slice() { rets.push(x); },
                    Stmt::Decl { name, value, .. } => if let [x] = value.values.as_slice() { locals.push((name.as_str(), x)); },
                    Stmt::If { arms, otherwise, .. } => { for (_, b) in arms { walk(b, rets, locals); } if let Some(b) = otherwise { walk(b, rets, locals); } }
                    Stmt::Loop { body, .. } | Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::Range { body, .. } | Stmt::Block { body, .. } => walk(body, rets, locals),
                    Stmt::Match { arms, .. } => for a in arms { walk(&a.body, rets, locals); },
                    Stmt::Tier { body, .. } => walk(std::slice::from_ref(body), rets, locals),
                    _ => {}
                }
            }
        }
        let mut out = Vec::new();
        if depth > 8 { return f.params.iter().filter(|p| p.kind != ParamKind::Owned).map(|p| p.name.clone()).collect(); }
        let (mut rets, mut locals) = (Vec::new(), Vec::new());
        walk(&f.body, &mut rets, &mut locals);
        let mut todo: Vec<&crate::ast::Value> = rets;
        let mut seen = 0;
        while let Some(v) = todo.pop() {
            seen += 1; if seen > 256 { break; }
            match v {
                crate::ast::Value::Expr(Expr::Owned(Verb::Lend | Verb::LendMut, n, _)) | crate::ast::Value::Expr(Expr::Var(n, _)) => {
                    if f.params.iter().any(|p| &p.name == n) { if !out.contains(n) { out.push(n.clone()); } }
                    else { for (ln, lv) in &locals { if ln == n { todo.push(lv); } } }
                }
                crate::ast::Value::Expr(Expr::Call(c)) => for a in self.call_froms(c, depth + 1) { if f.params.iter().any(|p| p.name == a) && !out.contains(&a) { out.push(a); } else { for (ln, lv) in &locals { if *ln == a { todo.push(lv); } } } },
                _ => {}
            }
        }
        out
    }

    /// The variables a call to a `-> lent` function lends it that its result may borrow:
    /// the arguments in its `@` group, or, without one, what its body may hand back.
    fn call_froms(&self, c: &crate::ast::Call, depth: usize) -> Vec<String> {
        let Some(g) = self.prog.funcs.iter().find(|f| f.name == *c.path.last().unwrap() && f.module == c.path[..c.path.len() - 1]) else { return Vec::new() };
        let names: Vec<String> = match &g.ret_lent {
            Some((_, Some(l))) => g.params.iter().filter(|p| p.life.as_ref() == Some(l)).map(|p| p.name.clone()).collect(),
            Some((_, None)) => self.may_return(g, depth),
            None => return Vec::new(),
        };
        g.params.iter().zip(&c.args.values).filter(|(p, _)| names.contains(&p.name)).filter_map(|(_, a)| match a {
            crate::ast::Value::Expr(Expr::Owned(Verb::Lend | Verb::LendMut, s, _)) => Some(s.clone()), _ => None }).collect()
    }

    /// The borrow rule on every path, not only the one a run takes (check claude, 2026-09-29):
    /// in each body, read in source order, a borrow `'r' := lend 's'` (or a `-> lent@x` call's
    /// result, for the arguments in @x) followed by a change, `give` or `lend.mut` of 's' and then a
    /// use of 'r' is refused. Loops are read once, top to bottom; a second trip round is left to
    /// the run-time check.
    fn check_borrow_order(&self) -> Result<(), RuntimeError> {
        // (line, the statement's own text, a new borrow: (name, what it borrows))
        fn flat(b: &[Stmt], out: &mut Vec<(usize, String, Option<(String, Vec<String>, bool)>)>, me: &Interp) {
            for st in b {
                match st {
                    Stmt::If { arms, otherwise, line } => { for (c, body) in arms { out.push((*line, format!("{c:?}"), None)); flat(body, out, me); } if let Some(o) = otherwise { flat(o, out, me); } }
                    Stmt::Loop { body, .. } | Stmt::Block { body, .. } | Stmt::Concur { body, .. } => flat(body, out, me),
                    Stmt::While { cond, body, line } => { out.push((*line, format!("{cond:?}"), None)); flat(body, out, me); }
                    Stmt::For { over, body, line, .. } => { out.push((*line, format!("{over:?}"), None)); flat(body, out, me); }
                    Stmt::Range { bounds, body, line, .. } => { out.push((*line, format!("{bounds:?}"), None)); flat(body, out, me); }
                    Stmt::Match { value, arms, line } => { out.push((*line, format!("{value:?}"), None)); for a in arms { flat(&a.body, out, me); } }
                    Stmt::Tier { body, .. } => flat(std::slice::from_ref(body), out, me),
                    Stmt::Decl { name, value, line, .. } => {
                        let from = match value.values.as_slice() {
                            [crate::ast::Value::Expr(Expr::Owned(Verb::Lend | Verb::LendMut, s, _))] => Some(vec![s.clone()]),
                            [crate::ast::Value::Expr(Expr::Call(c))] => Some(me.call_froms(c, 0)).filter(|v| !v.is_empty()),
                            _ => None,
                        };
                        let via_call = matches!(value.values.as_slice(), [crate::ast::Value::Expr(Expr::Call(_))]);
                        out.push((*line, format!("{st:?}"), from.map(|f| (name.clone(), f, via_call))));
                    }
                    other => out.push((stmt_line(other), format!("{other:?}"), None)),
                }
            }
        }
        let uses = |t: &str, r: &str| ["Var", "Index", "Field", "FieldIndex", "CallValue", "Spread"].iter().any(|k| t.contains(&format!("{k}(\"{r}\""))) || t.contains(&format!("Lend, \"{r}\"")) || t.contains(&format!("Copy, \"{r}\""));
        let writes = |t: &str, s: &str| -> Option<&'static str> {
            if t.contains(&format!("Give, \"{s}\"")) { Some("given away") }
            else if t.contains(&format!("LendMut, \"{s}\"")) { Some("lent mutably") }
            else if ["Assign", "AssignElem", "AssignField"].iter().any(|k| t.starts_with(&format!("{k} {{ name: \"{s}\""))) { Some("changed") }
            else { None }
        };
        let bodies = self.prog.funcs.iter().map(|f| &f.body).chain(self.prog.main.iter());
        for body in bodies {
            let mut ev = Vec::new();
            flat(body, &mut ev, self);
            // a borrow of a borrow borrows what that one does (check claude, 2026-10-05: 's' from
            // 'r', 'r' from 'o2', and 'o2' given away): follow each chain back to its owners
            let mut held: Vec<(String, Vec<String>)> = Vec::new();
            for (_, _, b) in ev.iter_mut() {
                let Some((r, froms, _)) = b else { continue };
                let mut all = froms.clone();
                for f in froms.iter() { if let Some((_, more)) = held.iter().rev().find(|(n, _)| n == f) { for m in more { if !all.contains(m) { all.push(m.clone()); } } } }
                *froms = all.clone();
                held.push((r.clone(), all));
            }
            for (i, (_, _, b)) in ev.iter().enumerate() {
                let Some((r, froms, via_call)) = b else { continue };
                let rel = if *via_call { "depends on it" } else { "borrows it" };
                for s in froms {
                    let mut wrote: Option<(usize, &str)> = None;
                    for (line, t, nb) in &ev[i + 1..] {
                        if matches!(nb, Some((n, _, _)) if n == r) { break; }   // a new 'r': the old borrow is over
                        if let Some((at, what)) = wrote { if uses(t, r) {
                            // the error is at the change, the line to move or fix (owner, 2026-09-29: a)
                            return rt(at, format!("'{s}' is {what} here while '{r}' still {rel} and is used on {}: a borrow never aliases a change and never outlives its owner", line_of(line)));
                        } }
                        if wrote.is_none() { if let Some(w) = writes(t, s) { wrote = Some((*line, w)); } }
                    }
                }
            }
        }
        Ok(())
    }

    /// Something was written through `name` (a change, a `give`, a `lend.mut`): every other borrow
    /// of the same thing made before now is stale. The writer itself, if a `lent.mut`, stays fresh.
    fn wrote(&mut self, name: &str, line: usize, what: &'static str) {
        self.write_count += 1;
        let n = self.write_count;
        if let Some(s) = self.lookup_mut(name) {
            let key = Rc::as_ptr(&s.cell) as *const () as usize;
            if s.kind == SlotKind::LentMut { s.since = n; }
            self.writes.insert(key, (n, line, what, name.to_string()));
        }
    }

    /// A borrow used after what it borrows was changed or given away (check claude's finding, 2026-09-29).
    fn stale(&self, name: &str, s: &Slot, line: usize) -> Result<(), RuntimeError> {
        // over `lend.mut 'xs'`: a read of 'xs' sees the element as changed so far, and after it
        // the element may not be used again in that trip (owner, 2026-09-30: a)
        for w in &self.mut_walks {
            if Rc::ptr_eq(&s.cell, &w.list) {
                if let Value::List(xs) = &mut *w.list.borrow_mut() { if let Some(slot) = xs.get_mut(w.pos.get()) { *slot = w.elem.borrow().clone(); } }
                if w.read_at.get().is_none() { w.read_at.set(Some(line)); }
            } else if Rc::ptr_eq(&s.cell, &w.elem) {
                if let Some(at) = w.read_at.get() {
                    return rt(at, format!("'{}' is used here while '{}', the loop's element, still borrows it `lend.mut` and is used on {}: inside `for '{}' in lend.mut '{}'`, the list can be read only after the element's last use in that trip", w.name, w.elem_name, line_of(line), w.elem_name, w.name));
                }
            }
        }
        if !matches!(s.kind, SlotKind::Lent | SlotKind::LentMut) { return Ok(()); }
        let own = Rc::as_ptr(&s.cell) as *const () as usize;
        for key in std::iter::once(&own).chain(s.deps.iter()) {
            if let Some((n, at, what, owner)) = self.writes.get(key) {
                if *n > s.since {
                    // the error is at the change, the line to move or fix (owner, 2026-09-29: a)
                    let rel = if *key == own { "borrows it" } else { "depends on it" };
                    return rt(*at, format!("'{owner}' is {what} here while '{name}' still {rel} and is used on {}: a borrow never aliases a change and never outlives its owner", line_of(line)));
                }
            }
        }
        Ok(())
    }

    fn read_var(&self, name: &str, line: usize) -> Result<Value, RuntimeError> {
        match self.lookup(name) {
            None => rt(line, format!("'{name}' is not declared")),
            Some(s) if matches!(s.kind, SlotKind::Extern | SlotKind::ExternMut) && self.nocheck_depth == 0 && self.marks_needed() =>
                rt(line, format!("'{name}' is extern: C's variable, which nothing can be proven about — reading it is a no_check site: `no_check … '{name}' …`")),
            Some(s) if !s.alive => rt(line, format!("'{name}' was given away and no longer owns anything")),
            Some(s) => { self.stale(name, s, line)?; Ok(s.cell.borrow().clone()) }
        }
    }

    fn eval(&mut self, e: &Expr) -> Result<(Value, Prov), RuntimeError> {
        // The type a value is stored into reaches the arithmetic that produces it, so a
        // bare literal can take its width from context — but it stops at a call, a
        // conversion or a construction: what `g[…]` *returns* says nothing about the
        // arithmetic in its arguments (found by the verification session, 2026-09-23).
        // It stops at a comparison too: a `bool` holds the comparison's answer, never its
        // operands, which take their width from each other (verification session, seed 93).
        // …except `simd.load` / `simd.fill`, which take their lane count from the declared type
        // (design/simd.md) — the stop had cut that off, so a `simd[int32, 4]` loaded 16 lanes
        // …and `list.fill`, whose value takes the element type, as a list literal's values do
        // (compiler claude, 2026-09-30: `list.fill[0.25, 'n']` into `list[deci64]` was refused)
        let simd_shape = matches!(e, Expr::Builtin(h, op, ..) if (h == "simd" && (op == "load" || op == "fill" || op == "load.some")) || (h == "list" && op == "fill"));
        let fresh = !simd_shape && matches!(e, Expr::Call(_) | Expr::CallValue(..) | Expr::Builtin(..) | Expr::Convert(..)
            | Expr::Construct(..) | Expr::MapFail(..) | Expr::MapLit(..) | Expr::CharFromCode(..)
            | Expr::Lambda(..) | Expr::ChannelNew(..) | Expr::EnumValue(..))
            || matches!(e, Expr::Binary(op, ..) if matches!(op.as_str(), "==" | "!==" | "<" | ">" | "</==" | ">/=="));
        let r = if fresh && self.expected.is_some() {
            let saved = self.expected.take();
            let r = self.eval_inner(e);
            self.expected = saved;
            r
        } else {
            self.eval_inner(e)
        };
        self.last_kind = if r.is_ok() { self.coll_kind(e) } else { None };
        self.last_lit = lit_digits(e);
        r
    }

    /// Is this expression, by its declared type, a `list` (false) or an `array` (true)?
    /// `None` when it is neither, or when nothing fixes it (a literal, a `:=` Cortex picks).
    fn coll_kind(&self, e: &Expr) -> Option<bool> {
        let kind = |t: &Ty| match t { Ty::List(_) => Some(false), Ty::Array(_) => Some(true), Ty::Ranged(b, _, _) => match &**b { Ty::List(_) => Some(false), Ty::Array(_) => Some(true), _ => None }, _ => None };
        match e {
            Expr::Var(n, _) | Expr::Owned(_, n, _) => self.lookup(n).and_then(|s| s.ty.as_ref().and_then(kind)),
            Expr::Paren(inner, _) => self.coll_kind(inner),
            Expr::Convert(t, ..) => kind(t),
            // a slice is a list, of an array too (owner, 2026-09-23)
            Expr::Builtin(h, op, ..) if (h == "list" || h == "array") && op == "slice" => Some(false),
            Expr::Call(c) if c.limb.is_none() => {
                let (module, name) = c.path.split_at(c.path.len() - 1);
                self.prog.funcs.iter().find(|f| f.module == module && f.name == name[0]).and_then(|f| f.ret.as_ref().and_then(kind))
            }
            _ => None,
        }
    }

    /// The one integer size an expression's plain math is at, by its declared names and
    /// conversions: `Err` names two sizes it mixes; `Ok(None)` when nothing states one.
    fn plain_size(&self, e: &Expr) -> Result<Option<Ty>, String> {
        let base = |t: Ty| match t { Ty::Ranged(b, _, _) => *b, t => t };
        match e {
            Expr::Var(n, _) | Expr::Owned(_, n, _) => Ok(self.lookup(n).and_then(|s| s.ty.clone()).map(base)),
            Expr::Convert(t, ..) => Ok(Some(base(t.clone()))),
            Expr::Paren(inner, _) | Expr::Unary(_, inner, _) => self.plain_size(inner),
            Expr::Binary(op, l, r, _) if matches!(op.as_str(), "+" | "-" | "x" | "xx") || is_div_mod(op) => {
                match (self.plain_size(l)?, self.plain_size(r)?) {
                    (Some(a), Some(b)) if ty_name(&a) != ty_name(&b) => Err(format!("{} and {}", ty_name(&a), ty_name(&b))),
                    (a, b) => Ok(a.or(b)),
                }
            }
            _ => Ok(None),
        }
    }

    /// The bin width an expression's math is at, from its declared names and conversions
    /// (never from where it is stored); `None` for bare literals alone.
    fn bin_width(&self, e: &Expr) -> Option<u8> {
        let w = |t: Option<Ty>| match t { Some(Ty::Bin(w)) => Some(w), Some(Ty::Ranged(b, _, _)) => match *b { Ty::Bin(w) => Some(w), _ => None }, _ => None };
        match e {
            Expr::Var(n, _) | Expr::Owned(_, n, _) => w(self.lookup(n).and_then(|s| s.ty.clone())),
            Expr::Convert(t, ..) => w(Some(t.clone())),
            Expr::Paren(inner, _) | Expr::Unary(_, inner, _) => self.bin_width(inner),
            Expr::Binary(_, l, r, _) => match (self.bin_width(l), self.bin_width(r)) { (Some(a), Some(b)) => Some(a.max(b)), (a, b) => a.or(b) },
            Expr::Call(c) if c.limb.is_none() => {
                let (module, name) = c.path.split_at(c.path.len() - 1);
                w(self.prog.funcs.iter().find(|f| f.module == module && f.name == name[0]).and_then(|f| f.ret.clone()))
            }
            _ => None,
        }
    }

    /// `'xs'[i]` when `'xs'` holds a list or array: the one position inside, as an index.
    /// `'xs'[2]` without `pos` on a list: refused, every index saying `pos` (owner, 2026-10-07: "`a`");
    /// `None` when the variable holds a function, which `[…]` calls.
    fn as_index(&self, name: &str, _args: &Bracket, _has_handler: bool, line: usize) -> Result<Option<Expr>, RuntimeError> {
        let holds_list = self.lookup(name).map(|s| matches!(&*s.cell.borrow(), Value::List(_) | Value::Str(_) | Value::Simd(_))).unwrap_or(false);
        if !holds_list { return Ok(None); }
        rt(line, format!("a read says it is a position: `'{name}'[pos 2]` (owner, 2026-10-07)"))
    }

    /// A type C's numbers are checked against when they come in: a `layout.C enum`, a range,
    /// or a struct with a ranged field. Its name for messages, or `None`.
    fn c_checked(&self, t: &Ty) -> Option<String> {
        match t {
            Ty::Ranged(..) => Some(ty_name(t)),
            Ty::Struct(n) if self.prog.enums.iter().any(|e| &e.name == n && e.c_values.is_some()) => Some(n.clone()),
            Ty::Struct(n) if self.prog.structs.iter().any(|d| &d.name == n && d.fields.iter().any(|f| matches!(f.ty, Ty::Ranged(..)))) => Some(n.clone()),
            _ => None,
        }
    }

    /// The element type a pointer expression is declared to point at, when stated.
    fn ptr_elem(&self, e: &crate::ast::Value) -> Option<Ty> {
        match e { crate::ast::Value::Expr(Expr::Var(n, _)) => match self.lookup(n).and_then(|s| s.ty.clone()) { Some(Ty::Ptr(inner, _)) => Some(*inner), _ => None }, _ => None }
    }

    /// C's number read as a checked type: a C enum's case, or the value if it fits the range.
    fn from_c(&self, v: Value, t: &Ty) -> Option<Value> {
        match (t, &v) {
            (Ty::Struct(n), Value::Int(k)) => {
                let e = self.prog.enums.iter().find(|e| &e.name == n)?;
                let i = e.c_values.as_ref()?.iter().position(|x| x == k)?;
                Some(Value::Enum(e.name.clone(), e.cases[i].name.clone(), None))
            }
            (Ty::Struct(_), Value::Enum(..)) => Some(v),
            (Ty::Ranged(..), _) => if self.check_type(&v, t, 0).is_ok() { Some(v) } else { None },
            _ => Some(v),
        }
    }

    /// The case of `v` among a `match`'s arms, after checking every case is covered.
    fn pick_arm<'a>(&self, v: Value, arms: &'a [Arm], line: usize) -> Result<(&'a Arm, Option<Value>, Option<Ty>), RuntimeError> {
        let (tname, case, data) = match v { Value::Enum(t, c, d) => (t, c, d), other => return rt(line, format!("`match` takes an enum value, got {} — for other types use `if`", other.kind())) };
        let def = match self.prog.enums.iter().find(|e| e.name == tname) { Some(d) => d.clone(), None => return rt(line, format!("no enum named `{tname}`")) };
        for a in arms {
            match def.cases.iter().find(|c| c.name == a.name) {
                None => return rt(a.line, format!("`{tname}` has no case `{}` (it has {})", a.name, def.cases.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", "))),
                Some(c) => if a.bind.is_some() && c.ty.is_none() { return rt(a.line, format!("`{}` carries nothing; there is no value to name", a.name)); },
            }
        }
        let missing: Vec<&str> = def.cases.iter().filter(|c| !arms.iter().any(|a| a.name == c.name)).map(|c| c.name.as_str()).collect();
        if !missing.is_empty() { return rt(line, format!("`match` on `{tname}` does not handle {}; every case must be covered", missing.join(", "))); }
        // the data's type, so a bound name has a width (check claude, 2026-09-29); a generic case's is left open
        let dty = def.cases.iter().find(|c| c.name == case).and_then(|c| c.ty.clone()).filter(|t| !matches!(t, Ty::Var(_)) && def.params.is_empty());
        Ok((arms.iter().find(|a| a.name == case).unwrap(), data.map(|d| *d), dty))
    }

    /// Run one case with its data bound: its value if it is one, else its statements' flow.
    fn run_arm(&mut self, arm: &Arm, data: Option<Value>, as_stmt: bool) -> Result<Flow, RuntimeError> {
        self.push();
        if let (Some(b), Some(d)) = (&arm.bind, data) {
            let ty = self.arm_ty.take();
            let r = self.declare(b, Slot { cell: Rc::new(RefCell::new(d)), ty, kind: SlotKind::Owned, alive: true, immut: false, sized: false, sealed: false , since: 0, deps: Vec::new() }, arm.line);
            if let Err(e) = r { self.pop(); return Err(e); }
        }
        let r = match &arm.value {
            // a case holding only a `match`, inside a `match` used as a statement: that inner match is
            // a statement too — the outer match decides (owner, 2026-09-29: a)
            Some(Expr::Match(v, arms, l)) if as_stmt => self.stmt(&Stmt::Match { value: v.clone(), arms: arms.clone(), line: *l }),
            Some(e) => self.eval(e).map(|(v, _)| Flow::Give(v)),
            None => self.stmts(&arm.body),
        };
        self.pop();
        r
    }

    fn kind_clash(&self, have: Option<bool>, want: &Ty, line: usize) -> Result<(), RuntimeError> {
        let want = match want { Ty::Ranged(b, _, _) => &**b, t => t };
        match (have, want) {
            (Some(false), Ty::Array(_)) => rt(line, "a list where an array is wanted: nothing converts by itself — `to.array[give …]`"),
            (Some(true), Ty::List(_)) => rt(line, "an array where a list is wanted: nothing converts by itself — `to.list[give …]`"),
            _ => Ok(()),
        }
    }

    fn eval_inner(&mut self, e: &Expr) -> Result<(Value, Prov), RuntimeError> {
        let line = e.line();
        match e {
            // a literal where a deci goes is that deci, from its digits (owner, 2026-09-29): too many
            // digits for the width is an error, never rounded
            Expr::Int(i, _) if self.deci_ctx().is_some() => { let w = self.deci_ctx().unwrap(); Ok((Value::Deci(crate::deci::Deci::from_int(*i, w).or_else(|m| rt(line, format!("`{i}` {m}: write fewer digits, or use a wider deci")))?), Prov::Literal)) }
            Expr::Bin(_, digits, _) if self.deci_ctx().is_some() => { let w = self.deci_ctx().unwrap(); Ok((Value::Deci(self.deci_literal(digits, w, line)?), Prov::Literal)) }
            Expr::Int(i, _) => Ok((Value::Int(*i), Prov::Literal)),
            Expr::RawInt(i, _) => Ok((Value::Int(*i), Prov::Raw)),
            Expr::BigInt(s, _) => Ok((Value::Big(crate::bigint::BigInt::parse(s).expect("lexed digits")), Prov::Literal)),
            // `*0.1` — exactly this, a deci (owner, 2026-10-02): of the width the place wants, or,
            // under `:=`, deci64 — the oracle's pick; Cortex chooses the width
            Expr::Exact(digits, _) => {
                if let Some(t @ (Ty::Bin(_) | Ty::Brain | Ty::Int(_) | Ty::Uint(_))) = &self.expected {
                    return rt(line, format!("`*{digits}` is exact, a deci; a {} cannot hold it exactly: drop the `*`, or make this a deci", ty_name(t)));
                }
                let w = self.deci_ctx().unwrap_or(64);
                Ok((Value::Deci(self.deci_literal(digits, w, line)?), Prov::Plain))
            }
            Expr::Duration(n, _) => Ok((Value::Duration(*n), Prov::Plain)),
            Expr::ChannelNew(_, cap, _) => {
                self.next_chan_id += 1;
                Ok((Value::Chan(Rc::new(RefCell::new(Chan { queue: std::collections::VecDeque::new(), cap: *cap, timer: None, id: self.next_chan_id }))), Prov::Fresh))
            }
            Expr::MapLit(pairs, _) => {
                let mut out: Vec<(Value, Value)> = Vec::new();
                for (ke, vb) in pairs {
                    let (k, kp) = self.eval(ke)?;
                    match kp { Prov::Bare(n) => return rt(line, format!("bare '{n}' as a map key: say `give '{n}'` or `copy '{n}'`")), Prov::Borrowed(..) => return rt(line, "a map owns its keys; a lend cannot be one (`copy` or `give`)"), _ => {} }
                    self.key_ok(&k, line)?;
                    let (v, vp) = self.bracket(vb, line)?;
                    match vp { Prov::Bare(n) => return rt(line, format!("bare '{n}' as a map value: say `give '{n}'` or `copy '{n}'`")), Prov::Borrowed(..) => return rt(line, "a map owns its values; a lend cannot be one (`copy` or `give`)"), _ => {} }
                    for (k2, _) in &out { if self.keys_equal(k2, &k, line)? { return rt(line, format!("the key {} is written twice in this map literal", k.show_useful())); } }
                    out.push((k, v));
                }
                Ok((Value::Map(out), Prov::Fresh))
            }
            Expr::MapFail(op, args, handler, _) => {
                let v = self.map_fail(op, args, handler.as_ref(), line, true)?;
                let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                Ok((v, prov))
            }
            Expr::CharFromCode(unit, b, handler, _) => {
                let fails = vec![FailCase { name: "not_a_char".into(), ty: None }];
                let uname = match unit { CharUnit::Point => "point", CharUnit::Byte => "byte", CharUnit::Grapheme => "grapheme" };
                self.check_handler(handler.as_ref(), &fails, &format!("char.{uname}.from_code"), line)?;
                let (v, _) = self.single(b, line)?;
                let n = match v { Value::Int(n) => n, other => return rt(line, format!("`char.{uname}.from_code` takes a number, got {}", other.kind())) };
                let made = match unit {
                    CharUnit::Byte => u8::try_from(n).ok().map(|b| (b as char).to_string()),
                    _ => u32::try_from(n).ok().and_then(char::from_u32).map(|c| c.to_string()),
                };
                match made {
                    Some(s) => Ok((Value::Char(unit.clone(), s), Prov::Plain)),
                    None => { let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("char.{uname}.from_code"), "not_a_char".into(), None, true)?; let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain }; Ok((v, prov)) }
                }
            }
            Expr::Spread(n, _, _) => rt(line, format!("`...'{n}'` unpacks a list into a call's variadic arguments; it has no meaning here")),
            Expr::Bin(b, _, _) => Ok((Value::Bin(*b), Prov::Literal)),
            Expr::Bool(b, _) => Ok((Value::Bool(*b), Prov::Plain)),
            Expr::Text(t, _) => Ok((Value::Str(t.clone()), Prov::Fresh)),
            Expr::Newline(_) => Ok((Value::Str("\n".into()), Prov::Fresh)),
            Expr::Char(c, _) => {
                // With `:=`, the unit is inferred: the narrowest that holds the content
                // (provisional — Cortex would also look at use).
                let unit = if c.len() == 1 { CharUnit::Byte }
                    else if c.chars().count() == 1 { CharUnit::Point }
                    else if grapheme_count(c) == 1 { CharUnit::Grapheme }
                    else { return rt(line, format!("`{c}` is {} characters, not one", grapheme_count(c))); };
                let prov = if unit == CharUnit::Grapheme { Prov::Fresh } else { Prov::Plain };
                Ok((Value::Char(unit, c.clone()), prov))
            }
            Expr::Var(name, _) => {
                let v = self.read_var(name, line)?;
                // a const is baked in: every use is a fresh value, nobody owns it
                let prov = if self.is_const(name) { if v.is_owned() { Prov::Fresh } else { Prov::Plain } }
                    else if v.is_owned() { Prov::Bare(name.clone()) } else { Prov::Plain };
                Ok((v, prov))
            }
            Expr::Owned(verb, name, _) => {
                let (kind, alive, cell, generic) = match self.lookup(name) {
                    Some(s) => (s.kind, s.alive, s.cell.clone(), matches!(s.ty, Some(Ty::Var(_)) | Some(Ty::Inst(..)))),
                    None => return rt(line, format!("'{name}' is not declared")),
                };
                if !alive { return rt(line, format!("'{name}' was already given away")); }
                self.stale(name, self.lookup(name).unwrap(), line)?;
                if kind == SlotKind::Const { return rt(line, format!("'{name}' is a const: nobody owns it, every use is a fresh value — write it bare, no `give`/`copy`/`lend`")); }
                if matches!(kind, SlotKind::Extern | SlotKind::ExternMut) { return rt(line, format!("'{name}' is extern: C's plain value, it copies by itself — no `give`/`copy`/`lend`")); }
                let v = cell.borrow().clone();
                if !v.is_owned() {
                    // In generic code the type may be owned or plain; the verbs are written for
                    // the owned case and do nothing for a plain one (design/generics.md).
                    if generic { return Ok((v, Prov::Plain)); }
                    return rt(line, format!("'{name}' is a plain {} and owns nothing; it copies by itself — no `give`/`copy`/`lend`", v.kind()));
                }
                match verb {
                    Verb::Give => {
                        if kind != SlotKind::Owned { return rt(line, format!("'{name}' is lent, not owned; you cannot give away what you do not own")); }
                        self.lookup_mut(name).unwrap().alive = false;
                        self.wrote(name, line, "given away");
                        Ok((v, Prov::Given))
                    }
                    // copying a function that owns captures copies what it owns; its lends stay lends
                    Verb::Copy => Ok((match v {
                        Value::Func(f) => Value::Func(FuncVal {
                            captures: f.captures.iter().zip(&f.owned).map(|((n, c, m), o)| (n.clone(), if *o { Rc::new(RefCell::new(c.borrow().clone())) } else { c.clone() }, *m)).collect(),
                            def: f.def, owned: f.owned,
                        }),
                        v => v,
                    }, Prov::Copied)),
                    Verb::Lend => Ok((v, Prov::Borrowed(cell, false))),
                    Verb::LendMut => {
                        if kind == SlotKind::Lent { return rt(line, format!("'{name}' is lent (shared); it cannot be lent mutably")); }
                        self.wrote(name, line, "lent mutably");
                        Ok((v, Prov::Borrowed(cell, true)))
                    }
                }
            }
            Expr::Call(c) => {
                self.call_borrow = None;
                let v = self.call(c)?;
                match self.call_borrow.take() { Some((cell, m)) => { self.pending_deps = std::mem::take(&mut self.call_deps); Ok((v, Prov::Borrowed(cell, m))) } None => Ok((v, Prov::Fresh)) }
            }
            Expr::Nested(b) => self.bracket(b, line),
            Expr::Index(name, idx, _) => {
                let v = self.read_var(name, line)?;
                let items = match v {
                    // a vector's lane, 1-based like every position (simd.md; check claude, 2026-09-29)
                    Value::List(items) | Value::Simd(items) => items,
                    Value::Str(_) => return rt(line, format!("'{name}' is a text: `[pos …]` would not say which unit — the 3rd byte or the 3rd grapheme; use `cord::str.at.grapheme[lend '{name}', pos …]` (decided 2026-09-20)")),
                    other => return rt(line, format!("'{name}' is {}, not a list or array; it cannot be indexed", other.kind())),
                };
                let (i, _) = self.eval(idx)?;
                let i = match i { Value::Int(i) => i, other => return rt(line, format!("an index is an integer, got {}", other.kind())) };
                // 1-based (decided 2026-09-17): 'xs'[pos 1] is the first; 'xs'[pos 0] does not exist.
                if i < 1 || i as usize > items.len() {
                    return site(line, format!("position {i} is out of range for '{name}', which has {} element(s) (1 to {})", items.len(), items.len()));
                }
                let item = items[(i - 1) as usize].clone();
                let prov = if item.is_owned() { Prov::Bare(format!("{name}'[{i}]'")) } else { Prov::Plain };
                Ok((item, prov))
            }
            Expr::FieldIndex(name, field, idx, _) => {
                let v = self.read_var(name, line)?;
                let fv = match v {
                    Value::Struct(sname, fields) => match fields.into_iter().find(|(k, _)| k == field) {
                        Some((_, v)) => v,
                        None => return rt(line, format!("{sname} has no field `{field}`")),
                    },
                    other => return rt(line, format!("'{name}' is {}, not a struct", other.kind())),
                };
                let items = match fv { Value::List(items) => items, other => return rt(line, format!("'{name}'.{field} is {}, not a list or array; it cannot be indexed", other.kind())) };
                let (i, _) = self.eval(idx)?;
                let i = match i { Value::Int(i) => i, other => return rt(line, format!("an index is an integer, got {}", other.kind())) };
                if i < 1 || i as usize > items.len() {
                    return site(line, format!("position {i} is out of range for '{name}'.{field}, which has {} element(s) (1 to {})", items.len(), items.len()));
                }
                let item = items[(i - 1) as usize].clone();
                let prov = if item.is_owned() { Prov::Bare(format!("{name}'.{field}[{i}]'")) } else { Prov::Plain };
                Ok((item, prov))
            }
            Expr::Field(name, field, _) => {
                let v = self.read_var(name, line)?;
                let (sname, fields) = match v { Value::Struct(n, f) => (n, f), other => return rt(line, format!("'{name}' is {}, not a struct; it has no field `{field}`", other.kind())) };
                match fields.into_iter().find(|(k, _)| k == field) {
                    Some((_, fv)) => {
                        let prov = if fv.is_owned() { Prov::Bare(format!("{name}'.{field} (write `copy '{name}'.{field}` or `lend '{name}'.{field}`; the field is owned by '{name}")) } else { Prov::Plain };
                        Ok((fv, prov))
                    }
                    None => rt(line, format!("{sname} has no field `{field}`")),
                }
            }
            Expr::OwnedField(verb, name, field, _) => {
                let (kind, alive, cell) = match self.lookup(name) {
                    Some(s) => (s.kind, s.alive, s.cell.clone()),
                    None => return rt(line, format!("'{name}' is not declared")),
                };
                if !alive { return rt(line, format!("'{name}' was already given away")); }
                let fv = match &*cell.borrow() {
                    Value::Struct(sname, fields) => match fields.iter().find(|(k, _)| k == field) {
                        Some((_, v)) => v.clone(),
                        None => return rt(line, format!("{sname} has no field `{field}`")),
                    },
                    other => return rt(line, format!("'{name}' is {}, not a struct", other.kind())),
                };
                if !fv.is_owned() { return rt(line, format!("'{name}'.{field} is a plain {} and owns nothing; it copies by itself — no `give`/`copy`/`lend`", fv.kind())); }
                match verb {
                    Verb::Give => {
                        // Decided 2026-09-18: giving a field away consumes the whole variable.
                        if kind != SlotKind::Owned { return rt(line, format!("'{name}' is lent, not owned; you cannot give away part of what you do not own")); }
                        self.lookup_mut(name).unwrap().alive = false;
                        self.wrote(name, line, "given away");
                        Ok((fv, Prov::Given))
                    }
                    Verb::Copy => Ok((fv, Prov::Copied)),
                    Verb::Lend | Verb::LendMut => {
                        if *verb == Verb::LendMut && kind == SlotKind::Lent { return rt(line, format!("'{name}' is lent (shared); its fields cannot be lent mutably")); }
                        let tmp = Rc::new(RefCell::new(fv.clone()));
                        if *verb == Verb::LendMut { self.field_borrows.push((cell.clone(), field.clone(), tmp.clone())); }
                        Ok((fv, Prov::Borrowed(tmp, *verb == Verb::LendMut)))
                    }
                }
            }
            Expr::OwnedElem(verb, name, idx, _) => {
                // an element handed over with a verb: lend/copy read it; give would leave a hole
                let (v, _) = self.eval(&Expr::Index(name.clone(), idx.clone(), line))?;
                if !v.is_owned() { return rt(line, format!("'{name}'[pos …] is a plain {} and owns nothing; it copies by itself — no `give`/`copy`/`lend`", v.kind())); }
                match verb {
                    Verb::Give => rt(line, format!("giving an element out of '{name}' would leave a hole; `copy` it, or take it out: `list.remove[lend.mut '{name}', …]` hands it back (decided 2026-09-21)")),
                    Verb::Copy => Ok((v, Prov::Copied)),
                    Verb::Lend => Ok((v.clone(), Prov::Borrowed(Rc::new(RefCell::new(v)), false))),
                    Verb::LendMut => rt(line, format!("`lend.mut` of an element of '{name}' is not modelled by the oracle; lend the whole list")),
                }
            }
            Expr::Construct(sname, given, _) => {
                let def = match self.prog.structs.iter().find(|d| d.name == *sname) {
                    Some(d) => d.clone(),
                    None => return rt(line, format!("no struct named `{sname}` is declared (a call to a function is `{sname}[…]` with plain values, not `field = …`)")),
                };
                let from = self.module.clone();
                self.check_path(&from, &def.module, line)?;
                if !self.reachable(&from, &def.module, def.vis) {
                    return rt(line, private_msg(&self.module, &def.module, &format!("struct `{sname}`")));
                }
                // a generic struct's blanks are filled by the values given (Cortex: from use)
                let saved_tvars = std::mem::take(&mut *self.tvars.borrow_mut());
                let r = self.construct_fields(&def, sname, given, line);
                *self.tvars.borrow_mut() = saved_tvars;
                return r;
            }
            Expr::ConstPath(module, name, from, _) => {
                self.check_path(from, module, line)?;
                let def = match self.prog.consts.iter().find(|c| &c.module == module && &c.name == name) {
                    Some(c) => c,
                    None => return rt(line, format!("no const '{name}' in `{}`", root_or(module))),
                };
                if !self.reachable(from, module, def.vis) {
                    return rt(line, format!("const '{name}' is private to `{}`; mark it `pub` or `pub.neuron`", root_or(module)));
                }
                let v = self.consts[&const_key(module, name)].cell.borrow().clone();
                let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                Ok((v, prov))
            }
            // `to.array[give 'l']` / `to.list[give 'a']` (owner, 2026-09-23): the elements are
            // handed over as they are; only whether it can grow changes
            Expr::Convert(Ty::List(_) | Ty::Array(_), b, _, _, _) => {
                let (v, prov) = self.single(b, line)?;
                match (&v, &prov) {
                    (_, Prov::Bare(n)) => rt(line, format!("converting bare '{n}': say `give '{n}'` (no copy) or `copy '{n}'` (keeps '{n}')")),
                    (_, Prov::Borrowed(..)) => rt(line, "converting a lend would copy it without saying so: `copy` it or `give` it"),
                    (Value::List(_), _) => Ok((v, Prov::Fresh)),
                    (other, _) => rt(line, format!("`to.list` / `to.array` take a list or an array, got {}", other.kind())),
                }
            }
            // `to.Key['n']` — a number into a C enum: a case with that number, or `out_of_range`
            // (owner, 2026-09-27: what comes from C as a plain number is converted this way)
            Expr::Convert(Ty::Struct(tname), b, handler, _, _) => {
                let def = match self.prog.enums.iter().find(|e| &e.name == tname) {
                    Some(e) if e.c_values.is_some() => e.clone(),
                    Some(_) => return rt(line, format!("no conversion to the enum `{tname}`; only a `layout.C enum`'s numbers convert")),
                    None => return rt(line, format!("no conversion to {tname}; conversions are between numbers, text, lists and arrays, and into a `layout.C enum`")),
                };
                let fails = vec![FailCase { name: "out_of_range".into(), ty: None }];
                self.check_handler(handler.as_ref(), &fails, &format!("to.{tname}"), line)?;
                let (v, _) = self.single(b, line)?;
                let n = match v { Value::Int(n) => n, other => return rt(line, format!("`to.{tname}` takes a number, got {}", other.kind())) };
                let vals = def.c_values.as_ref().unwrap();
                match vals.iter().position(|x| *x == n) {
                    Some(i) => Ok((Value::Enum(def.name.clone(), def.cases[i].name.clone(), None), Prov::Fresh)),
                    None => {
                        let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{tname}"), "out_of_range".into(), None, true)?;
                        let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                        Ok((v, prov))
                    }
                }
            }
            // `to.int8.bits[128]`: the value's bits read as the type's (owner, 2026-10-06), as a literal
            // lands without ASP; a value the type's bits cannot hold is refused, as there
            Expr::Convert(ty @ (Ty::Int(w) | Ty::Uint(w)), b, _, 3, _) => {
                let (v, _) = self.single(b, line)?;
                let bits = match w { IntWidth::W8 => 8, IntWidth::W16 => 16, IntWidth::W32 => 32, IntWidth::W64 => 64, IntWidth::W128 => 128, IntWidth::Other(n) => *n };
                let Value::Int(i) = v else { return rt(line, format!("`to.{}.bits[…]` takes a whole number, not {}", ty_name(ty), v.kind())) };
                if bits >= 128 { return rt(line, format!("`to.{}.bits` is not built in the oracle", ty_name(ty))); }
                if i < -(1i128 << (bits - 1)) || i >= 1i128 << bits {
                    return rt(line, format!("`{i}` does not fit the {bits} bits of {}: `to.{}.bits` reads a pattern of {bits} bits", ty_name(ty), ty_name(ty)));
                }
                let u = i.rem_euclid(1i128 << bits);
                let out = if matches!(ty, Ty::Int(_)) && u >= 1i128 << (bits - 1) { u - (1i128 << bits) } else { u };
                Ok((Value::Int(out), Prov::Fresh))
            }
            Expr::Convert(ty, b, handler, form, _) => {
                let form = *form;
                let useful_flag = form == 2;
                let useful = &useful_flag;
                let (v, _) = self.single(b, line)?;
                // a fallback takes the conversion's type: `to.deci32['x'] otherwise 0` is a deci32 0
                self.fallback_ty = match ty { Ty::Ranged(bt, ..) => Some((**bt).clone()), Ty::TextRanged(e, ..) => Some(Ty::Str(e.clone())), t @ (Ty::Int(_) | Ty::Uint(_) | Ty::Bin(_) | Ty::Brain | Ty::Deci(_) | Ty::Str(_) | Ty::Char(_) | Ty::Bool) => Some(t.clone()), _ => None };
                let target = ty_name(ty);
                let full_ty = ty;
                let text_base;
                let (base, ranged): (&Ty, bool) = match ty {
                    Ty::Ranged(b, _, _) => (b, true),
                    Ty::TextRanged(e, _, _, _) => { text_base = Ty::Str(e.clone()); (&text_base, true) }   // a bounded text: out of it is `out_of_range` (2026-09-22)
                    t => (t, false),
                };
                let can_fail = matches!(v, Value::Str(_)) && matches!(base, Ty::Int(_) | Ty::Uint(_) | Ty::Bin(_) | Ty::Brain | Ty::Deci(_));
                // text → number fails `not_a_number`, and a real number too big for the type fails
                // `out_of_range`: text comes from outside, so it is a failure, not a check (owner, 2026-09-24: c)
                let mut fails = if can_fail { vec![FailCase { name: "not_a_number".into(), ty: None }, FailCase { name: "out_of_range".into(), ty: None }] } else { Vec::new() };
                let from_text = can_fail;
                // bin → int fails unless the bin is whole (decided 2026-09-21): rounding is said first, in std
                if matches!(v, Value::Bin(_) | Value::Brain(_)) && matches!(base, Ty::Int(_) | Ty::Uint(_)) { fails.push(FailCase { name: "not_whole".into(), ty: None }); }
                // deci (owner, 2026-09-29): a deci → integer fails `not_whole`, as a bin does; text or an
                // integer with more digits than the width fails `too_many_digits` — an integer only when its
                // type could have that many (an int32 into a deci64 always fits)
                if matches!(v, Value::Deci(_)) && matches!(base, Ty::Int(_) | Ty::Uint(_)) { fails.push(FailCase { name: "not_whole".into(), ty: None }); }
                // a whole bin or deci too big for the integer fails `out_of_range`, as text does (owner,
                // 2026-09-30: a); one `otherwise` handles both
                if matches!(v, Value::Bin(_) | Value::Brain(_) | Value::Deci(_)) && matches!(base, Ty::Int(_) | Ty::Uint(_)) && !ranged { fails.push(FailCase { name: "out_of_range".into(), ty: None }); }
                if let Ty::Deci(w) = base {
                    let p = crate::deci::precision(*w);
                    let digits = |n: i128| n.unsigned_abs().to_string().len();
                    let may_exceed = match &v {
                        // a value past i128 has at least 39 digits, more than any deci holds
                        Value::Str(_) | Value::Big(_) => true,
                        // a wider deci into a narrower one (owner, 2026-09-29: a)
                        Value::Deci(d) => d.width > *w,
                        Value::Int(_) => match b.values.first() { Some(crate::ast::Value::Expr(e)) => match self.expr_ty(e) {
                            Some(t @ (Ty::Int(_) | Ty::Uint(_))) => crate::value::int_range(&t).map_or(true, |(lo, hi)| digits(lo).max(digits(hi)) > p),
                            Some(Ty::Ranged(_, Bound::Int(lo), Bound::Int(hi))) => digits(lo).max(digits(hi)) > p,
                            _ => false,
                        }, _ => false },
                        _ => false,
                    };
                    if may_exceed { fails.push(FailCase { name: "too_many_digits".into(), ty: None }); }
                }
                // text → char fails unless the text is exactly one unit; char → a smaller unit fails
                // unless it fits (decided 2026-09-21). Numbers never convert to or from chars.
                if let Ty::Char(unit) = base {
                    match &v {
                        Value::Str(_) => fails.push(FailCase { name: "not_one_char".into(), ty: None }),
                        Value::Char(from, _) => match (from, unit) {
                            (CharUnit::Grapheme, CharUnit::Point) => fails.push(FailCase { name: "not_one_point".into(), ty: None }),
                            (CharUnit::Grapheme | CharUnit::Point, CharUnit::Byte) => fails.push(FailCase { name: "not_one_byte".into(), ty: None }),
                            _ => {}
                        },
                        _ => {}
                    }
                }
                // text into ASCII fails `not_ascii` unless every character is ASCII (owner, 2026-09-30: "`otherwise`
                // exists"); from ASCII text, a number or a byte it cannot
                // bytes into text (owner, 2026-09-30: "yea"): a list or array of uint8 read as UTF-8, failing
                // `not_utf8` when they are not; only utf8 reads bytes so far
                let src_ty = match b.values.first() { Some(crate::ast::Value::Expr(e)) => self.expr_ty(e), _ => None };
                let from_bytes = matches!(&src_ty, Some(Ty::List(t) | Ty::Array(t)) if **t == Ty::Uint(IntWidth::W8));
                if from_bytes {
                    match base {
                        Ty::Str(Encoding::Utf8) => fails.push(FailCase { name: "not_utf8".into(), ty: None }),
                        Ty::Str(_) => return rt(line, format!("only `to.str.utf8` reads bytes as text (owner, 2026-09-30); `to.{target}` from bytes is not decided — convert to utf8 first")),
                        _ => {}
                    }
                }
                if matches!(base, Ty::Str(Encoding::Ascii)) {
                    let src = src_ty.clone();
                    let may_fail = match (&v, &src) {
                        (_, Some(Ty::Str(Encoding::Ascii))) => false,
                        (Value::Str(_), _) => true,
                        (Value::Char(u, _), _) => *u != CharUnit::Byte,
                        _ => false,
                    };
                    if may_fail { fails.push(FailCase { name: "not_ascii".into(), ty: None }); }
                }
                // into a range type, a value out of range is a failure, not a check (decided 2026-09-20)
                if ranged && !from_text { fails.push(FailCase { name: "out_of_range".into(), ty: None }); }
                let ty = base;
                self.check_handler(handler.as_ref(), &fails, &format!("to.{target}"), line)?;
                // Into a range type, a value that does not even fit the base type is out of the range
                // too: a failure the writer handles, never a crash (input.md, decided 2026-09-20).
                macro_rules! fit_base { ($v:expr) => {
                    if let Err(e) = self.check_type(&$v, ty, line) {
                        if !ranged && !from_text { return Err(e); }
                        let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{target}"), "out_of_range".into(), None, true)?;
                        let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                        return Ok((v, prov));
                    }
                } }
                let out = match (ty, &v) {
                    (Ty::Str(_), Value::Nothing) => return rt(line, "a call that returns nothing has no text"),
                    (Ty::Str(Encoding::Utf8), Value::List(items)) if from_bytes => {
                        let bytes: Vec<u8> = items.iter().map(|x| match x { Value::Int(n) => *n as u8, _ => 0 }).collect();
                        match String::from_utf8(bytes) {
                            Ok(t) => Value::Str(t),
                            Err(_) => {
                                let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{target}"), "not_utf8".into(), None, true)?;
                                let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                                return Ok((v, prov));
                            }
                        }
                    }
                    (Ty::Str(Encoding::Ascii), Value::Str(s) | Value::Char(_, s)) if !s.is_ascii() => {
                        let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{target}"), "not_ascii".into(), None, true)?;
                        let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                        return Ok((v, prov));
                    }
                    // a list or array is not text (owner, 2026-09-30: b): bytes read as UTF-8 above, any other
                    // list is joined or printed by the writer
                    (Ty::Str(_), Value::List(_)) => return rt(line, format!("a list is not text: only a list of `uint8` converts, read as UTF-8 bytes (`to.str.utf8['bytes'] otherwise …`); join the items with `++`, or print the list directly (owner, 2026-09-30)")),
                    // a struct, a map, or a case carrying data has no one text; the writer builds the text they want
                    // (owner, 2026-09-30: "refuse")
                    (Ty::Str(_), Value::Struct(n, _)) => return rt(line, format!("a `{n}` is not text: build the text you want from its fields, e.g. `to.str.utf8['p'.x] ++ …`, or print it directly (owner, 2026-09-30)")),
                    (Ty::Str(_), Value::Map(_)) => return rt(line, "a map is not text: build the text you want from its entries, or print it directly (owner, 2026-09-30)"),
                    // an enum with any data-carrying case is not text, whichever case this value is: the rule
                    // comes from the type, as every conversion's does (owner, 2026-09-30: a)
                    (Ty::Str(_), Value::Enum(t, _, None)) if self.prog.enums.iter().find(|e| &e.name == t).is_some_and(|e| e.cases.iter().any(|k| k.ty.is_some())) => {
                        let dc = self.prog.enums.iter().find(|e| &e.name == t).and_then(|e| e.cases.iter().find(|k| k.ty.is_some())).map(|k| k.name.clone()).unwrap_or_default();
                        return rt(line, format!("a `{t}` is not text: its case `{dc}` carries data, so no `{t}` converts, whichever case it holds; `match` it and build the text you want, or print it directly (owner, 2026-09-30)"));
                    }
                    (Ty::Str(_), Value::Enum(t, c, Some(_))) => return rt(line, format!("`{t}.{c}` carries data, so it is not text: `match` it and build the text you want, or print it directly (owner, 2026-09-30)")),
                    // `to.useful.str…` is `print.useful`'s text, `to.str…` is `print`'s (owner, 2026-09-30)
                    // a value that is not text says which text it becomes, as a print says which form it
                    // shows (owner, 2026-10-02): `to.stored.str.utf8[…]` or `to.useful.str.utf8[…]`
                    (Ty::Str(_), other) if form == 0 && !matches!(other, Value::Str(_) | Value::Char(..)) =>
                        return rt(line, format!("say which text `to.{target}` makes of {}: `to.stored.{target}[…]` (what is stored) or `to.useful.{target}[…]` (the useful value) (owner, 2026-10-02)", other.kind())),
                    (Ty::Str(_), other) => Value::Str(if *useful { show_useful_as(other, src_ty.as_ref()) } else { other.show_stored() }),
                    // a bin into a deci says which value it takes, as text does (owner, 2026-10-02: "Ah, opt 1")
                    (Ty::Deci(_), other @ (Value::Bin(_) | Value::Brain(_))) if form == 0 =>
                        return rt(line, format!("say which value `to.{target}` takes of a {}: `to.stored.{target}[…]` (what is stored) or `to.useful.{target}[…]` (the useful value) (owner, 2026-10-02)", other.kind())),
                    (Ty::Int(_) | Ty::Uint(_), Value::Int(i)) => { fit_base!(Value::Int(*i)); Value::Int(*i) }
                    // a wide integer's value (owner, 2026-10-03): into an integer type as any narrowing
                    // is; into a bin, a brain16 or a deci, rounded from its digits as a literal is
                    (Ty::Int(_) | Ty::Uint(_), Value::Big(_)) => { fit_base!(v.clone()); v.clone() }
                    (Ty::Bin(w), Value::Big(b)) => {
                        let x: f64 = b.to_string().parse().unwrap();
                        let x = if *w == 32 { (x as f32) as f64 } else if *w == 16 { round_f16(x) } else { x };
                        if x.is_infinite() { return site(line, format!("{b} does not fit bin{w}: it rounds past the largest value")); }
                        Value::Bin(x)
                    }
                    (Ty::Brain, Value::Big(b)) => {
                        let x = crate::value::round_brain(b.to_string().parse().unwrap());
                        if x.is_infinite() { return site(line, format!("{b} does not fit brain16: it rounds past the largest value")); }
                        Value::Brain(x)
                    }
                    // past i128 it always has more digits than a deci holds: `too_many_digits`, as an
                    // integer's does (compiler claude, 2026-10-05)
                    (Ty::Deci(w), Value::Big(b)) => match crate::deci::Deci::parse(&b.to_string(), *w) {
                        Ok(Some(d)) => Value::Deci(d),
                        _ => match handler { Some(h) => { let v = self.handle_failure(h, line, &format!("to.{target}"), "too_many_digits".into(), None, true)?; let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain }; return Ok((v, prov)); } None => return rt(line, format!("{b} has too many digits for deci{w}")) },
                    },
                    (Ty::Bin(_), Value::Int(i)) => { let b = Value::Bin(*i as f64); fit_base!(b.clone()); b }
                    (Ty::Bin(_), Value::Bin(x)) => { fit_base!(Value::Bin(*x)); Value::Bin(*x) }
                    // the brain float (owner, 2026-10-02): into it rounds to the nearest, out of it into a
                    // bin32/bin64 is exact, into a bin16 rounds (a check site past bin16's largest)
                    (Ty::Brain, Value::Int(i)) => Value::Brain(crate::value::round_brain(*i as f64)),
                    (Ty::Brain, Value::Bin(x)) => {
                        let r = no_new_neg_zero(*x, crate::value::round_brain(*x));
                        if r.is_infinite() { return site(line, format!("{x} does not fit brain16: it rounds past the largest value")); }
                        Value::Brain(r)
                    }
                    (Ty::Brain, Value::Deci(d)) => {
                        let r = crate::value::round_brain(d.show_stored().parse::<f64>().unwrap_or(d.to_f64()));
                        if r.is_infinite() { return site(line, format!("{} does not fit brain16: it rounds past the largest value", d.show_stored())); }
                        Value::Brain(r)
                    }
                    (Ty::Brain, Value::Brain(x)) => Value::Brain(*x),
                    (Ty::Brain, Value::Str(s)) => {
                        let t = s.strip_prefix(['-', '+']).unwrap_or(s);
                        let (mant, exp) = match t.find(['e', 'E']) { Some(i) => (&t[..i], Some(&t[i + 1..])), None => (t, None) };
                        let (int, frac) = match mant.split_once('.') { Some((a, b)) => (a, Some(b)), None => (mant, None) };
                        let digits = |d: &str| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit());
                        let ok = digits(int) && frac.map_or(true, digits) && exp.map_or(true, |e| digits(e.strip_prefix(['-', '+']).unwrap_or(e)));
                        let why = match s.parse::<f64>() {
                            Ok(x) if ok => { let r = crate::value::round_brain(x); if r.is_infinite() { "out_of_range" } else { fit_base!(Value::Brain(r)); return Ok((Value::Brain(r), Prov::Plain)); } }
                            _ => "not_a_number",
                        };
                        let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{target}"), why.into(), None, true)?;
                        let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                        return Ok((v, prov));
                    }
                    (Ty::Bin(w), Value::Brain(x)) => {
                        let b = if *w == 16 { Value::Bin(round_f16(*x)) } else { Value::Bin(*x) };
                        if let Value::Bin(y) = b { if y.is_infinite() { return site(line, format!("{} does not fit bin16: past its largest value, 65504", exact_decimal(*x))); } }
                        fit_base!(b.clone()); b
                    }
                    (Ty::Int(_) | Ty::Uint(_), Value::Brain(x)) => {
                        if x.fract() != 0.0 {
                            let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{target}"), "not_whole".into(), None, true)?;
                            let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                            return Ok((v, prov));
                        }
                        let i = Value::Int(*x as i128);
                        if x.abs() >= 1.7e38 || self.check_type(&i, ty, line).is_err() {
                            let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{target}"), "out_of_range".into(), None, true)?;
                            let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                            return Ok((v, prov));
                        }
                        i
                    }
                    (Ty::Deci(w), Value::Brain(x)) => {
                        let d = if *useful {
                            match crate::deci::Deci::parse(&crate::value::brain_useful(*x), *w) { Ok(Some(d)) => Ok(d), _ => crate::deci::Deci::from_bin_useful(*x, *w) }
                        } else { crate::deci::Deci::from_bin_exact(*x, *w) };
                        Value::Deci(d.or_else(|m| site(line, format!("{x} {m}")))?)
                    }
                    (Ty::Int(_) | Ty::Uint(_), Value::Bin(x)) => {
                        if !x.is_finite() || x.fract() != 0.0 {
                            let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{target}"), "not_whole".into(), None, true)?;
                            let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                            return Ok((v, prov));
                        }
                        let i = Value::Int(*x as i128);
                        if x.abs() >= 1.7e38 || self.check_type(&i, ty, line).is_err() {
                            let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{target}"), "out_of_range".into(), None, true)?;
                            let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                            return Ok((v, prov));
                        }
                        i
                    }
                    (Ty::Int(_) | Ty::Uint(_) | Ty::Bin(_), Value::Str(s)) => {
                        // text that is a bin: an optional sign, digits, an optional `.` and digits, and an
                        // optional exponent `e`/`E` (owner, 2026-09-24: b). No `inf`, no `nan`, nothing
                        // else: those fail `not_a_number` and land in the writer's `otherwise`.
                        let bin_text = |t: &str| {
                            let t = t.strip_prefix(['-', '+']).unwrap_or(t);
                            let (mant, exp) = match t.find(['e', 'E']) { Some(i) => (&t[..i], Some(&t[i + 1..])), None => (t, None) };
                            let (int, frac) = match mant.split_once('.') { Some((a, b)) => (a, Some(b)), None => (mant, None) };
                            let digits = |d: &str| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit());
                            digits(int) && frac.map_or(true, digits)
                                && exp.map_or(true, |e| digits(e.strip_prefix(['-', '+']).unwrap_or(e)))
                        };
                        // rounded once, from the digits, straight to the width (never 64 bits first)
                        let parsed = match ty {
                            Ty::Bin(w @ (16 | 32)) if bin_text(s) => Some(Value::Bin(round_decimal(&plain_decimal(s), *w))),
                            Ty::Bin(_) => if bin_text(s) { s.parse::<f64>().ok().map(Value::Bin) } else { None },
                            _ => s.parse::<i128>().ok().map(Value::Int),
                        };
                        // a well-formed number past what the oracle can hold is still a number: too big
                        let digits = s.strip_prefix(['-', '+']).unwrap_or(s);
                        let huge = parsed.is_none() && !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit());
                        let infinite = matches!(parsed, Some(Value::Bin(x)) if x.is_infinite()) && s.bytes().any(|b| b.is_ascii_digit());
                        if huge || infinite {
                            let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{target}"), "out_of_range".into(), None, true)?;
                            let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                            return Ok((v, prov));
                        }
                        match parsed {
                            Some(p) => { fit_base!(p.clone()); p }
                            None => {
                                let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{target}"), "not_a_number".into(), None, true)?;
                                let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                                return Ok((v, prov));
                            }
                        }
                    }
                    // a C enum's case to its number (owner, 2026-09-28): never fails; the number must fit
                    (Ty::Int(_) | Ty::Uint(_), Value::Enum(tn, cn, None)) => {
                        let Some(e) = self.prog.enums.iter().find(|e| &e.name == tn && e.c_values.is_some()) else { return rt(line, format!("only a `layout.C enum` has numbers; `{tn}` is a Cyborg enum")) };
                        let i = e.cases.iter().position(|c| &c.name == cn).unwrap();
                        let k = Value::Int(e.c_values.as_ref().unwrap()[i]);
                        fit_base!(k.clone());
                        k
                    }
                    (Ty::Int(_) | Ty::Uint(_), Value::Char(_, c)) => return rt(line, format!("a character does not convert to a number: `char.code[`{c}`]` is its code, `to.int32[to.str.utf8[…]]` its digit value (decided 2026-09-21)")),
                    (Ty::Char(_), Value::Int(_)) => return rt(line, "a number does not convert to a character: `char.point.from_code[…] otherwise …` (decided 2026-09-21)"),
                    (Ty::Char(unit), Value::Str(s) | Value::Char(_, s)) => {
                        let count = match unit { CharUnit::Byte => s.len(), CharUnit::Point => s.chars().count(), CharUnit::Grapheme => grapheme_count(s) };
                        if count != 1 {
                            let which = if matches!(v, Value::Str(_)) { "not_one_char" } else if *unit == CharUnit::Point { "not_one_point" } else { "not_one_byte" };
                            let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{target}"), which.into(), None, true)?;
                            let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                            return Ok((v, prov));
                        }
                        Value::Char(unit.clone(), s.clone())
                    }
                    // deci (owner, 2026-09-29)
                    (Ty::Deci(w), Value::Int(i)) => match crate::deci::Deci::from_int(*i, *w) {
                        Ok(d) => Value::Deci(d),
                        Err(m) => match handler { Some(h) => { let v = self.handle_failure(h, line, &format!("to.{target}"), "too_many_digits".into(), None, true)?; let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain }; return Ok((v, prov)); } None => return rt(line, format!("{i} {m}")) },
                    },
                    (Ty::Deci(w), Value::Bin(x)) => {
                        if !x.is_finite() { return rt(line, format!("a bin {} into a deci is not decided", if x.is_nan() { "NaN" } else { "infinity" })); }
                        // `.useful`: the shortest decimal that reads back as the same bin *at its own width*
                        // (a bin32 0.1 is `0.1`, not its 64-bit widening; compiler session, 2026-09-29)
                        let src_ty = match b.values.first() { Some(crate::ast::Value::Expr(e)) => self.expr_ty(e), _ => None };
                        let d = if *useful {
                            let text = show_useful_as(&Value::Bin(*x), src_ty.as_ref());
                            match crate::deci::Deci::parse(&text, *w) { Ok(Some(d)) => Ok(d), _ => crate::deci::Deci::from_bin_useful(*x, *w) }
                        } else { crate::deci::Deci::from_bin_exact(*x, *w) };
                        Value::Deci(d.or_else(|m| site(line, format!("{x} {m}")))?)
                    }
                    (Ty::Deci(w), Value::Deci(d)) => {
                        if d.width <= *w { Value::Deci(d.widen(*w)) } else {
                            match d.narrow(*w) {
                                Ok(n) => Value::Deci(n),
                                Err(why) => {
                                    let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{target}"), why.into(), None, true)?;
                                    let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                                    return Ok((v, prov));
                                }
                            }
                        }
                    }
                    (Ty::Deci(w), Value::Str(s)) => match crate::deci::Deci::parse(s.trim_start_matches('+'), *w) {
                        Ok(Some(d)) => Value::Deci(d),
                        other => {
                            let why = match other { Ok(None) => "not_a_number", Err(m) if m.contains("digits") => "too_many_digits", _ => "out_of_range" };
                            let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{target}"), why.into(), None, true)?;
                            let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                            return Ok((v, prov));
                        }
                    },
                    (Ty::Int(_) | Ty::Uint(_), Value::Deci(d)) => match d.to_int() {
                        None => {
                            let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{target}"), "not_whole".into(), None, true)?;
                            let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                            return Ok((v, prov));
                        }
                        Some(Ok(i)) if self.check_type(&Value::Int(i), ty, line).is_ok() => Value::Int(i),
                        Some(_) => {
                            let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{target}"), "out_of_range".into(), None, true)?;
                            let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                            return Ok((v, prov));
                        }
                    },
                    (Ty::Bin(w), Value::Deci(d)) => {
                        // rounded once, from the digits, straight to the width
                        let b = if *w == 16 || *w == 32 { Value::Bin(round_decimal(&d.show_stored(), *w)) } else { Value::Bin(d.to_f64()) };
                        fit_base!(b.clone()); b
                    }
                    (Ty::Bool, _) => return rt(line, "nothing converts to bool: there is no truthiness; compare instead"),
                    (t, other) => return rt(line, format!("no conversion from {} to {}", other.kind(), ty_name(t))),
                };
                // a converted bin16/bin32 holds what its width holds (DECISIONS 2026-09-14)
                let before = out.clone();
                let out = round_to_width(ty, out);
                if let (Value::Bin(a), Value::Bin(b)) = (&before, &out) {
                    if a.is_finite() && b.is_infinite() {
                        if !(ranged || from_text) { return site(line, format!("{a} does not fit {}: it rounds past the largest value", ty_name(ty))); }
                        let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{target}"), "out_of_range".into(), None, true)?;
                        let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                        return Ok((v, prov));
                    }
                }
                if let Ty::TextRanged(..) = full_ty {
                    if self.check_type(&out, full_ty, line).is_err() {
                        let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{}", ty_name(full_ty)), "out_of_range".into(), None, true)?;
                        let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                        return Ok((v, prov));
                    }
                }
                if let Ty::Ranged(_, lo, hi) = full_ty {
                    let inside = match (&out, lo, hi) { (Value::Int(i), Bound::Int(a), Bound::Int(b)) => a <= i && i <= b, (Value::Bin(x), Bound::Bin(a), Bound::Bin(b)) => a <= x && x <= b, (Value::Deci(d), Bound::Bin(a), Bound::Bin(b)) => { let x = d.to_f64(); a <= &x && &x <= b } _ => true };
                    if !inside {
                        let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{}", ty_name(full_ty)), "out_of_range".into(), None, true)?;
                        let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                        return Ok((v, prov));
                    }
                }
                let prov = if out.is_owned() { Prov::Fresh } else { Prov::Plain };
                Ok((out, prov))
            }
            Expr::EnumValue(tname, case, _, _) => self.enum_value(tname, case, None, &Vec::new(), line),
            Expr::FnRef(path, from, _) => {
                let (module, name) = path.split_at(path.len() - 1);
                let name = &name[0];
                self.check_path(from, module, line)?;
                let f = match self.prog.funcs.iter().find(|f| f.module == module && &f.name == name) {
                    Some(f) => f,
                    None => return rt(line, format!("no function `{name}` in `{}` (a bare name with no brackets is the function as a value)", root_or(module))),
                };
                if !self.reachable(from, &f.module, f.vis) { return rt(line, format!("`{name}` is private to `{}`", root_or(&f.module))); }
                if f.is_op { return rt(line, format!("`{name}` is an operator; it is not a value")); }
                if f.is_extern { return rt(line, format!("`{name}` is extern; a C function as a value is not decided")); }
                Ok((Value::Func(FuncVal { def: Rc::new(f.clone()), captures: Vec::new(), owned: Vec::new() }), Prov::Plain))
            }
            Expr::OpValue(op, def, _) => {
                let def = match def {
                    Some(d) => d.clone(),
                    None => {
                        let ops: Vec<&Func> = self.prog.funcs.iter().filter(|f| f.is_op && f.name == *op).collect();
                        match ops.len() {
                            0 => return rt(line, format!("no operator `{op}` to take as a value")),
                            1 => Rc::new(ops[0].clone()),
                            // oracle's limit: it does not pick by where the value goes
                            _ => return rt(line, format!("there are several `op {op}`; the oracle cannot yet pick one by where the value goes")),
                        }
                    }
                };
                Ok((Value::Func(FuncVal { def, captures: Vec::new(), owned: Vec::new() }), Prov::Plain))
            }
            Expr::Lambda(def, caps, _) => {
                // captures are taken now, each with its verb; nothing else from this scope is visible inside
                let mut captures = Vec::new();
                let mut owned = Vec::new();
                for (verb, v) in caps {
                    let (kind, alive, cell, val) = match self.lookup(v) { Some(s) => (s.kind, s.alive, s.cell.clone(), s.cell.borrow().clone()), None => return rt(line, format!("'{v}' is not declared")) };
                    if !alive { return rt(line, format!("'{v}' was already given away")); }
                    owned.push(matches!(verb, Verb::Copy | Verb::Give));
                    match verb {
                        Verb::Lend => captures.push((v.clone(), cell, false)),
                        Verb::LendMut => { if kind == SlotKind::Lent { return rt(line, format!("'{v}' is lent (shared); it cannot be lent mutably")); } captures.push((v.clone(), cell, true)); }
                        Verb::Copy => captures.push((v.clone(), Rc::new(RefCell::new(val)), true)),
                        Verb::Give => {
                            if kind != SlotKind::Owned { return rt(line, format!("'{v}' is lent, not owned; you cannot give away what you do not own")); }
                            if val.is_owned() { self.lookup_mut(v).unwrap().alive = false; }
                            captures.push((v.clone(), Rc::new(RefCell::new(val)), true));
                        }
                    }
                }
                let v = Value::Func(FuncVal { def: def.clone(), captures, owned });
                let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                Ok((v, prov))
            }
            Expr::CallValue(name, args, handler, _) => {
                // `'xs'[2]` reads a list and `'f'[2]` calls a function: no `pos` since 2026-09-27
                // (owner), so what the variable holds decides
                if let Some(idx) = self.as_index(name, args, handler.is_some(), line)? {
                    return self.eval(&Expr::Index(name.clone(), Box::new(idx), line));
                }
                let v = self.call_value(name, args, handler.as_ref(), line)?;
                let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                Ok((v, prov))
            }
            Expr::SizeOf(ty, is_size, _) => {
                let (size, align) = self.size_align(ty, line)?;
                Ok((Value::Int(if *is_size { size } else { align } as i128), Prov::Literal))
            }
            Expr::Builtin(head, op, args, _) if head == "ptr" && op == "read" && args.values.first().and_then(|a| self.ptr_elem(a)).and_then(|t| self.c_checked(&t)).is_some() => {
                let t = self.ptr_elem(&args.values[0]).unwrap();
                let n = self.c_checked(&t).unwrap();
                if self.trust_c_depth == 0 { return rt(line, format!("`ptr.read` of a `{n}` from C can fail (out_of_range); say what happens: `ptr.read[…] otherwise …`")); }
                let v = self.builtin(head, op, args, line)?;
                match self.from_c(v, &t) { Some(v) => { let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain }; Ok((v, prov)) }
                    None => site(line, format!("`NO_FUCKING_CHECKS` trusted C, and C sent no `{n}` here — the compiled program would be undefined")) }
            }
            Expr::Match(value, arms, _) => {
                let (v, _) = self.single(value, line)?;
                let (arm, data, dty) = self.pick_arm(v, arms, line)?;
                self.arm_ty = dty;
                match self.run_arm(arm, data, false)? {
                    Flow::Give(v) => { let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain }; Ok((v, prov)) }
                    Flow::Normal => rt(arm.line, format!("case `{}` neither gives a value nor leaves (return, fail, break, continue)", arm.name)),
                    f => Err(RuntimeError { line, msg: String::new(), exit: None, site: false, leave: Some(f) }),
                }
            }
            Expr::PtrCast(ty, args, line) => {
                let line = *line;
                if self.nocheck_depth == 0 && self.marks_needed() { return rt(line, "`ptr.cast` says the bytes are a different type, which the compiler cannot vouch for — it is a no_check site"); }
                if let Err(m) = self.c_shaped(ty) { return rt(line, format!("a pointer points at C-shaped data, so it casts only to a C-shaped type: {m}")); }
                let (pv, _) = self.value(&args.values[0])?;
                // the address and whether it may write stay as they were: a `ptr` stays read-only
                // (owner, 2026-09-30: a); the oracle's pointers carry no element type, so the rest is a relabel
                match pv { Value::Ptr(p) => Ok((Value::Ptr(p), Prov::Plain)), other => rt(line, format!("`ptr.cast` needs a pointer, got {}", other.kind())) }
            }
            Expr::PtrReadChecked(args, handler, _) => {
                let t = args.values.first().and_then(|a| self.ptr_elem(a));
                let Some(n) = t.as_ref().and_then(|t| self.c_checked(t)) else { return rt(line, "`ptr.read` of a plain type cannot fail, so it takes no `otherwise`") };
                let t = t.unwrap();
                let fails = vec![FailCase { name: "out_of_range".into(), ty: None }];
                self.check_handler(handler.as_ref(), &fails, "ptr.read", line)?;
                let v = self.builtin("ptr", "read", args, line)?;
                match self.from_c(v, &t) {
                    Some(v) => { let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain }; Ok((v, prov)) }
                    None => { let _ = n; let v = self.handle_failure(handler.as_ref().unwrap(), line, "ptr.read", "out_of_range".into(), None, true)?; let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain }; Ok((v, prov)) }
                }
            }
            Expr::Builtin(head, op, args, bline) => {
                self.one_lend_mut(args, *bline)?;
                let v = self.builtin(head, op, args, line)?;
                if matches!(v, Value::Nothing) { return rt(line, format!("`{head}.{op}` hands nothing back; it cannot be used as a value")); }
                let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                Ok((v, prov))
            }
            Expr::Paren(inner, _) => self.eval(inner),
            Expr::Unary(op, inner, _) => {
                let (v, prov) = self.eval(inner)?;
                let keep = if matches!(prov, Prov::Literal) { Prov::Literal } else { Prov::Plain };
                if op == "not.bit" {
                    // flip every bit, at the operand's declared width (the oracle needs a typed variable)
                    // a vector's lanes give the width, as for the wrapping operators
                    let width = match self.expr_ty(inner) { Some(Ty::Simd(e, _)) => Some(*e), w => w };
                    let (bits, signed) = match width { Some(Ty::Int(w)) => (true_bits(&w), true), Some(Ty::Uint(w)) => (true_bits(&w), false), _ => return rt(line, "`not.bit` flips a type's bits, so it needs to know how many: give the value a type, or say it in the declaration (`let 'x': uint32 = not.bit 'v';`)") };
                    // 127 bits and up, on big numbers (owner, 2026-10-03): not x is -x - 1, then taken at the width
                    if bits >= 127 || matches!(v, Value::Big(_)) {
                        let Some(x) = as_big(&v) else { return rt(line, format!("`not.bit` is integer arithmetic, got {}", v.kind())) };
                        let mut r = x.neg().sub(&crate::bigint::BigInt::from_i128(1)).mod_pow2(bits as u64);
                        if signed && r.cmp(&crate::bigint::BigInt::pow2(bits as u64 - 1)).is_ge() { r = r.sub(&crate::bigint::BigInt::pow2(bits as u64)); }
                        return Ok((from_big(r), Prov::Plain));
                    }
                    let x = match v { Value::Int(x) => x, o => return rt(line, format!("`not.bit` is integer arithmetic, got {}", o.kind())) };
                    let m: i128 = 1i128 << bits;
                    let mut r = (!x).rem_euclid(m);
                    if signed && r >= m / 2 { r -= m; }
                    return Ok((Value::Int(r), Prov::Plain));
                }
                match (op.as_str(), v) {
                    ("not", Value::Bool(b)) => Ok((Value::Bool(!b), Prov::Plain)),
                    // a mask, lane by lane (owner, 2026-09-27: `simd[bool, N]` is a type)
                    ("not", Value::Simd(ls)) if ls.iter().all(|l| matches!(l, Value::Bool(_))) =>
                        Ok((Value::Simd(ls.iter().map(|l| match l { Value::Bool(b) => Value::Bool(!b), o => o.clone() }).collect()), Prov::Plain)),
                    ("not", o) => rt(line, format!("`not` needs a bool, got {}", o.kind())),
                    ("-", Value::Int(i)) => Ok((match i.checked_neg() { Some(n) => Value::Int(n), None => from_big(crate::bigint::BigInt::from_i128(i).neg()) }, keep)),
                    ("-", Value::Big(b)) => Ok((from_big(b.neg()), keep)),
                    // plain negation never gives -0; `neg.ieee` flips the sign, so 0 becomes -0 (owner, 2026-10-07: "yea")
                    ("-", Value::Bin(b)) => Ok((Value::Bin(-b + 0.0), keep)),
                    ("-", Value::Brain(b)) => Ok((Value::Brain(-b + 0.0), keep)),
                    ("neg.ieee", Value::Bin(b)) => Ok((Value::Bin(-b), keep)),
                    ("neg.ieee", Value::Brain(b)) => Ok((Value::Brain(-b), keep)),
                    ("neg.ieee", o) => rt(line, format!("`neg.ieee` keeps a -0, which only a bin or brain16 has, and {} {} has none: write `-'x'`", if o.kind().starts_with(['a', 'e', 'i', 'o', 'u']) { "an" } else { "a" }, o.kind())),
                    ("-", Value::Deci(d)) => Ok((Value::Deci(d.neg()), keep)),
                    ("-", o) => rt(line, format!("`-` needs a number, got {}", o.kind())),
                    _ => unreachable!(),
                }
            }
            // Wrapping arithmetic (decided 2026-09-20): `+~ -~ x~ xx~` are always defined —
            // modulo the type's size. The oracle wraps at the left operand's declared
            // integer width (Cortex knows every width; the oracle only knows declared ones).
            // `div.trunc.wrap` and the other three (owner, 2026-10-03): the true quotient, then wrapped at
            // the type's width, as ARM gives the smallest value divided by -1
            Expr::Binary(op, l, r, _) if op.starts_with("div.") && op.ends_with(".wrap") => {
                let base = op.strip_suffix(".wrap").unwrap();
                let width = self.expr_ty(l).or_else(|| self.expr_ty(r));
                let (bits, signed) = match width {
                    Some(Ty::Int(w)) => (true_bits(&w), true),
                    Some(Ty::Uint(w)) => (true_bits(&w), false),
                    _ => return rt(line, format!("`{op}` wraps modulo a type's size, so it needs to know which: give a side a type")),
                };
                let (a, _) = self.eval(l)?;
                let (b, _) = self.eval(r)?;
                let exact = self.binary(base, a, b, line)?;
                let Some(v) = as_big(&exact) else { return rt(line, format!("`{op}` is integer arithmetic, got {}", exact.kind())) };
                let mut w = v.mod_pow2(bits as u64);
                if signed && w.cmp(&crate::bigint::BigInt::pow2(bits as u64 - 1)).is_ge() { w = w.sub(&crate::bigint::BigInt::pow2(bits as u64)); }
                Ok((from_big(w), Prov::Plain))
            }
            Expr::Binary(op, l, r, _) if matches!(op.as_str(), "+~" | "-~" | "x~" | "xx~") => {
                // Wrapping is at one size, and it is always written, never guessed: operands of two
                // sizes, or an exact sum that mixes them, need `to.int16[…]` (owner, 2026-09-23: b)
                // a power's exponent is a count, like a shift's, not a value being wrapped: only the
                // base's size counts for `xx~` (provisional, by analogy with shifts)
                let (sl, sr) = (self.plain_size(l), if op == "xx~" { Ok(None) } else { self.plain_size(r) });
                let mixed = match (&sl, &sr) {
                    (Err(m), _) | (_, Err(m)) => Some(m.clone()),
                    (Ok(Some(a)), Ok(Some(b))) if ty_name(a) != ty_name(b) => Some(format!("{} and {}", ty_name(a), ty_name(b))),
                    _ => None,
                };
                if let Some(m) = mixed { return rt(line, format!("`{op}` wraps at one size, and this mixes {m}: say which, `to.int16[…]` around a side")); }
                let width = if op == "xx~" { self.expr_ty(l) } else { self.expr_ty(l).or_else(|| self.expr_ty(r)) };
                // a vector wraps lane by lane at its element's size (simd.md; check claude, 2026-09-29)
                let width = match width { Some(Ty::Simd(e, _)) => Some(*e), w => w };
                let (bits, signed) = match width {
                    Some(Ty::Int(w)) => (true_bits(&w), true),
                    Some(Ty::Uint(w)) => (true_bits(&w), false),
                    // the width can come from a name, a conversion, either side, or the type
                    // stored into (2026-09-23); a bare literal with none of those has no size to wrap in
                    Some(t) => return rt(line, format!("`{op}` wraps modulo an integer's size, and {} is not an integer", ty_name(&t))),
                    None => return rt(line, format!("`{op}` wraps modulo a type's size, so it needs to know which: give a side a type, or say it in the declaration (`let 'x': uint64 = 'a' {op} 'b';`)")),
                };
                let (a, _) = self.eval(l)?;
                let (b, _) = self.eval(r)?;
                // at 127 bits and up (int128, uint128, the wide `!std_` ones) the wrap is worked out
                // on big numbers, exactly, then taken modulo 2^bits (owner, 2026-10-03)
                let big_wrap = |x: &Value, y: &Value| -> Result<Value, RuntimeError> {
                    let (x, y) = (as_big(x).unwrap(), as_big(y).unwrap());
                    let raw = match op.as_str() {
                        "+~" => x.add(&y), "-~" => x.sub(&y), "x~" => x.mul(&y),
                        _ => {
                            if y.is_neg() { return site(line, "a negative exponent on integers would produce a fraction — a check site: Cortex proves the exponent is never negative, or the final build refuses"); }
                            // square and multiply, taking the remainder at every step
                            let (mut base, mut e, mut acc) = (x.mod_pow2(bits as u64), y.clone(), crate::bigint::BigInt::from_i128(1));
                            let two = crate::bigint::BigInt::from_i128(2);
                            while !e.is_zero() {
                                if !e.mod_pow2(1).is_zero() { acc = acc.mul(&base).mod_pow2(bits as u64); }
                                e = e.divrem_trunc(&two).unwrap().0;
                                base = base.mul(&base).mod_pow2(bits as u64);
                            }
                            acc
                        }
                    };
                    let mut v = raw.mod_pow2(bits as u64);
                    if signed && v.cmp(&crate::bigint::BigInt::pow2(bits as u64 - 1)).is_ge() { v = v.sub(&crate::bigint::BigInt::pow2(bits as u64)); }
                    Ok(from_big(v))
                };
                let wrap = |x: i128, y: i128| -> Result<i128, RuntimeError> {
                    let raw = match op.as_str() {
                        "+~" => x.wrapping_add(y),
                        "-~" => x.wrapping_sub(y),
                        "x~" => x.wrapping_mul(y),
                        _ => { if y < 0 { return site(line, "a negative exponent on integers would produce a fraction — a check site: Cortex proves the exponent is never negative, or the final build refuses"); } let mut acc: i128 = 1; for _ in 0..y { acc = acc.wrapping_mul(x); } acc }
                    };
                    let m: i128 = 1i128 << bits;
                    let mut v = raw.rem_euclid(m);
                    if signed && v >= m / 2 { v -= m; }
                    Ok(v)
                };
                let lane = |v: &Value| match v { Value::Int(i) => Some(*i), _ => None };
                match (a, b) {
                    (a @ (Value::Int(_) | Value::Big(_)), b @ (Value::Int(_) | Value::Big(_))) if bits >= 127 || matches!(a, Value::Big(_)) || matches!(b, Value::Big(_)) => Ok((big_wrap(&a, &b)?, Prov::Plain)),
                    (Value::Int(x), Value::Int(y)) => Ok((Value::Int(wrap(x, y)?), Prov::Plain)),
                    (Value::Simd(xs), Value::Simd(ys)) if xs.len() == ys.len() => {
                        let mut out = Vec::with_capacity(xs.len());
                        for (p, q) in xs.iter().zip(&ys) { match (lane(p), lane(q)) { (Some(x), Some(y)) => out.push(Value::Int(wrap(x, y)?)), _ => return rt(line, format!("`{op}` is integer arithmetic, lane by lane")) } }
                        Ok((Value::Simd(out), Prov::Plain))
                    }
                    (Value::Simd(xs), Value::Int(y)) => { let mut out = Vec::new(); for p in &xs { match lane(p) { Some(x) => out.push(Value::Int(wrap(x, y)?)), None => return rt(line, format!("`{op}` is integer arithmetic, lane by lane")) } } Ok((Value::Simd(out), Prov::Plain)) }
                    (Value::Int(x), Value::Simd(ys)) => { let mut out = Vec::new(); for q in &ys { match lane(q) { Some(y) => out.push(Value::Int(wrap(x, y)?)), None => return rt(line, format!("`{op}` is integer arithmetic, lane by lane")) } } Ok((Value::Simd(out), Prov::Plain)) }
                    (a, b) => rt(line, format!("`{op}` is integer arithmetic, got {} and {}", a.kind(), b.kind())),
                }
            }
            // Bitwise (decided 2026-09-21): integers only; `shl` past the width is a check site.
            // `shl.masked` / `shr.masked` (owner, 2026-10-07): the count taken modulo the width, the bits pushed
            // past it dropped; no check site
            Expr::Binary(op, l, r, _) if op == "shl.masked" || op == "shr.masked" => {
                let (a, _) = self.eval(l)?;
                let (b, _) = self.eval(r)?;
                let width_ty = self.expr_ty(l);
                let Some(t @ (Ty::Int(w) | Ty::Uint(w))) = width_ty.as_ref() else { return rt(line, format!("`{op}` needs a width, and nothing here gives one: declare where the value goes, or write `to.uint32[…] {op} …`")) };
                let bits = true_bits(w);
                if bits >= 127 { return rt(line, format!("`{op}` on {} is not built in the oracle", ty_name(t))); }
                let (Value::Int(x), Value::Int(y)) = (a, b) else { return rt(line, format!("`{op}` works on integers")) };
                let c = y.rem_euclid(bits as i128) as u32;
                let m = 1i128 << bits;
                // the number as its bits at the width first: an unsigned `shr` brings in zeros, a signed one
                // copies the sign
                let x = if matches!(t, Ty::Int(_)) { let u = x.rem_euclid(m); if u >= m / 2 { u - m } else { u } } else { x.rem_euclid(m) };
                let v = if op == "shl.masked" { (x << c).rem_euclid(m) } else { x >> c };
                let v = if matches!(t, Ty::Int(_)) { let u = v.rem_euclid(m); if u >= m / 2 { u - m } else { u } } else { v.rem_euclid(m) };
                Ok((Value::Int(v), Prov::Plain))
            }
            Expr::Binary(op_full, l, r, _) if matches!(op_full.as_str(), "and.bit" | "or.bit" | "xor" | "shl" | "shr" | "and.bit.wrap" | "or.bit.wrap" | "xor.wrap") => {
                // `.wrap` takes each operand as its bits at the operator's width; without it an operand must
                // fit that width (owner, 2026-10-07: "yea", strict by default, `.wrap` the hardware way)
                let (op, wrap_bits) = match op_full.strip_suffix(".wrap") { Some(b) => (b.to_string(), true), None => (op_full.clone(), false) };
                let op = &op;
                let (a, _) = self.eval(l)?;
                let (b, _) = self.eval(r)?;
                // a vector is N of the same operation, lane by lane (design/simd.md)
                if matches!(a, Value::Simd(_)) || matches!(b, Value::Simd(_)) {
                    let v = self.simd_bitwise(op, a, b, line)?;
                    return Ok((v, Prov::Plain));
                }
                // past i128, or a shift on a type of 127 bits and up, on big numbers (owner, 2026-10-03)
                let width_ty = self.expr_ty(l);
                let tbits = match &width_ty { Some(Ty::Int(w)) | Some(Ty::Uint(w)) => Some(true_bits(w)), _ => None };
                if matches!(a, Value::Big(_)) || matches!(b, Value::Big(_)) || (op.starts_with("sh") && tbits.map_or(false, |t| t >= 127)) {
                    let (Some(x), Some(y)) = (as_big(&a), as_big(&b)) else { return rt(line, format!("`{op}` works on integers, got {} and {}", a.kind(), b.kind())) };
                    let v = match op.as_str() {
                        "and.bit" => x.bitop(&y, b'&'),
                        "or.bit" => x.bitop(&y, b'|'),
                        "xor" => x.bitop(&y, b'^'),
                        _ => {
                            if y.is_neg() { return rt(line, format!("`{op}` by a negative count")); }
                            let bits = tbits.unwrap_or(128) as u64;
                            let n = y.to_u64().unwrap_or(u64::MAX);
                            if n >= bits { return site(line, format!("`{op}` by {y} on a {bits}-bit value: every bit is lost")); }
                            if op == "shl" {
                                let r = x.shl(n);
                                if let Some((lo, hi)) = width_ty.as_ref().and_then(crate::value::big_range) { if r.cmp(&lo).is_lt() || r.cmp(&hi).is_gt() { return site(line, format!("`shl` by {n} overflows: {r} does not fit the {bits}-bit type")); } }
                                r
                            } else { x.shr_floor(n) }
                        }
                    };
                    return Ok((from_big(v), Prov::Plain));
                }
                let (mut x, mut y) = match (a, b) { (Value::Int(x), Value::Int(y)) => (x, y), (a, b) => return rt(line, format!("`{op}` works on integers, got {} and {}", a.kind(), b.kind())) };
                if !op.starts_with("sh") {
                    let wty = self.expr_ty_own(l).or_else(|| self.expr_ty_own(r)).or_else(|| self.expected.clone());
                    let wty = match wty { Some(Ty::Ranged(b, _, _)) => Some(*b), t => t };
                    if let (Some(t @ (Ty::Int(w) | Ty::Uint(w))), Some((lo, hi))) = (wty.as_ref(), wty.as_ref().and_then(crate::value::int_range)) {
                        let bits = true_bits(w);
                        if wrap_bits && bits < 127 {
                            let m = 1i128 << bits;
                            let fit = |v: i128| { let u = v.rem_euclid(m); if matches!(t, Ty::Int(_)) && u >= m / 2 { u - m } else { u } };
                            x = fit(x); y = fit(y);
                        } else if !wrap_bits {
                            for v in [x, y] {
                                if v < lo || v > hi { return site(line, format!("{v} does not fit {}: a bitwise operand is taken at the operator's width (`{op}.wrap` takes its bits instead)", ty_name(t))); }
                            }
                        }
                    }
                }
                let v = match op.as_str() {
                    "and.bit" => x & y,
                    "or.bit" => x | y,
                    "xor" => x ^ y,
                    "shl" | "shr" => {
                        if y < 0 { return rt(line, format!("`{op}` by a negative count")); }
                        let width_ty = self.expr_ty(l);
                        // `shl` pushes bits off the left end, so its width must be known (operators.md,
                        // owner 2026-09-23: from any operand or the context; context stops at a call)
                        if width_ty.is_none() {
                            return rt(line, format!("`{op}` needs a width, and nothing here gives one: declare where the value goes (`let 'v': uint32 = 1 {op} 3;`) or write `to.uint32[1] {op} 3`"));
                        }
                        let bits = match &width_ty { Some(Ty::Int(w)) | Some(Ty::Uint(w)) => int_bits(w), _ => 127 };
                        // the shifted number must fit its width first, as a bitwise operand must (owner,
                        // 2026-10-07: "`a`"); `.masked` takes its bits instead
                        if let Some((t, (lo, hi))) = width_ty.as_ref().and_then(|t| crate::value::int_range(t).map(|r| (t, r))) {
                            if x < lo || x > hi { return site(line, format!("{x} does not fit {}: a shifted number is taken at its width (`{op}.masked` takes its bits instead)", ty_name(t))); }
                        }
                        // compare before narrowing: `y as u32` would wrap a huge count to a small one (seed 1125)
                        if y >= bits as i128 {
                            let what = if width_ty.is_some() { format!("{} {bits}-bit value", if bits.to_string().starts_with('8') || bits == 11 || bits == 18 { "an" } else { "a" }) } else { "a value of any width".to_string() };
                            return site(line, format!("`{op}` by {y} on {what}: every bit is lost"));
                        }
                        if op == "shl" {
                            let r = x.checked_shl(y as u32).ok_or_else(|| RuntimeError { line, msg: "shift overflow".into(), exit: None, site: true, leave: None })?;
                            if let Some(t) = width_ty { if let Some((lo, hi)) = crate::value::int_range(&t) { if r < lo || r > hi { return site(line, format!("`shl` by {y} overflows: {r} does not fit the {bits}-bit type")); } } }
                            r
                        } else { x >> (y as u32) }
                    }
                    _ => unreachable!(),
                };
                Ok((Value::Int(v), Prov::Plain))
            }
            // `and.then` / `or.else` (owner, 2026-09-25): the right side is worked out only when
            // the left has not already decided the answer; plain `and`/`or` always do both
            Expr::Binary(op, l, r, _) if op == "and.then" || op == "or.else" => {
                let (a, _) = self.eval(l)?;
                let Value::Bool(a) = a else { return rt(line, format!("`{op}` works on bools, got {}", a.kind())) };
                if (op == "and.then" && !a) || (op == "or.else" && a) { return Ok((Value::Bool(a), Prov::Plain)); }
                let (b, _) = self.eval(r)?;
                let Value::Bool(b) = b else { return rt(line, format!("`{op}` works on bools, got {}", b.kind())) };
                Ok((Value::Bool(b), Prov::Plain))
            }
            Expr::Binary(op, l, r, _) if !is_div_mod(op.strip_suffix(".ieee").unwrap_or(op)) && !matches!(op.strip_suffix(".ieee").unwrap_or(op), "+" | "-" | "x" | "xx" | "++" | "==" | "!==" | "<" | ">" | "</==" | ">/==" | "and" | "or") => {
                self.custom_op(op, l, r, line)
            }
            Expr::Binary(full, l, r, _) => {
                // `plus.ieee` is `plus` keeping IEEE's -0 (owner, 2026-10-07)
                let ieee = full.ends_with(".ieee");
                let op = &full.strip_suffix(".ieee").unwrap_or(full).to_string();
                // A comparison's operands take their width from each other (2026-09-23): the
                // stored `bool` says nothing, but a typed side sizes a literal group on the other
                // (`'n' > (0 -~ 1)` wraps in uint64; verification session, seed 90).
                let cmp = matches!(op.as_str(), "==" | "!==" | "<" | ">" | "</==" | ">/==");
                let saved_expected = self.expected.clone();
                // beside a deci, a group of bare numbers takes the deci's width, as a literal does
                // (`'a' + (1.5 x 2)`; compiler session, 2026-09-29)
                let mut deci_set = false;
                if !cmp && self.deci_ctx().is_none() && (matches!(op.as_str(), "+" | "-" | "x") || is_div_mod(op)) {
                    let side = self.expr_ty(l).or_else(|| self.expr_ty(r));
                    if let Some(t @ Ty::Deci(_)) = side { self.expected = Some(t); deci_set = true; }
                }
                if cmp { self.expected = None; let other = self.expr_ty(r); self.expected = other; }
                let r_a = self.eval(l);
                let la = self.last_lit.take();
                if cmp { self.expected = saved_expected.clone(); }
                let (a, pa) = r_a?;
                // the language's operator, defined for a user type (2026-09-20)
                if matches!(a, Value::Struct(..) | Value::Enum(..)) {
                    let shape = shape_of(&a);
                    if self.prog.funcs.iter().any(|f| f.is_op && f.name == *full && f.params[0].ty.as_ref().map(|t| ty_shape(t) == shape).unwrap_or(false)) {
                        return self.custom_op_with(full, (a, pa), l, r, line);
                    }
                }
                if cmp { self.expected = None; let other = self.expr_ty(l); self.expected = other; }
                let r_b = self.eval(r);
                let lb = self.last_lit.take();
                if cmp || deci_set { self.expected = saved_expected; }
                let (b, pb) = r_b?;
                // a bin literal meeting a bin16/bin32 value in arithmetic is at that width, rounded
                // once from its digits (owner, 2026-09-24: bin math at its values' width)
                let (a, b) = if cmp || op == "++" { (a, b) } else {
                    let a = match (&a, &pa, &la, self.bin_width(r)) { (Value::Bin(_), Prov::Literal, Some(d), Some(w @ (16 | 32))) => Value::Bin(round_decimal(d, w)), _ => a };
                    let b = match (&b, &pb, &lb, self.bin_width(l)) { (Value::Bin(_), Prov::Literal, Some(d), Some(w @ (16 | 32))) => Value::Bin(round_decimal(d, w)), _ => b };
                    (a, b)
                };
                // A literal beside a deci is a deci of its width, from its digits (owner, 2026-09-29).
                let lit_deci = |me: &Self, v: &Value, digits: &Option<String>, w: u8| -> Result<Option<Value>, RuntimeError> {
                    Ok(match (v, digits) {
                        (Value::Int(i), _) => Some(Value::Deci(crate::deci::Deci::from_int(*i, w).or_else(|m| rt(line, format!("`{i}` {m}: write fewer digits, or use a wider deci")))?)),
                        (Value::Bin(_), Some(d)) => Some(Value::Deci(me.deci_literal(d, w, line)?)),
                        _ => None,
                    })
                };
                let (a, b) = match (&a, &b) {
                    (Value::Deci(d), Value::Int(_) | Value::Bin(_)) if matches!(pb, Prov::Literal) => match lit_deci(self, &b, &lb, d.width)? { Some(x) => (a.clone(), x), None => (a, b) },
                    (Value::Int(_) | Value::Bin(_), Value::Deci(d)) if matches!(pa, Prov::Literal) => match lit_deci(self, &a, &la, d.width)? { Some(x) => (x, b.clone()), None => (a, b) },
                    _ => (a, b),
                };
                // A bare literal beside a brain16 is a brain16, rounded to the nearest (owner, 2026-10-02)
                let lit_brain = |v: &Value, digits: &Option<String>| -> Option<Value> { match v {
                    Value::Int(i) => Some(Value::Brain(crate::value::round_brain(*i as f64))),
                    Value::Bin(x) => Some(Value::Brain(crate::value::round_brain(digits.as_ref().and_then(|d| d.parse::<f64>().ok()).unwrap_or(*x)))),
                    _ => None } };
                let (a, b) = match (&a, &b) {
                    (Value::Brain(_), Value::Int(_) | Value::Bin(_)) if matches!(pb, Prov::Literal) => (a.clone(), lit_brain(&b, &lb).unwrap()),
                    (Value::Int(_) | Value::Bin(_), Value::Brain(_)) if matches!(pa, Prov::Literal) => (lit_brain(&a, &la).unwrap(), b.clone()),
                    _ => (a, b),
                };
                // A bare literal adopts the other side's family.
                let (a, b) = match (&a, &b, &pa, &pb) {
                    (Value::Bin(_), Value::Int(i), _, Prov::Literal) => (a.clone(), Value::Bin(*i as f64)),
                    (Value::Int(i), Value::Bin(_), Prov::Literal, _) => (Value::Bin(*i as f64), b.clone()),
                    _ => (a, b),
                };
                if op == "++" {
                    // Text join (decided 2026-09-19): both sides text — nothing converts by
                    // itself; a number is spelled `to.str.utf8[…]` (later that day).
                    let is_text = |v: &Value| matches!(v, Value::Str(_) | Value::Char(..));
                    for (side, v) in [("left", &a), ("right", &b)] {
                        if !is_text(v) {
                            return rt(line, format!("`++` joins text, and its {side} side is {}; convert it first: `to.str.utf8[…]`", v.kind()));
                        }
                    }
                    return Ok((Value::Str(format!("{}{}", a.show_stored(), b.show_stored())), Prov::Fresh));
                }
                let both_lit = matches!(pa, Prov::Literal) && matches!(pb, Prov::Literal);
                // a deci power's exponent is an integer or a whole literal, never a deci variable: the
                // compiler cannot know it is whole (owner, 2026-09-29: a)
                if op == "xx" && matches!(a, Value::Deci(_)) && matches!(b, Value::Deci(_)) && !matches!(pb, Prov::Literal) {
                    return rt(line, "a deci power's exponent is an integer or a whole literal, not a deci that may hold a fraction: `'r' xx to.int32['k'] otherwise …`");
                }
                let divisor = if op.starts_with("mod.") { if let Value::Bin(y) = &b { Some(*y) } else { None } } else { None };
                let mut v = self.binary(full, a, b, line)?;
                // a bin16/bin32 remainder is rounded where it lands, and that can reach the divisor
                // again (-0.000001 mod.floor 360.0 is 360 in a bin32): still below it (owner, 2026-10-02: b)
                if let (Some(y), Value::Bin(rem)) = (divisor, &v) {
                    let rounded = match self.bin_width(l).or_else(|| self.bin_width(r)) { Some(32) => (*rem as f32) as f64, Some(16) => round_f16(*rem), _ => *rem };
                    if rounded.abs() == y.abs() { v = Value::Bin(below_divisor(rounded, y, &op[4..])); }
                }
                // a vector's lanes are checked at every operation, at the lane's type — not exact in
                // between as plain numbers are: wider lanes would halve what a register holds
                // (owner, 2026-09-29: b)
                if let Value::Simd(lanes) = &v {
                    if matches!(op.as_str(), "+" | "-" | "x" | "xx") {
                        let elem = match self.expr_ty(l).or_else(|| self.expr_ty(r)) { Some(Ty::Simd(e, _)) => Some(*e), _ => None };
                        if let Some(e) = elem {
                            for (k, lane) in lanes.iter().enumerate() {
                                if self.check_type(lane, &e, line).is_err() {
                                    return site(line, format!("lane {} of `{op}` is {}, which does not fit {}: a vector's lanes are checked at every operation", k + 1, lane.show_stored(), ty_name(&e)));
                                }
                            }
                        }
                    }
                }
                // bin16/bin32 math rounds after every operation, at its values' width, as the
                // hardware does (owner, 2026-09-24: a — "the programmer could've chose a better one")
                if let Value::Bin(x) = v {
                    let w = match (self.bin_width(l), self.bin_width(r)) {
                        (Some(a), Some(b)) => Some(a.max(b)),
                        (a, b) => a.or(b).or_else(|| match &self.expected { Some(Ty::Bin(w)) => Some(*w), Some(Ty::Ranged(b, _, _)) => match **b { Ty::Bin(w) => Some(w), _ => None }, _ => None }),
                    };
                    if let Some(w @ (16 | 32)) = w {
                        // `.ieee` rounds as IEEE does, an underflow keeping its sign
                        let r = if ieee { Value::Bin(if w == 32 { (x as f32) as f64 } else { round_f16(x) }) } else { round_to_width(&Ty::Bin(w), Value::Bin(x)) };
                        if let Value::Bin(y) = r {
                            if x.is_finite() && y.is_infinite() { return site(line, format!("{x} does not fit bin{w}: `{op}` rounds past the largest value")); }
                        }
                        return Ok((r, Prov::Plain));
                    }
                }
                // Plain integer math is exact, and checked where the value lands — a typed `let`,
                // an assignment, an argument, a return (owner, 2026-09-23: "the best perf &
                // safety"). In between, the one limit is the machine's 64 bits: a value no 64-bit
                // type holds is a check site. Bare numbers alone are worked out at compile time.
                // The limit in between is the widest operand's width, at least 64 bits (owner,
                // 2026-10-03: "Ah, okay"), so a `!std_uint256` sum works at 256 bits.
                if matches!(op.as_str(), "+" | "-" | "x" | "xx") && !both_lit {
                    let width = |e: &Expr| match self.expr_ty(e) { Some(Ty::Int(w) | Ty::Uint(w)) => true_bits(&w), Some(Ty::Ranged(b, ..)) => match *b { Ty::Int(w) | Ty::Uint(w) => true_bits(&w), _ => 0 }, _ => 0 };
                    let bits = 64.max(width(l)).max(width(r)) as u64;
                    let past = match &v {
                        Value::Int(n) if bits == 64 => *n < i64::MIN as i128 || *n > u64::MAX as i128,
                        Value::Int(_) if bits >= 128 => false,
                        Value::Int(_) | Value::Big(_) => {
                            let b = as_big(&v).unwrap();
                            b.cmp(&crate::bigint::BigInt::pow2(bits - 1).neg()).is_lt() || b.cmp(&crate::bigint::BigInt::pow2(bits).sub(&crate::bigint::BigInt::from_i128(1))).is_gt()
                        }
                        _ => false,
                    };
                    if past {
                        return site(line, format!("{} goes past {bits} bits in between — `{op}` is exact, but no {bits}-bit type holds this", v.show_stored()));
                    }
                }
                Ok((v, if both_lit { Prov::Literal } else { Prov::Plain }))
            }
        }
    }

    /// `div.X` / `mod.X` within one family (owner, 2026-10-02). On integers every `div` is whole; on
    /// bins, `brain16` and decis `floor`, `ceil` and `trunc` give a whole quotient and `near` is
    /// ordinary division, rounded to the nearest value the type holds, so `mod.near` is refused there
    /// (it leaves nothing over). `a == (a div.X b) x b + (a mod.X b)` for the whole ones.
    fn div_mod(&self, kind: &str, mode: &str, a: Value, b: Value, line: usize, ieee: bool) -> Result<Value, RuntimeError> {
        use Value::*;
        let op = format!("{kind}.{mode}");
        let zero = if kind == "div" { "division by zero".to_string() } else { "`mod` by zero".to_string() };
        let fractional = !matches!(a, Int(_));
        if fractional && kind == "mod" && mode == "near" {
            return rt(line, format!("`mod.near` on {}s leaves nothing over: `div.near` there is ordinary division (owner, 2026-10-02). For what a whole quotient leaves, `mod.floor`, `mod.ceil` or `mod.trunc`", a.kind()));
        }
        let ovf = || RuntimeError { line, msg: format!("integer overflow in `{op}`"), exit: None, site: true, leave: None };
        Ok(match (a, b) {
            (Int(_), Int(0)) => return site(line, zero),
            (Int(x), Int(y)) => {
                let (mut q, mut r) = (x.checked_div(y).ok_or_else(ovf)?, x.checked_rem(y).ok_or_else(ovf)?);
                if r != 0 {
                    let up = (r < 0) == (y < 0);   // the true quotient is above q
                    let away = match mode {
                        "floor" => !up, "ceil" => up, "trunc" => false,
                        _ => match (2 * r.unsigned_abs()).cmp(&y.unsigned_abs()) { std::cmp::Ordering::Greater => true, std::cmp::Ordering::Less => false, std::cmp::Ordering::Equal => q % 2 != 0 },
                    };
                    if away { if up { q += 1; r -= y; } else { q -= 1; r += y; } }
                }
                Int(if kind == "div" { q } else { r })
            }
            (Bin(_), Bin(y)) | (Brain(_), Brain(y)) if y == 0.0 => return site(line, zero),
            (Bin(x), Bin(y)) => {
                let v = if kind == "div" && mode == "near" { x / y } else { let (q, r) = float_div_mod(x, y, mode); if kind == "div" { q } else { r } };
                // a whole quotient that rounds to zero is 0, never -0 (owner, 2026-10-07: "`b`"): `+ 0.0`
                let v = if kind == "div" && mode != "near" && !ieee { v + 0.0 } else { v };
                if v.is_infinite() || v.is_nan() { return site(line, format!("`{op}` on {x:e} and {y:e} goes past the largest bin (it would be infinity): a bin never holds one")); }
                Bin(v)
            }
            (Brain(x), Brain(y)) => {
                let v = if kind == "div" && mode == "near" { ((x as f32) / (y as f32)) as f64 } else { let (q, r) = float_div_mod(x, y, mode); if kind == "div" { q } else { r } };
                let v = crate::value::round_brain(v);
                let v = if kind == "div" && mode != "near" && !ieee { v + 0.0 } else { v };
                // rounding to brain16 can reach the divisor again (359 rounds to 360): still below it
                let v = if kind == "mod" { below_divisor(v, y, mode) } else { v };
                if v.is_infinite() || v.is_nan() { return site(line, format!("`{op}` on {x:e} and {y:e} goes past the largest brain16 (it would be infinity)")); }
                Brain(v)
            }
            (Deci(x), Deci(y)) => {
                if kind == "div" && mode == "near" {
                    match x.div(&y) { None => return site(line, zero), Some(r) => Deci(r.or_else(|m| site(line, m))?) }
                } else {
                    match x.divmod_whole(&y, mode) { None => return site(line, zero), Some(r) => { let (q, r) = r.or_else(|m| site(line, m))?; Deci(if kind == "div" { q } else { r }) } }
                }
            }
            (a, b) => return rt(line, format!("`{op}` between {} and {}", a.kind(), b.kind())),
        })
    }

    /// No bin or brain16 result is ever -0 (owner, 2026-10-07: "`a`"); an operator's `.ieee` form keeps
    /// IEEE's -0 (owner, 2026-10-07: "yea"), `plus.ieee`, `times.ieee`, `div.near.ieee` and the rest.
    fn binary(&self, op: &str, a: Value, b: Value, line: usize) -> Result<Value, RuntimeError> {
        match op.strip_suffix(".ieee") {
            Some(base) => self.binary_at(base, a, b, line, true),
            None => Ok(no_neg_zero(self.binary_at(op, a, b, line, false)?)),
        }
    }

    fn binary_at(&self, op: &str, a: Value, b: Value, line: usize, ieee: bool) -> Result<Value, RuntimeError> {
        // A vector is N of what the operator already does (design/simd.md, 2026-09-22):
        // lane by lane, the scalar rule. A comparison gives a simd of bools, never one bool.
        match (&a, &b) {
            (Value::Simd(x), Value::Simd(y)) => {
                if x.len() != y.len() { return rt(line, format!("`{op}` between a simd of {} lanes and one of {}", x.len(), y.len())); }
                let mut out = Vec::with_capacity(x.len());
                for (l, r) in x.iter().zip(y) { out.push(self.binary_at(op, l.clone(), r.clone(), line, ieee)?); }
                return Ok(Value::Simd(out));
            }
            (Value::Simd(x), scalar) if matches!(scalar, Value::Int(_) | Value::Bin(_) | Value::Bool(_)) => {
                let mut out = Vec::with_capacity(x.len());
                for l in x { out.push(self.binary_at(op, l.clone(), scalar.clone(), line, ieee)?); }
                return Ok(Value::Simd(out));
            }
            (scalar, Value::Simd(y)) if matches!(scalar, Value::Int(_) | Value::Bin(_) | Value::Bool(_)) => {
                let mut out = Vec::with_capacity(y.len());
                for r in y { out.push(self.binary_at(op, scalar.clone(), r.clone(), line, ieee)?); }
                return Ok(Value::Simd(out));
            }
            _ => {}
        }
        use Value::*;
        let ovf = |what: &str| RuntimeError { line, msg: format!("integer overflow in `{what}`"), exit: None, site: true, leave: None };
        // integers past i128 (owner, 2026-10-03): when a value is past it, or a result would be, the
        // operation is worked out exactly on big numbers
        if matches!(a, Int(_) | Big(_)) && matches!(b, Int(_) | Big(_)) && (matches!(a, Big(_)) || matches!(b, Big(_)) || int_overflows(op, &a, &b)) {
            if let Some(v) = big_binary(op, &a, &b, line)? { return Ok(v); }
        }
        // `div.X` / `mod.X` (owner, 2026-10-02): every division says how it rounds, `floor`, `ceil`,
        // `trunc` or `near`, on every number family, and each `mod.X` is what `div.X` leaves over
        // `.ieee` keeps IEEE's -0 (owner, 2026-10-07): only a bin or brain16 has one
        if ieee && !matches!((&a, &b), (Bin(_), Bin(_)) | (Brain(_), Brain(_))) {
            return rt(line, format!("`{op}.ieee` keeps a -0, which only a bin or brain16 has, and {} {} has none: write `{op}`", if a.kind().starts_with(['a', 'e', 'i', 'o', 'u']) { "an" } else { "a" }, a.kind()));
        }
        if let Some((kind, mode)) = op.split_once('.') {
            if (kind == "div" || kind == "mod") && matches!(mode, "floor" | "ceil" | "trunc" | "near") {
                let same = matches!((&a, &b), (Int(_), Int(_)) | (Bin(_), Bin(_)) | (Brain(_), Brain(_))) || matches!((&a, &b), (Deci(x), Deci(y)) if x.width == y.width);
                if same { return self.div_mod(kind, mode, a, b, line, ieee); }
            }
        }
        Ok(match (op, a, b) {
            ("+", Int(x), Int(y)) => Int(x.checked_add(y).ok_or_else(|| ovf("+"))?),
            ("-", Int(x), Int(y)) => Int(x.checked_sub(y).ok_or_else(|| ovf("-"))?),
            ("x", Int(x), Int(y)) => Int(x.checked_mul(y).ok_or_else(|| ovf("x"))?),
            ("xx", Int(x), Int(y)) => {
                if y < 0 { return site(line, "a negative exponent on integers would produce a fraction — a check site: Cortex proves the exponent is never negative, or the final build refuses"); }
                let mut acc: i128 = 1;
                for _ in 0..y { acc = acc.checked_mul(x).ok_or_else(|| ovf("xx"))?; }
                Int(acc)
            }
            // a bin never holds an infinity or a NaN (owner, 2026-09-29: a): a result that would be
            // one is a check site, as integer overflow and division by zero are
            ("+" | "-" | "x" | "xx", Bin(x), Bin(y)) => {
                let r = match op { "+" => x + y, "-" => x - y, "x" => x * y, _ => x.powf(y) };
                if r.is_nan() { return site(line, format!("`{op}` on {x:e} and {y:e} has no number as its answer (NaN): a bin never holds one")); }
                if r.is_infinite() { return site(line, format!("`{op}` on {x:e} and {y:e} goes past the largest bin (it would be infinity): a bin never holds one")); }
                Bin(r)
            }
            // brain16 (owner, 2026-10-02): each operation is done at 32 bits and rounded to the
            // nearest brain16, as LLVM emits it (widen, operate, `bfcvt`); its own family
            ("+" | "-" | "x" | "xx", Brain(x), Brain(y)) => {
                let (a, b) = (x as f32, y as f32);
                let r32 = match op { "+" => a + b, "-" => a - b, "x" => a * b, _ => a.powf(b) };
                let r = crate::value::round_brain(r32 as f64);
                if r.is_nan() { return site(line, format!("`{op}` on {x:e} and {y:e} has no number as its answer (NaN): a brain16 never holds one")); }
                if r.is_infinite() { return site(line, format!("`{op}` on {x:e} and {y:e} goes past the largest brain16 (it would be infinity)")); }
                Brain(r)
            }
            ("<", Brain(x), Brain(y)) => Bool(x < y),
            (">", Brain(x), Brain(y)) => Bool(x > y),
            ("</==", Brain(x), Brain(y)) => Bool(x <= y),
            (">/==", Brain(x), Brain(y)) => Bool(x >= y),
            (_, x, y) if (matches!(x, Brain(_)) || matches!(y, Brain(_))) && !matches!(op, "==" | "!==") =>
                return rt(line, format!("`{op}` between {} and {}: the brain float is a family of its own; convert one, `to.brain16[…]` or `to.bin32[…]`", x.kind(), y.kind())),
            // deci (owner, 2026-09-29): exact, then rounded to the width, half to even
            (_, Deci(x), Deci(y)) if x.width != y.width => return rt(line, format!("`{op}` between a deci{} and a deci{}: say which width, `to.deci{}[…]`", x.width, y.width, x.width.max(y.width))),
            ("+", Deci(x), Deci(y)) => Deci(x.add(&y).or_else(|m| site(line, m))?),
            ("-", Deci(x), Deci(y)) => Deci(x.sub(&y).or_else(|m| site(line, m))?),
            ("x", Deci(x), Deci(y)) => Deci(x.mul(&y).or_else(|m| site(line, m))?),
            ("==", Deci(x), Deci(y)) => Bool(x.cmp_value(&y).is_eq()),
            ("!==", Deci(x), Deci(y)) => Bool(!x.cmp_value(&y).is_eq()),
            ("<", Deci(x), Deci(y)) => Bool(x.cmp_value(&y).is_lt()),
            (">", Deci(x), Deci(y)) => Bool(x.cmp_value(&y).is_gt()),
            ("</==", Deci(x), Deci(y)) => Bool(x.cmp_value(&y).is_le()),
            (">/==", Deci(x), Deci(y)) => Bool(x.cmp_value(&y).is_ge()),
            // `mod` takes the divisor's sign, as integer `mod`; `xx` takes a whole exponent and rounds
            // like the other operations (owner, 2026-09-29: a, a)
            ("xx", Deci(x), Deci(y)) => {
                let n = match y.to_int() { Some(Ok(n)) => n, _ => return rt(line, format!("a deci power takes a whole exponent, not {}: a fractional power is cord's math", y.show_stored())) };
                match x.pow_int(n) { None => return site(line, "zero to a negative power: division by zero"), Some(r) => Deci(r.or_else(|m| site(line, m))?) }
            }
            ("xx", Deci(x), Int(n)) => match x.pow_int(n) { None => return site(line, "zero to a negative power: division by zero"), Some(r) => Deci(r.or_else(|m| site(line, m))?) },
            (_, Deci(_), Int(_) | Bin(_)) | (_, Int(_) | Bin(_), Deci(_)) =>
                return rt(line, format!("`{op}` between a deci and another number family: there is no implicit conversion between numeric families; `to.deci64[…]`")),
            (_, Int(_), Bin(_)) | (_, Bin(_), Int(_)) if matches!(op, "+" | "-" | "x" | "xx") || op.starts_with("div.") || op.starts_with("mod.") =>
                return rt(line, format!("`{op}` between an integer and a bin: there is no implicit conversion between numeric families")),
            // a char compares by its content, whatever unit it was declared in
            // durations (design/concurrency.md): compare, add, subtract, scale by an integer
            ("==", Duration(a), Duration(b)) => Bool(a == b),
            ("!==", Duration(a), Duration(b)) => Bool(a != b),
            ("<", Duration(a), Duration(b)) => Bool(a < b),
            (">", Duration(a), Duration(b)) => Bool(a > b),
            ("</==", Duration(a), Duration(b)) => Bool(a <= b),
            (">/==", Duration(a), Duration(b)) => Bool(a >= b),
            ("+", Duration(a), Duration(b)) => Duration(a + b),
            ("-", Duration(a), Duration(b)) => { if b > a { return rt(line, "a duration is never negative"); } Duration(a - b) }
            ("x", Duration(a), Int(k)) | ("x", Int(k), Duration(a)) => { if k < 0 { return rt(line, "a duration is never negative"); } Duration(a * k) }
            ("==", Char(_, a), Char(_, b)) => Bool(a == b),
            ("!==", Char(_, a), Char(_, b)) => Bool(a != b),
            ("==", x, y) => { same_family(&x, &y, line, "==")?; Bool(x == y) }
            ("!==", x, y) => { same_family(&x, &y, line, "!==")?; Bool(x != y) }
            ("<", Int(x), Int(y)) => Bool(x < y),
            (">", Int(x), Int(y)) => Bool(x > y),
            ("</==", Int(x), Int(y)) => Bool(x <= y),
            (">/==", Int(x), Int(y)) => Bool(x >= y),
            ("<", Bin(x), Bin(y)) => Bool(x < y),
            (">", Bin(x), Bin(y)) => Bool(x > y),
            ("</==", Bin(x), Bin(y)) => Bool(x <= y),
            (">/==", Bin(x), Bin(y)) => Bool(x >= y),
            ("<" | ">" | "</==" | ">/==", Str(_), Str(_)) => return rt(line, format!("`{op}` on text has no meaning yet: byte order is never what anyone means; a collation rule is cord's (decided 2026-09-20)")),
            ("and", Bool(x), Bool(y)) => Bool(x && y),
            ("or", Bool(x), Bool(y)) => Bool(x || y),
            (op, x, y) => return rt(line, format!("`{op}` cannot combine {} and {}", x.kind(), y.kind())),
        })
    }

    /// The static type of a single-expression bracket, for the few expressions the
    /// oracle knows one for: `list.length[…]` / `array.length[…]` are `uint64`
    /// (decided 2026-09-21). Everything else: unknown (a provisional gap, see README).
    /// Is the length of what this bracket produces known at this point? A literal list,
    /// `list.empty`, or a value of a bounded type `list[T][lo, hi]` — the cases Cortex
    /// always knows. (It would know more; the oracle keeps to these.)
    fn sized_from(value: &Bracket, ty: Option<&Ty>) -> bool {
        if matches!(ty, Some(Ty::Ranged(b, _, _)) if matches!(**b, Ty::List(_))) { return true; }
        if value.values.len() > 1 || value.one_list { return true; }
        matches!(value.values.as_slice(), [crate::ast::Value::Expr(Expr::Builtin(h, op, _, _))] if h == "list" && op == "empty")
    }

    fn known_ty(&self, value: &Bracket) -> Option<Ty> {
        match value.values.as_slice() {
            [crate::ast::Value::Expr(Expr::Builtin(head, op, args, _))] if (head == "list" || head == "array" || head == "map") && op == "length" => {
                let u64 = Ty::Uint(crate::ast::IntWidth::W64);
                // the length of a `list[T][lo, hi]` is a `uint64[lo, hi]`
                if let Some(crate::ast::Value::Expr(Expr::Owned(_, n, _))) = args.values.first() {
                    if let Some(Ty::Ranged(_, lo, hi)) = self.lookup(n).and_then(|s| s.ty.clone()) { return Some(Ty::Ranged(Box::new(u64), lo, hi)); }
                }
                Some(u64)
            }
            // `let 'a' := to.array[give 'l']` is an array of what 'l' held
            // `'y' := to.deci32['x'] otherwise 0` is a deci32, whatever its fallback (check claude, 2026-09-30)
            [crate::ast::Value::Expr(Expr::Convert(t @ (Ty::Int(_) | Ty::Uint(_) | Ty::Bin(_) | Ty::Deci(_)), _, _, _, _))] => Some(t.clone()),
            [crate::ast::Value::Expr(Expr::Convert(t @ (Ty::List(_) | Ty::Array(_)), b, _, _, _))] => {
                let elem = match b.values.first() {
                    Some(crate::ast::Value::Expr(Expr::Owned(_, n, _) | Expr::Var(n, _))) => match self.lookup(n).and_then(|s| s.ty.clone()) {
                        Some(Ty::List(e) | Ty::Array(e)) => Some(e),
                        Some(Ty::Ranged(b, _, _)) => match *b { Ty::List(e) | Ty::Array(e) => Some(e), _ => None },
                        _ => None,
                    },
                    _ => None,
                };
                let elem = elem.unwrap_or_else(|| Box::new(Ty::Var(String::new())));
                Some(if matches!(t, Ty::Array(_)) { Ty::Array(elem) } else { Ty::List(elem) })
            }
            _ => None,
        }
    }

    // ---- list / array operations (decided 2026-09-17) ---------------------

    fn builtin(&mut self, head: &str, op: &str, args: &Bracket, line: usize) -> Result<Value, RuntimeError> {
        if head == "ptr" { return self.ptr_op(op, args, line); }
        if head == "char" {
            // char.code[c] (decided 2026-09-21): the number under a point or a byte
            let (v, _) = self.value(&args.values[0])?;
            return match v {
                Value::Char(CharUnit::Grapheme, g) => rt(line, format!("`{g}` is a grapheme, which has no single code; its points: `str.each.point[lend to.str.utf8[…]]`, or `to.char.point[…] otherwise …` if it is one")),
                Value::Char(_, c) => Ok(Value::Int(c.chars().next().map(|ch| ch as i128).unwrap_or(0))),
                other => rt(line, format!("`char.code` takes a character, got {}", other.kind())),
            };
        }
        if head == "str" {
            // The units of a text (decided 2026-09-20): no single length, no bare walk.
            let (v, prov) = self.value(&args.values[0])?;
            let s = match v { Value::Str(s) => s, Value::Char(_, c) => c, other => return rt(line, format!("`str.{op}` needs a text, got {}", other.kind())) };
            if let Prov::Bare(n) = prov { return rt(line, format!("`str.{op}` looks at '{n}': `lend '{n}'`")); }
            return Ok(match op {
                "bytes" => Value::Int(s.len() as i128),
                "points" => Value::Int(s.chars().count() as i128),
                "graphemes" => Value::Int(grapheme_count(&s) as i128),
                "each.byte" => Value::List(s.bytes().map(|b| Value::Int(b as i128)).collect()),
                "each.point" => Value::List(s.chars().map(|c| Value::Char(CharUnit::Point, c.to_string())).collect()),
                "each.grapheme" => Value::List(graphemes(&s).into_iter().map(|g| Value::Char(CharUnit::Grapheme, g)).collect()),
                _ => unreachable!(),
            });
        }
        if head == "map" { return self.map_op(op, args, line); }
        if head == "channel" { return self.chan_send(args, line); }
        if head == "simd" { return self.simd_op(op, args, line); }
        if op == "empty" { return Ok(Value::List(Vec::new())); }
        // `list.fill[v, n]`: n copies of v (owner, 2026-09-30); a count below 0 is a check site
        if op == "fill" {
            let elem = match self.expected.take() {
                Some(Ty::List(t)) | Some(Ty::Array(t)) => Some(*t),
                Some(Ty::Ranged(b, _, _)) => match *b { Ty::List(t) | Ty::Array(t) => Some(t.as_ref().clone()), _ => None },
                _ => None,
            };
            let saved = std::mem::replace(&mut self.expected, elem.clone());
            let r = self.value(&args.values[0]);
            self.expected = None;
            let (v, prov) = match r { Ok(x) => x, Err(e) => { self.expected = saved; return Err(e); } };
            if let Prov::Bare(n) = &prov { if v.is_owned() { self.expected = saved; return rt(line, format!("`list.fill` makes copies of '{n}': `copy '{n}'`")); } }
            let v = elem_literal(elem.as_ref(), v, &prov);
            let counted = self.value(&args.values[1]);
            self.expected = saved;
            let n = match counted?.0 { Value::Int(n) => n, other => return rt(line, format!("`list.fill`'s count is an integer, got {}", other.kind())) };
            if n < 0 { return site(line, format!("`list.fill` of {n} copies: a count is 0 or more")); }
            return Ok(Value::List(vec![v; n as usize]));
        }
        if op == "slice" {
            // A slice owns its elements, as a list always does (owner, 2026-09-23): the copy is
            // the meaning, and Cortex elides it where nobody can tell. 1-based and inclusive,
            // like `[pos i]` and `range[…]`; a check site, like every language operation.
            if args.values.len() != 3 { return rt(line, format!("`{head}.slice` takes a list and two positions: `{head}.slice[lend 'xs', pos 2, pos 4]`")); }
            let (v, prov) = self.value(&args.values[0])?;
            let items = match &v { Value::List(i) => i.clone(), other => return rt(line, format!("`{head}.slice` needs a list or array, got {}", other.kind())) };
            if let Prov::Bare(n) = &prov { return rt(line, format!("`{head}.slice` takes the list by `lend '{n}'`")); }
            let mut ends = Vec::new();
            for a in &args.values[1..] {
                match self.value(a)?.0 { Value::Int(i) => ends.push(i), other => return rt(line, format!("a position is an integer, got {}", other.kind())) }
            }
            let (from, to) = (ends[0], ends[1]);
            if from < 1 || to > items.len() as i128 {
                return site(line, format!("`{head}.slice` from {from} to {to} runs outside a {} of {} element(s) (1 to {})", head, items.len(), items.len()));
            }
            if to < from - 1 { return site(line, format!("`{head}.slice` from {from} to {to} runs backwards; an empty slice is `pos n, pos n - 1`")); }
            return Ok(Value::List(items[(from - 1) as usize..to as usize].to_vec()));
        }
        let want = match op { "length" => 1, "append" => 2, "remove" => 2, _ => unreachable!() };
        if args.values.len() != want { return rt(line, format!("`{head}.{op}` takes {want} argument(s), given {}", args.values.len())); }
        let (v, prov) = self.value(&args.values[0])?;
        let items = match &v { Value::List(items) => items.clone(), other => return rt(line, format!("`{head}.{op}` needs a list or array, got {}", other.kind())) };
        // Which variable is this, and is it typed `array`?
        let (var_name, is_array, cell) = match &prov {
            Prov::Borrowed(cell, _) => {
                let name = match &args.values[0] { crate::ast::Value::Expr(Expr::Owned(_, n, _)) => n.clone(), _ => String::new() };
                let is_array = self.lookup(&name).map(|s| matches!(s.ty, Some(Ty::Array(_)))).unwrap_or(false);
                (name, is_array, Some(cell.clone()))
            }
            Prov::Bare(n) => return rt(line, format!("`{head}.{op}` takes the list by `lend '{n}'` (or `lend.mut` to change it)")),
            _ => (String::new(), false, None),
        };
        match op {
            "empty" => Ok(Value::List(Vec::new())),
            "length" => Ok(Value::Int(items.len() as i128)),
            "append" | "remove" => {
                if head == "array" { unreachable!() }
                if is_array { return rt(line, format!("'{var_name}' is an `array`, which cannot grow or shrink; declare it a `list`")); }
                let is_mut = matches!(prov, Prov::Borrowed(_, true));
                if !is_mut { return rt(line, format!("`list.{op}` changes the list: pass `lend.mut '{var_name}'`")); }
                let Some(cell) = cell else { return rt(line, "`list.append`/`remove` need a variable to change") };
                // the appended value lands in an element slot: the element type is its context (owner, 2026-09-29: a)
                let elem = if op == "append" { match self.lookup(&var_name).and_then(|s| s.ty.clone()) {
                    Some(Ty::List(t)) => Some(*t), Some(Ty::Ranged(b, _, _)) => match *b { Ty::List(t) => Some(*t), _ => None }, _ => None,
                } } else { None };
                let saved = std::mem::replace(&mut self.expected, elem);
                let r = self.value(&args.values[1]);
                self.expected = saved;
                let (arg, aprov) = r?;
                let mut items = items;
                // `list.remove` hands the removed element back (decided 2026-09-21): the list
                // stops owning it, so the caller does — or drops it, as a bare statement.
                let mut removed: Option<Value> = None;
                if op == "append" {
                    match aprov {
                        Prov::Bare(n) => return rt(line, format!("appending bare '{n}': say `give '{n}'` or `copy '{n}'`")),
                        Prov::Borrowed(..) => return rt(line, "a list owns its elements; append a `copy` or `give`, not a lend"),
                        _ => {}
                    }
                    let slot_ty = self.lookup(&var_name).and_then(|s| s.ty.clone());
                    let inner = match &slot_ty { Some(Ty::List(inner)) => Some(inner.clone()), Some(Ty::Ranged(b, _, _)) => match &**b { Ty::List(inner) => Some(inner.clone()), _ => None }, _ => None };
                    if let Some(inner) = inner { self.check_type(&arg, &inner, line)?; }
                    items.push(arg);
                } else {
                    let i = match arg { Value::Int(i) => i, other => return rt(line, format!("`list.remove` takes a 1-based position, got {}", other.kind())) };
                    if i < 1 || i as usize > items.len() { return site(line, format!("position {i} is out of range for '{var_name}', which has {} element(s)", items.len())); }
                    removed = Some(items.remove((i - 1) as usize));
                }
                // A length-bounded list (`list[T][lo, hi]`) is checked after every change (decided 2026-09-21).
                let new_len = Value::List(items);
                if let Some(t @ Ty::Ranged(..)) = self.lookup(&var_name).and_then(|s| s.ty.clone()) { self.check_type(&new_len, &t, line)?; }
                else if let Some(s) = self.lookup_mut(&var_name) { s.sized = false; }   // Cortex would track it; the oracle does not
                *cell.borrow_mut() = new_len;
                Ok(removed.unwrap_or(Value::Nothing))
            }
            _ => unreachable!(),
        }
    }

    // ---- map (decided 2026-09-22) -------------------------------------------
    // A language collection beside list and array. `map.get`/`map.remove` fail `missing`;
    // keys compare by structure, or by a type's own `op ==` / `fn hash` when it satisfies
    // `Key` — and then "equal keys hash the same" is a check site (the oracle tests it on
    // every comparison it makes; Cortex proves it).

    fn key_ok(&self, k: &Value, line: usize) -> Result<(), RuntimeError> {
        match k {
            Value::Func(_) | Value::Ptr(_) | Value::Map(_) | Value::Nothing => rt(line, format!("a {} cannot be a map key", k.kind())),
            Value::List(items) => { for it in items { self.key_ok(it, line)?; } Ok(()) }
            _ => Ok(()),
        }
    }

    /// Does this key's type bring its own `op ==` and `fn hash` (satisfies `Key`)?
    fn custom_key(&self, k: &Value) -> bool {
        let shape = shape_of(k);
        matches!(k, Value::Struct(..) | Value::Enum(..))
            && self.prog.funcs.iter().any(|f| f.is_op && f.name == "==" && f.params[0].ty.as_ref().map(|t| ty_shape(t) == shape).unwrap_or(false))
            && self.prog.funcs.iter().any(|f| !f.is_op && f.name == "hash" && f.params.len() == 1 && f.params[0].ty.as_ref().map(|t| ty_shape(t) == shape).unwrap_or(false))
    }

    fn call_named(&mut self, name: &str, shape: &str, arg: Value, line: usize) -> Result<Value, RuntimeError> {
        let def = self.prog.funcs.iter().find(|f| !f.is_op && f.name == name && f.params.len() == 1 && f.params[0].ty.as_ref().map(|t| ty_shape(t) == shape).unwrap_or(false)).cloned().unwrap();
        let kind = match def.params[0].kind { ParamKind::Owned => SlotKind::Owned, ParamKind::Lent => SlotKind::Lent, ParamKind::LentMut => SlotKind::LentMut };
        let bound = vec![(def.params[0].name.clone(), Slot { cell: Rc::new(RefCell::new(arg)), ty: def.params[0].ty.clone(), kind, alive: true, immut: false, sized: false, sealed: false , since: 0, deps: Vec::new() })];
        match self.run_body(&def, bound)? { Flow::Return(v) => Ok(v), _ => rt(line, format!("`{name}` returned nothing")) }
    }

    fn keys_equal(&mut self, a: &Value, b: &Value, line: usize) -> Result<bool, RuntimeError> {
        // plain values by value (maps.md): a deci 1.50 and 1.5 are one key (compiler session, 2026-09-29)
        if !self.custom_key(a) { return Ok(value_eq(a, b)); }
        let shape = shape_of(a);
        // bind both keys as temporaries so the user's `op ==` can be applied to them
        let mut scope = HashMap::new();
        scope.insert("__key_b".to_string(), Slot { cell: Rc::new(RefCell::new(b.clone())), ty: None, kind: SlotKind::Owned, alive: true, immut: true, sized: false, sealed: false , since: 0, deps: Vec::new() });
        self.scopes.push(scope);
        let ax = Expr::Var("__key_a".into(), line); let bx = Expr::Var("__key_b".into(), line);
        let r = self.custom_op_with("==", (a.clone(), Prov::Plain), &ax, &bx, line);
        self.scopes.pop();
        let (eq, _) = r?;
        let eq = match eq { Value::Bool(x) => x, other => return rt(line, format!("`op ==` for {shape} must return a bool, got {}", other.kind())) };
        if eq {
            // the consistency site: equal keys must hash the same (decided 2026-09-22)
            let ha = self.call_named("hash", &shape, a.clone(), line)?;
            let hb = self.call_named("hash", &shape, b.clone(), line)?;
            if ha != hb { return site(line, format!("`op ==` says two {shape} keys are equal but `hash` gives {} and {}; a map cannot trust this key type (prove or `no_check`)", ha.show_useful(), hb.show_useful())); }
        }
        Ok(eq)
    }

    fn map_arg(&mut self, args: &Bracket, want: usize, op: &str, line: usize) -> Result<(Rc<RefCell<Value>>, bool, Vec<Value>), RuntimeError> {
        if args.values.len() != want { return rt(line, format!("`map.{op}` takes {want} argument(s), given {}", args.values.len())); }
        let (v, prov) = self.value(&args.values[0])?;
        let (cell, is_mut) = match prov {
            Prov::Borrowed(cell, m) => (cell, m),
            Prov::Bare(n) => return rt(line, format!("`map.{op}` takes the map by `lend '{n}'` (or `lend.mut` to change it)")),
            _ => (Rc::new(RefCell::new(v.clone())), true),
        };
        if !matches!(&*cell.borrow(), Value::Map(_)) { return rt(line, format!("`map.{op}` needs a map, got {}", v.kind())); }
        // the key and the value land in the map's key and value types, which are their context, as a
        // parameter's type is an argument's (owner, 2026-09-29: a)
        let kv = match &args.values[0] {
            crate::ast::Value::Expr(Expr::Owned(_, n, _)) => match self.lookup(n).and_then(|s| s.ty.clone()) {
                Some(Ty::Map(k, v, _)) => Some((*k, *v)),
                Some(Ty::Ranged(b, _, _)) => match *b { Ty::Map(k, v, _) => Some((*k, *v)), _ => None },
                _ => None,
            },
            _ => None,
        };
        let mut rest = Vec::new();
        for (i, a) in args.values[1..].iter().enumerate() {
            let ctx = kv.as_ref().map(|(k, v)| if i == 0 { k.clone() } else { v.clone() });
            let saved = std::mem::replace(&mut self.expected, ctx);
            let r = self.value(a);
            self.expected = saved;
            let (v, prov) = r?;
            match prov {
                Prov::Bare(n) => return rt(line, format!("bare '{n}' into a map: say `give '{n}'` or `copy '{n}'`")),
                Prov::Borrowed(..) => return rt(line, "a map owns its keys and values; store a `copy` or `give`, not a lend"),
                _ => {}
            }
            rest.push(v);
        }
        Ok((cell, is_mut, rest))
    }

    fn map_op(&mut self, op: &str, args: &Bracket, line: usize) -> Result<Value, RuntimeError> {
        match op {
            "empty" => Ok(Value::Map(Vec::new())),
            "length" => { let (cell, _, _) = self.map_arg(args, 1, op, line)?; let n = match &*cell.borrow() { Value::Map(ps) => ps.len(), _ => 0 }; Ok(Value::Int(n as i128)) }
            "keys" | "values" | "pairs" => {
                // walking a map says which of the three (decided 2026-09-22): keys, values, or pairs
                // (a struct with `key` and `value`), in insertion order; a `map.unordered` cannot be walked.
                let (cell, _, _) = self.map_arg(args, 1, op, line)?;
                if let crate::ast::Value::Expr(Expr::Owned(_, n, _)) = &args.values[0] {
                    let t = self.lookup(n).and_then(|s| s.ty.clone());
                    let t = match t { Some(Ty::Ranged(b, _, _)) => Some(*b), t => t };
                    if let Some(Ty::Map(_, _, MapOrder::Unordered)) = t {
                        return rt(line, format!("'{n}' is a `map.unordered`, which promises never to be walked; `map.{op}` needs a `map.ordered` (or a plain `map`, and Cortex keeps the order)"));
                    }
                }
                let pairs = match &*cell.borrow() { Value::Map(ps) => ps.clone(), _ => Vec::new() };
                Ok(Value::List(pairs.into_iter().map(|(k, v)| match op {
                    "keys" => k, "values" => v,
                    _ => Value::Struct("map.pair".into(), vec![("key".into(), k), ("value".into(), v)]),
                }).collect()))
            }
            "has" => {
                let (cell, _, rest) = self.map_arg(args, 2, op, line)?;
                self.key_ok(&rest[0], line)?;
                let pairs = match &*cell.borrow() { Value::Map(ps) => ps.clone(), _ => Vec::new() };
                for (k, _) in &pairs { if self.keys_equal(k, &rest[0], line)? { return Ok(Value::Bool(true)); } }
                Ok(Value::Bool(false))
            }
            "set" => {
                let (cell, is_mut, rest) = self.map_arg(args, 3, op, line)?;
                if !is_mut { return rt(line, "`map.set` changes the map: pass `lend.mut`"); }
                self.key_ok(&rest[0], line)?;
                // the key and the value against the map's declared types (through a count bound)
                if let crate::ast::Value::Expr(Expr::Owned(_, n, _)) = &args.values[0] {
                    let t = self.lookup(n).and_then(|s| s.ty.clone());
                    let t = match t { Some(Ty::Ranged(b, _, _)) => Some(*b), t => t };
                    if let Some(Ty::Map(kt, vt, _)) = t { self.check_type(&rest[0], &kt, line)?; self.check_type(&rest[1], &vt, line)?; }
                }
                let mut pairs = match &*cell.borrow() { Value::Map(ps) => ps.clone(), _ => Vec::new() };
                let mut replaced = false;
                for (k, v) in pairs.iter_mut() { if self.keys_equal(k, &rest[0], line)? { *v = rest[1].clone(); replaced = true; break; } }
                if !replaced { pairs.push((rest[0].clone(), rest[1].clone())); }
                let new = Value::Map(pairs);
                if let crate::ast::Value::Expr(Expr::Owned(_, n, _)) = &args.values[0] {
                    if let Some(t @ Ty::Ranged(..)) = self.lookup(n).and_then(|s| s.ty.clone()) { self.check_type(&new, &t, line)?; }
                }
                *cell.borrow_mut() = new;
                Ok(Value::Nothing)
            }
            _ => unreachable!(),
        }
    }

    /// `map.get` / `map.remove`: hand the value back, or fail `missing`.
    fn map_fail(&mut self, op: &str, args: &Bracket, handler: Option<&Handler>, line: usize, value_used: bool) -> Result<Value, RuntimeError> {
        if op == "take" { return self.chan_take(args, handler, line, value_used); }
        let fails = vec![FailCase { name: "missing".into(), ty: None }];
        self.check_handler(handler, &fails, &format!("map.{op}"), line)?;
        let (cell, is_mut, rest) = self.map_arg(args, 2, op, line)?;
        if op == "remove" && !is_mut { return rt(line, "`map.remove` changes the map: pass `lend.mut`"); }
        self.key_ok(&rest[0], line)?;
        let mut pairs = match &*cell.borrow() { Value::Map(ps) => ps.clone(), _ => Vec::new() };
        let mut found = None;
        for (i, (k, _)) in pairs.iter().enumerate() { if self.keys_equal(k, &rest[0], line)? { found = Some(i); break; } }
        match found {
            Some(i) => {
                if op == "remove" {
                    let (_, v) = pairs.remove(i);
                    let new = Value::Map(pairs);
                    if let crate::ast::Value::Expr(Expr::Owned(_, n, _)) = &args.values[0] {
                        if let Some(t @ Ty::Ranged(..)) = self.lookup(n).and_then(|s| s.ty.clone()) { self.check_type(&new, &t, line)?; }
                    }
                    *cell.borrow_mut() = new; Ok(v)
                }
                else { Ok(pairs[i].1.clone()) }
            }
            None => self.handle_failure(handler.unwrap(), line, &format!("map.{op}"), "missing".into(), None, value_used),
        }
    }

    // ---- simd (design/simd.md, 2026-09-22) --------------------------------------
    // N plain values in one register. Lanes are a domain each; every operator is the
    // scalar one applied lane by lane (see `binop`). The oracle's target is ARM64's
    // 128-bit registers, as the parser enforces.

    /// `and.bit` and friends on vectors: the same, lane by lane.
    fn simd_bitwise(&mut self, op: &str, a: Value, b: Value, line: usize) -> Result<Value, RuntimeError> {
        let lanes = |v: &Value, n: usize| -> Vec<Value> { match v { Value::Simd(l) => l.clone(), other => vec![other.clone(); n] } };
        let n = match (&a, &b) { (Value::Simd(x), _) => x.len(), (_, Value::Simd(y)) => y.len(), _ => 0 };
        let (xs, ys) = (lanes(&a, n), lanes(&b, n));
        if xs.len() != ys.len() { return rt(line, format!("`{op}` between a simd of {} lanes and one of {}", xs.len(), ys.len())); }
        let mut out = Vec::with_capacity(n);
        for (x, y) in xs.iter().zip(&ys) {
            let (x, y) = match (x, y) { (Value::Int(x), Value::Int(y)) => (*x, *y), (x, y) => return rt(line, format!("`{op}` works on integers, got lanes of {} and {}", x.kind(), y.kind())) };
            out.push(Value::Int(match op { "and.bit" => x & y, "or.bit" => x | y, "xor" => x ^ y, "shl" => x << y, _ => x >> y }));
        }
        Ok(Value::Simd(out))
    }

    fn simd_lanes(&mut self, v: &Value, op: &str, line: usize) -> Result<Vec<Value>, RuntimeError> {
        match v { Value::Simd(l) => Ok(l.clone()), other => rt(line, format!("`simd.{op}` takes a simd value, got {}", other.kind())) }
    }

    fn simd_op(&mut self, op: &str, args: &Bracket, line: usize) -> Result<Value, RuntimeError> {
        match op {
            "fill" => {
                if args.values.len() != 1 { return rt(line, "`simd.fill` takes one value: `simd.fill[0]`"); }
                // the value lands in a lane, so the lane's type is its context (check claude, seed 4106)
                let lane = match &self.expected { Some(Ty::Simd(t, _)) => Some((**t).clone()), _ => None };
                let saved = std::mem::replace(&mut self.expected, lane.clone());
                let r = self.value(&args.values[0]);
                self.expected = saved;
                let (v, _) = r?;
                if !matches!(v, Value::Int(_) | Value::Bin(_) | Value::Bool(_)) { return rt(line, format!("a simd holds plain numbers or bools, not {}", v.kind())); }
                // the lane count comes from the declared type; 16 when nothing states one
                let n = match &self.expected { Some(Ty::Simd(_, n)) => *n, _ => 16usize };
                Ok(Value::Simd(vec![v; n]))
            }
            // `simd.load.some[lend 'xs', pos 'i', k]` loads k items, the other lanes zero;
            // `simd.store.some[lend.mut 'xs', pos 'i', k, 'v']` writes the first k lanes (owner, 2026-09-27: a)
            "load.some" | "store.some" => {
                let store = op == "store.some";
                let want = if store { 4 } else { 3 };
                if args.values.len() != want { return rt(line, format!("`simd.{op}` takes {want} arguments: `simd.{op}[{}, pos 'i', 'k'{}]`", if store { "lend.mut 'xs'" } else { "lend 'xs'" }, if store { ", 'v'" } else { "" })); }
                let (xs, prov) = self.value(&args.values[0])?;
                let cell = match &prov { Prov::Borrowed(c, m) => { if store && !*m { return rt(line, "`simd.store.some` changes the list: pass `lend.mut`"); } c.clone() }
                    Prov::Bare(n) => return rt(line, format!("`simd.{op}` takes the list by `lend '{n}'`")), _ => Rc::new(RefCell::new(xs.clone())) };
                let items = match &*cell.borrow() { Value::List(i) => i.clone(), other => return rt(line, format!("`simd.{op}` needs a list or array, got {}", other.kind())) };
                let i = match self.value(&args.values[1])?.0 { Value::Int(i) => i, other => return rt(line, format!("a position is an integer, got {}", other.kind())) };
                let k = match self.value(&args.values[2])?.0 { Value::Int(k) => k, other => return rt(line, format!("a lane count is an integer, got {}", other.kind())) };
                let (n, v) = if store {
                    let saved = self.expected.take();
                    let r = self.value(&args.values[3]);
                    self.expected = saved;
                    match r?.0 { Value::Simd(l) => (l.len(), Some(l)), other => return rt(line, format!("`simd.store.some` writes a simd value, got {}", other.kind())) }
                } else { (match &self.expected { Some(Ty::Simd(_, n)) => *n, _ => 16usize }, None) };
                if k < 0 || k as usize > n { return site(line, format!("`simd.{op}` of {k} lanes: a vector of {n} lanes takes 0 to {n}")); }
                if i < 1 || (i - 1) as usize + k as usize > items.len() {
                    return site(line, format!("`simd.{op}` of {k} lanes at position {i} runs past the end of a list of {} element(s)", items.len()));
                }
                let at = (i - 1) as usize;
                let k = k as usize;
                match v {
                    None => {
                        let zero = match items.first() { Some(Value::Bin(_)) => Value::Bin(0.0), Some(Value::Bool(_)) => Value::Bool(false), _ => Value::Int(0) };
                        let mut lanes = items[at..at + k].to_vec();
                        lanes.resize(n, zero);
                        Ok(Value::Simd(lanes))
                    }
                    Some(l) => {
                        if let Value::List(xs) = &mut *cell.borrow_mut() { for j in 0..k { xs[at + j] = l[j].clone(); } }
                        Ok(Value::Nothing)
                    }
                }
            }
            "load" | "store" => {
                let want = if op == "load" { 2 } else { 3 };
                if args.values.len() != want { return rt(line, format!("`simd.{op}` takes {want} arguments: `simd.{op}[lend 'xs', pos 'i'{}]`", if op == "store" { ", 'v'" } else { "" })); }
                let (xs, prov) = self.value(&args.values[0])?;
                let cell = match &prov { Prov::Borrowed(c, m) => { if op == "store" && !*m { return rt(line, "`simd.store` changes the list: pass `lend.mut`"); } c.clone() }
                    Prov::Bare(n) => return rt(line, format!("`simd.{op}` takes the list by `lend '{n}'`")), _ => Rc::new(RefCell::new(xs.clone())) };
                let items = match &*cell.borrow() { Value::List(i) => i.clone(), other => return rt(line, format!("`simd.{op}` needs a list or array, got {}", other.kind())) };
                let (iv, _) = self.value(&args.values[1])?;
                let i = match iv { Value::Int(i) => i, other => return rt(line, format!("a position is an integer, got {}", other.kind())) };
                // the lane count comes from the declared type (`'a': simd[int32, 4] = simd.load[…]`);
                // 16 when nothing states one
                let n = match &self.expected { Some(Ty::Simd(_, n)) => *n, _ => 16usize };
                let n = match &args.values[2..] { [a] => { let (v, _) = self.value(a)?; match v { Value::Simd(l) => l.len(), _ => n } } _ => n };
                if i < 1 || (i - 1) as usize + n > items.len() {
                    return site(line, format!("`simd.{op}` of {n} lanes at position {i} runs past the end of a list of {} element(s)", items.len()));
                }
                let at = (i - 1) as usize;
                if op == "load" { return Ok(Value::Simd(items[at..at + n].to_vec())); }
                let (v, _) = self.value(&args.values[2])?;
                let lanes = self.simd_lanes(&v, op, line)?;
                let mut items = items;
                for (k, l) in lanes.into_iter().enumerate() { items[at + k] = l; }
                *cell.borrow_mut() = Value::List(items);
                Ok(Value::Nothing)
            }
            "all" | "any" => {
                if args.values.len() != 1 { return rt(line, format!("`simd.{op}` takes one simd of bools")); }
                let (v, _) = self.value(&args.values[0])?;
                let lanes = self.simd_lanes(&v, op, line)?;
                let mut out = op == "all";
                for l in &lanes {
                    let b = match l { Value::Bool(b) => *b, other => return rt(line, format!("`simd.{op}` takes a simd of bools, whose lanes are {}", other.kind())) };
                    if op == "all" { out = out && b; } else { out = out || b; }
                }
                Ok(Value::Bool(out))
            }
            "shuffle" => {
                let (v, _) = self.value(&args.values[0])?;
                let lanes = self.simd_lanes(&v, op, line)?;
                let (order, _) = { let saved = self.expected.take(); let r = self.value(&args.values[1]); self.expected = saved; r? };
                let order = match order { Value::List(o) => o, other => return rt(line, format!("`simd.shuffle`'s order is a list of lane numbers, got {}", other.kind())) };
                let n = lanes.len();
                if order.len() != n { return rt(line, format!("`simd.shuffle` of {n} lanes takes {n} lane numbers, given {}", order.len())); }
                let mut out = Vec::with_capacity(n);
                for o in order {
                    match o { Value::Int(k) if k >= 1 && k as usize <= n => out.push(lanes[k as usize - 1].clone()),
                              Value::Int(k) => return rt(line, format!("lane {k} does not exist; a vector of {n} lanes has lanes 1 to {n}")),
                              other => return rt(line, format!("a lane number is an integer, got {}", other.kind())) }
                }
                Ok(Value::Simd(out))
            }
            "max" | "min" | "sum" => {
                if args.values.len() != 1 { return rt(line, format!("`simd.{op}` takes one simd")); }
                let (v, _) = self.value(&args.values[0])?;
                let lanes = self.simd_lanes(&v, op, line)?;
                let mut acc = lanes[0].clone();
                for l in &lanes[1..] {
                    acc = match (&acc, l) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(match op { "max" => *a.max(b), "min" => *a.min(b), _ => a.checked_add(*b).ok_or_else(|| RuntimeError { line, msg: "`simd.sum` overflows".into(), exit: None, site: true, leave: None })? }),
                        (Value::Bin(a), Value::Bin(b)) => Value::Bin(match op { "max" => a.max(*b), "min" => a.min(*b), _ => a + b }),
                        (a, b) => return rt(line, format!("`simd.{op}` folds numbers, not {} and {}", a.kind(), b.kind())),
                    };
                }
                Ok(no_neg_zero(acc))
            }
            // `simd.sum` is worked out in full and checked where it lands, as any plain number; `.lane`
            // checks it at the lane's type, `.wrap` wraps it there as the chip's own sum does, never a
            // check (owner, 2026-10-07: "There should be a way to do the other option too", then "yea")
            "sum.lane" | "sum.wrap" => {
                if args.values.len() != 1 { return rt(line, format!("`simd.{op}` takes one simd")); }
                let lane_ty = match args.values.first() { Some(crate::ast::Value::Expr(e)) => match self.expr_ty(e) { Some(Ty::Simd(t, _)) => Some(*t), _ => None }, _ => None };
                let (v, _) = self.value(&args.values[0])?;
                let lanes = self.simd_lanes(&v, op, line)?;
                let Some(lane_ty) = lane_ty else { return rt(line, format!("`simd.{op}` works at the lane's type, so it needs to know it: give the vector a type")) };
                if op == "sum.wrap" {
                    let (bits, signed) = match &lane_ty { Ty::Int(w) => (true_bits(w), true), Ty::Uint(w) => (true_bits(w), false), t => return rt(line, format!("`simd.sum.wrap` wraps modulo an integer's size, and {} is not an integer", ty_name(t))) };
                    let m: i128 = 1i128 << bits;
                    let mut acc: i128 = 0;
                    for l in &lanes { let Value::Int(x) = l else { return rt(line, "`simd.sum.wrap` is integer arithmetic") }; acc = (acc + x).rem_euclid(m); }
                    if signed && acc >= m / 2 { acc -= m; }
                    return Ok(Value::Int(acc));
                }
                let mut acc = lanes[0].clone();
                for l in &lanes[1..] {
                    acc = match (&acc, l) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
                        (Value::Bin(a), Value::Bin(b)) => round_to_width(&lane_ty, Value::Bin(a + b)),
                        (a, b) => return rt(line, format!("`simd.{op}` folds numbers, not {} and {}", a.kind(), b.kind())),
                    };
                    if matches!(acc, Value::Bin(x) if x.is_infinite()) { return site(line, format!("`simd.sum.lane` goes past the largest {}: the sum is checked at the lane's type", ty_name(&lane_ty))); }
                }
                // the total is checked, whatever order the lanes are added in: a wrapped partial sum
                // comes back, so only the answer can be wrong
                if self.check_type(&acc, &lane_ty, line).is_err() {
                    return site(line, format!("`simd.sum.lane` is {}, which does not fit {}: the sum is checked at the lane's type", acc.show_stored(), ty_name(&lane_ty)));
                }
                Ok(no_neg_zero(acc))
            }
            _ => unreachable!(),
        }
    }

    // ---- channels (design/concurrency.md, 2026-09-22) ---------------------------

    fn chan_arg(&mut self, args: &Bracket, want: usize, op: &str, line: usize) -> Result<(Rc<RefCell<Chan>>, Vec<Value>), RuntimeError> {
        if args.values.len() != want { return rt(line, format!("`channel.{op}` takes {want} argument(s), given {}", args.values.len())); }
        let (v, prov) = self.value(&args.values[0])?;
        let c = match (v, &prov) {
            (Value::Chan(c), Prov::Borrowed(..)) | (Value::Chan(c), Prov::Plain) | (Value::Chan(c), Prov::Fresh) => c,
            (Value::Chan(_), Prov::Bare(n)) => return rt(line, format!("`channel.{op}` takes the channel by `lend '{n}'`")),
            (other, _) => return rt(line, format!("`channel.{op}` needs a channel, got {}", other.kind())),
        };
        let mut rest = Vec::new();
        for a in &args.values[1..] {
            let (v, prov) = self.value(a)?;
            match prov {
                Prov::Bare(n) => return rt(line, format!("bare '{n}' into a channel: say `give '{n}'` or `copy '{n}'` — the sender no longer has it")),
                Prov::Borrowed(..) => return rt(line, "a value sent down a channel is given, never lent (no two tasks may hold one thing)"),
                _ => {}
            }
            rest.push(v);
        }
        Ok((c, rest))
    }

    fn chan_send(&mut self, args: &Bracket, line: usize) -> Result<Value, RuntimeError> {
        let (c, rest) = self.chan_arg(args, 2, "send", line)?;
        if c.borrow().timer.is_some() { return rt(line, "a timer channel (`cord::time.after`/`every`) only receives from time; nothing can send into it"); }
        // The oracle cannot suspend a task mid-body, so a sender never waits here: a
        // hand-off or a full bounded channel simply queues (provisional — the waiting is
        // unobservable in output; only its absence in a deadlock would be). Takers wait.
        c.borrow_mut().queue.push_back(rest.into_iter().next().unwrap());
        Ok(Value::Nothing)
    }

    /// A value from the channel if one is ready. `time_may_pass`: nothing else can run, so a
    /// timer channel fires.
    fn chan_ready(&mut self, c: &Rc<RefCell<Chan>>, time_may_pass: bool) -> Option<Value> {
        let mut ch = c.borrow_mut();
        if let Some(v) = ch.queue.pop_front() { return Some(v); }
        match &mut ch.timer {
            Some(Timer::After { fired, nanos }) if time_may_pass && !*fired => { *fired = true; Some(Value::Duration(*nanos)) }
            Some(Timer::Every { nanos }) if time_may_pass => Some(Value::Duration(*nanos)),
            _ => None,
        }
    }

    fn chan_take(&mut self, args: &Bracket, handler: Option<&Handler>, line: usize, value_used: bool) -> Result<Value, RuntimeError> {
        let fails = vec![FailCase { name: "closed".into(), ty: None }];
        self.check_handler(handler, &fails, "channel.take", line)?;
        let (c, _) = self.chan_arg(args, 1, "take", line)?;
        loop {
            if let Some(v) = self.chan_ready(&c, false) { return Ok(v); }
            if self.run_any_task()? { continue; }
            if let Some(v) = self.chan_ready(&c, true) { return Ok(v); }
            // nothing can ever send: closed
            return self.handle_failure(handler.unwrap(), line, "channel.take", "closed".into(), None, value_used);
        }
    }

    fn run_whichever_arm(&mut self, arm: &WhicheverArm, v: Value) -> Result<Flow, RuntimeError> {
        self.push();
        let owned = v.is_owned();
        let r = (|| {
            self.declare(&arm.name, Slot { cell: Rc::new(RefCell::new(v)), ty: None, kind: SlotKind::Owned, alive: true, immut: false, sized: false, sealed: false , since: 0, deps: Vec::new() }, arm.line)?;
            let _ = owned;
            self.block(&arm.body)
        })();
        self.pop();
        r
    }

    // ---- calls ------------------------------------------------------------

    /// While something is lent `lend.mut`, nothing else may borrow it (owner, 2026-10-02: a): one
    /// call lending the same variable twice, once to change, is refused, a handed-back borrow
    /// (`grow[lend.mut 'u']`) counting as a lend of what it borrows.
    fn one_lend_mut(&self, args: &crate::ast::Bracket, line: usize) -> Result<(), RuntimeError> {
        let mut lends: Vec<(String, bool)> = Vec::new();
        for a in &args.values {
            match a {
                crate::ast::Value::Expr(Expr::Owned(Verb::LendMut, s, _)) => lends.push((s.clone(), true)),
                crate::ast::Value::Expr(Expr::Owned(Verb::Lend, s, _)) => lends.push((s.clone(), false)),
                crate::ast::Value::Expr(Expr::Call(c)) => {
                    let m = self.prog.funcs.iter().find(|f| f.name == *c.path.last().unwrap() && f.module == c.path[..c.path.len() - 1]).and_then(|f| f.ret_lent.as_ref()).map(|(m, _)| *m);
                    if let Some(m) = m { for s in self.call_froms(c, 0) { lends.push((s, m)); } }
                }
                _ => {}
            }
        }
        for (i, (s, m)) in lends.iter().enumerate() {
            for (t, n) in &lends[i + 1..] {
                if s == t && (*m || *n) {
                    return rt(line, format!("'{s}' is lent to change (`lend.mut`) and lent again in the same call: while something is lent to change, nothing else may borrow it (owner, 2026-10-02). To read and change it, pass it once with `lend.mut`; to keep how it was, `copy` it first"));
                }
            }
        }
        Ok(())
    }

    fn call(&mut self, c: &Call) -> Result<Value, RuntimeError> {
        self.one_lend_mut(&c.args, c.line)?;
        let shown = c.written.join(".");
        // a name written bare may be this Neuron's, std's, or another Neuron's `bare.optional` / `bare.only`
        // one; more than one is an error where it is written, and this Neuron's own does not win
        // (owner, 2026-10-06: "`a`, and no")
        if c.bare {
            let own_module: Vec<String> = c.from.iter().cloned().chain(c.written[..c.written.len() - 1].iter().cloned()).collect();
            let own = self.prog.funcs.iter().any(|f| f.module == own_module && Some(&f.name) == c.written.last() && !f.is_op);
            let std = c.limb.as_deref() == Some("std");
            let others = self.bare_fns_elsewhere(&c.from, &c.written, false);
            let mut who: Vec<String> = Vec::new();
            if own {
                let mine: Vec<&str> = own_module.iter().filter(|m| !m.starts_with("::")).map(String::as_str).chain(c.written.last().map(String::as_str)).collect();
                who.push(format!("`my.{}` (this Neuron's)", mine.join(".")));
            }
            if std { who.push(format!("`cord::{shown}`")); }
            for f in &others { who.push(format!("`{}::{shown}`", f.module[0].trim_start_matches("::"))); }
            if who.len() > 1 { return rt(c.line, format!("`{shown}` could be any of {}: write the path", who.join(", "))); }
            if who.is_empty() {
                // another Neuron's name that needs its path says so, rather than "no function"
                let prog: &'p Program = self.prog;
                if let Some(f) = prog.funcs.iter().find(|f| f.module.first().map_or(false, |r| r.starts_with("::")) && f.vis == crate::ast::Vis::Pub && !f.is_op
                    && f.module[1..].iter().map(String::as_str).chain(std::iter::once(f.name.as_str())).collect::<Vec<_>>() == c.written.iter().map(String::as_str).collect::<Vec<_>>()
                    && self.bare_of.get(&bare_key("fn", &f.module, &f.name)) == Some(&crate::ast::Bare::Never)) {
                    let l = f.module[0].trim_start_matches("::");
                    return rt(c.line, format!("`{shown}` is `{l}`'s and `bare.never`: `{l}` requires its path, so write `{l}::{shown}[…]`"));
                }
            }
            if !std && !own && others.len() == 1 {
                let mut local = c.clone();
                local.bare = false;
                local.path = others[0].module.iter().cloned().chain(std::iter::once(others[0].name.clone())).collect();
                return self.user_call(&local);
            }
        }
        match &c.limb {
            Some(l) if l == "std" => self.std_call(c),
            // another Neuron, loaded from `Kinesin.nrp` under its own root (owner, 2026-09-29)
            Some(l) if self.prog.modules.iter().any(|m| m.path == [format!("::{l}")]) => {
                let mut local = c.clone();
                local.limb = None;
                local.path.insert(0, format!("::{l}"));
                // a `bare.only` name forbids its path (owner, 2026-10-06)
                let (module, name) = local.path.split_at(local.path.len() - 1);
                if self.bare_of.get(&bare_key("fn", module, &name[0])) == Some(&crate::ast::Bare::Only) {
                    return rt(c.line, format!("`{l}::{shown}` is `bare.only`: `{l}` forbids its path, so write `{shown}[…]`"));
                }
                self.user_call(&local)
            }
            Some(l) => rt(c.line, format!("no Neuron with the prefix `{l}`: a Neuron this one uses is listed in its `Kinesin.nrp` (`dependencies = [kin::Dep[path = …],]`), and its own `Kinesin.nrp` says `prefix = \"{l}\"`")),
            None => self.user_call(c),
        }
    }

    fn user_call(&mut self, c: &Call) -> Result<Value, RuntimeError> {
        let (module, name) = c.path.split_at(c.path.len() - 1);
        let name = &name[0];
        // `Status.claimed["x"]` — an enum case with data, spelled like a call
        if !module.is_empty() {
            let tname = module.join(".");
            if self.prog.enums.iter().any(|e| e.name == tname) {
                if c.handler.is_some() { return rt(c.line, "an enum value cannot fail; drop the `otherwise`"); }
                let from = c.from.clone();
                return self.enum_value(&tname, name, Some(&c.args), &from, c.line).map(|(v, _)| v);
            }
        }
        self.check_path(&c.from, module, c.line)?;
        let func = match self.prog.funcs.iter().find(|f| f.module == module && &f.name == name) {
            Some(f) => f,
            None => return rt(c.line, format!("no function `{name}` in `{}`", root_or(module))),
        };
        if !self.reachable(&c.from, &func.module, func.vis) {
            return rt(c.line, private_msg(&c.from, &func.module, &format!("`{}`", join_path(&func.module, name))));
        }
        if func.is_op { return rt(c.line, format!("`{name}` is an operator; it goes between its operands (`'a' {name} 'b'`), never called with `[…]`")); }
        if func.is_test { return rt(c.line, format!("`{name}` is a `test fn`: `synapse test` runs it, and nothing calls it")); }
        if func.is_extern {
            if self.nocheck_depth == 0 && self.marks_needed() { return rt(c.line, format!("`{name}` is extern; nothing about it can be proven — the call is a no_check site: `no_check … {name}[…]`")); }
            let ranged_ret = matches!(func.ret, Some(Ty::Ranged(..)));
            if !ranged_ret && c.handler.is_some() { return rt(c.line, format!("`{name}` is extern and cannot fail; drop the `otherwise` (a wrapper turns its return into a failure — or give the extern a ranged return, which fails `out_of_range`)")); }
            if ranged_ret {
                // the extern's return contract: a failure, not a check site — `nocheck` does not remove it
                let value_used = !self.call_is_stmt;
                self.call_is_stmt = false;
                let fails = vec![FailCase { name: "out_of_range".into(), ty: None }];
                self.check_handler(c.handler.as_ref(), &fails, name, c.line)?;
                let v = self.extern_call(func, c)?;
                let ret = func.ret.clone().unwrap();
                if self.check_type(&v, &ret, c.line).is_err() {
                    return self.handle_failure(c.handler.as_ref().unwrap(), c.line, name, "out_of_range".into(), None, value_used);
                }
                return Ok(v);
            }
            return self.extern_call(func, c);
        }
        let variadic = func.params.last().map(|p| p.variadic).unwrap_or(false);
        let fixed = if variadic { func.params.len() - 1 } else { func.params.len() };
        if (!variadic && c.args.values.len() != fixed) || (variadic && c.args.values.len() < fixed) {
            return rt(c.line, format!("`{name}` takes {}{} argument(s), given {}", if variadic { "at least " } else { "" }, fixed, c.args.values.len()));
        }
        // The caller's side of failure (decided 2026-09-19): a call that can fail must say
        // what happens; one that cannot must not. Checked before the call runs, as the
        // compiler would.
        let value_used = !self.call_is_stmt;
        self.call_is_stmt = false;
        self.check_handler(c.handler.as_ref(), &func.fails, name, c.line)?;
        // this call's blanks (`<T>`) are its own
        let saved_tvars = std::mem::take(&mut *self.tvars.borrow_mut());
        let r = self.user_call_inner(func, c, name, value_used);
        *self.tvars.borrow_mut() = saved_tvars;
        r
    }

    fn user_call_inner(&mut self, func: &Func, c: &Call, name: &str, value_used: bool) -> Result<Value, RuntimeError> {
        self.call_func_with(func, c, name, value_used, Vec::new())
    }

    fn call_func_with(&mut self, func: &Func, c: &Call, name: &str, value_used: bool, captures: Vec<(String, Rc<RefCell<Value>>, bool)>) -> Result<Value, RuntimeError> {

        // Arguments against the parameter contract (design/ownership-syntax.md).
        // A variadic parameter (`...'values'`) collects the rest into a list, each
        // argument checked as if bound to that parameter on its own.
        let mut bound: Vec<(String, Slot)> = Vec::new();
        // `...'xs'` (decided 2026-09-21) unpacks a list of known length into the variadic
        // part: each element becomes one argument. Its length must be known here.
        enum Arg<'a> { Ast(&'a crate::ast::Value), Val(Value, Prov) }
        let mut pairs: Vec<(Arg, &Param)> = Vec::new();
        let variadic_at = func.params.last().filter(|p| p.variadic).map(|_| func.params.len() - 1);
        let mut pos = 0usize;
        for arg in c.args.values.iter() {
            if let crate::ast::Value::Expr(Expr::Spread(xs, verb, _)) = arg {
                let Some(vi) = variadic_at else { return rt(c.line, format!("`...'{xs}'` unpacks into a variadic parameter, and `{name}` has none")) };
                if pos < vi { return rt(c.line, format!("`...'{xs}'` unpacks into the variadic part of `{name}`, but parameter '{}' comes first and takes one value", func.params[pos].name)); }
                let slot = match self.lookup(xs) { Some(s) => s.clone(), None => return rt(c.line, format!("'{xs}' is not declared")) };
                if !slot.alive { return rt(c.line, format!("'{xs}' was given away")); }
                let items = match &*slot.cell.borrow() { Value::List(items) => items.clone(), other => return rt(c.line, format!("`...'{xs}'` unpacks a list; '{xs}' is a {}", other.kind())) };
                if !slot.sized { return rt(c.line, format!("`...'{xs}'`: the length of '{xs}' is not known here, so `{name}` cannot be compiled for it — give '{xs}' a bound (`list[T][lo, hi]`) or pass it as a list (decided 2026-09-21)")); }
                let param = &func.params[vi];
                let owned_elems = items.iter().any(|it| it.is_owned());
                // Owned elements into an owning parameter need the verb said (decided 2026-09-21):
                // `...copy 'xs'` duplicates each, `...give 'xs'` moves each out and leaves '{xs}' empty.
                match (verb, param.kind, owned_elems) {
                    (Some(_), _, false) => return rt(c.line, format!("the elements of '{xs}' are plain values and own nothing; they copy by themselves — `...'{xs}'` with no verb")),
                    (Some(_), ParamKind::Lent, true) => return rt(c.line, format!("`{name}` only looks at '{}': `...'{xs}'` with no verb lends the elements", param.name)),
                    (None, ParamKind::Owned, true) => return rt(c.line, format!("`{name}` takes ownership of '{}': say `...copy '{xs}'` or `...give '{xs}'`", param.name)),
                    (_, ParamKind::LentMut, true) => return rt(c.line, format!("`lend.mut` of the elements of '{xs}' by `...` is not modelled by the oracle")),
                    _ => {}
                }
                for it in items {
                    let prov = match (verb, it.is_owned()) {
                        (_, false) => Prov::Plain,
                        (None, true) => Prov::Borrowed(Rc::new(RefCell::new(it.clone())), false),
                        (Some(Verb::Copy), true) => Prov::Copied,
                        (Some(_), true) => Prov::Given,
                    };
                    pairs.push((Arg::Val(it, prov), param));
                }
                if matches!(verb, Some(Verb::Give)) && owned_elems {
                    // every element moved out: the list is empty now, not holed
                    if let Some(s) = self.lookup_mut(xs) { *s.cell.borrow_mut() = Value::List(Vec::new()); }
                }
                pos += 1;
                continue;
            }
            pairs.push((Arg::Ast(arg), &func.params[pos.min(func.params.len() - 1)]));
            pos += 1;
        }
        let mut rest: Vec<Value> = Vec::new();
        for (arg, param) in pairs {
            // the declared type of a variable handed over by a verb, to hold a lend to its layout
            let src_ty = match &arg { Arg::Ast(crate::ast::Value::Expr(Expr::Owned(_, n, _))) => self.lookup(n).and_then(|s| s.ty.clone()), _ => None };
            // an argument lands in its parameter, so the parameter's type is its context, as a
            // declaration's is (owner, 2026-09-29: a): `show[0 -~ 1]` into a uint8 wraps at 8 bits
            let (v, prov) = match arg {
                Arg::Ast(a) => {
                    let pt = param.ty.clone().filter(|t| !matches!(t, Ty::Var(_) | Ty::Inst(..)));
                    let saved = std::mem::replace(&mut self.expected, pt);
                    let r = self.value(a);
                    self.expected = saved;
                    r?
                }
                Arg::Val(v, p) => (v, p),
            };
            let pn = &param.name;
            let slot = match param.kind {
                ParamKind::Owned => {
                    match &prov {
                        Prov::Bare(n) => return rt(c.line, format!("`{name}` takes ownership of '{pn}' — pass `give '{n}'` or `copy '{n}'`")),
                        Prov::Borrowed(..) => return rt(c.line, format!("`{name}` takes ownership of '{pn}' but you lent it; `give` or `copy` instead")),
                        _ => {}
                    }
                    let v = self.fit(v, prov, param.ty.as_ref(), c.line)?;
                    Slot { cell: Rc::new(RefCell::new(v)), ty: param.ty.clone(), kind: SlotKind::Owned, alive: true, immut: param.mutability == Some(Mutability::Immut), sized: false, sealed: false , since: 0, deps: Vec::new() }
                }
                ParamKind::Lent | ParamKind::LentMut => {
                    let want_mut = param.kind == ParamKind::LentMut;
                    let kind = if want_mut { SlotKind::LentMut } else { SlotKind::Lent };
                    match prov {
                        Prov::Borrowed(cell, is_mut) => {
                            if want_mut && !is_mut { return rt(c.line, format!("`{name}` may modify '{pn}': pass `lend.mut`, not `lend`")); }
                            if !want_mut && is_mut { return rt(c.line, format!("`{name}` only looks at '{pn}': pass `lend`, not `lend.mut`")); }
                            // a lend hands the value over where it is, so its widths must match the
                            // parameter's (owner, 2026-09-24: a); a range's bounds do not change the layout
                            if let (Some(st), Some(pt)) = (&src_ty, &param.ty) {
                                let mut vars = Vec::new(); pt.vars_into(&mut vars); st.vars_into(&mut vars);
                                if vars.is_empty() && layout_name(st) != layout_name(pt) {
                                    return rt(c.line, format!("`{name}` looks at '{pn}' as {}, but what you lent is {}: a lend hands over the value where it is, so the sizes must match (change the parameter, or make it generic)", ty_name(pt), ty_name(st)));
                                }
                            }
                            if let Some(t) = &param.ty { self.check_type(&v, t, c.line)?; }
                            Slot { cell, ty: param.ty.clone(), kind, alive: true, immut: false, sized: false, sealed: false , since: 0, deps: Vec::new() }
                        }
                        Prov::Bare(n) => return rt(c.line, format!("`{name}` only looks at '{pn}' — pass `lend '{n}'`")),
                        Prov::Given => return rt(c.line, format!("`{name}` only looks at '{pn}': it is `lent`, so the caller writes `lend`, not `give`")),
                        Prov::Copied => return rt(c.line, format!("`{name}` only looks at '{pn}': it is `lent`, so the caller writes `lend`, not `copy`")),
                        // A plain value, or a fresh temporary (literal, call result): nothing owns
                        // it, so it lives for the call and is lent by the call itself (decided 2026-09-14).
                        other => {
                            let v = self.fit(v, other, param.ty.as_ref(), c.line)?;
                            Slot { cell: Rc::new(RefCell::new(v)), ty: param.ty.clone(), kind, alive: true, immut: false, sized: false, sealed: false , since: 0, deps: Vec::new() }
                        }
                    }
                }
            };
            if param.variadic { rest.push(slot.cell.borrow().clone()); } else { bound.push((pn.clone(), slot)); }
        }
        if let Some(p) = func.params.last().filter(|p| p.variadic) {
            let kind = match p.kind { ParamKind::Owned => SlotKind::Owned, ParamKind::Lent => SlotKind::Lent, ParamKind::LentMut => SlotKind::LentMut };
            bound.push((p.name.clone(), Slot { cell: Rc::new(RefCell::new(Value::List(rest))), ty: None, kind, alive: true, immut: p.mutability == Some(Mutability::Immut), sized: true, sealed: false , since: 0, deps: Vec::new() }));
        }

        // `<T: Ordered>`: the type that filled the blank must keep the promise
        for (var, iface) in &func.bounds {
            let shape = self.tvars.borrow().get(var).cloned();
            if let Some(shape) = shape {
                if let Err(m) = self.satisfies(&shape, iface, c.line)? { return rt(c.line, format!("`{name}` needs `<{var}>: {iface}`, but {m}")); }
            }
        }
        // captured variables (a function value's `with […]`) are visible inside as what they were taken as
        for (cname, cell, mutable) in captures {
            if bound.iter().any(|(n, _)| n == &cname) { return rt(c.line, format!("'{cname}' is both a parameter and a capture")); }
            bound.push((cname, Slot { cell, ty: None, kind: if mutable { SlotKind::Owned } else { SlotKind::Lent }, alive: true, immut: false, sized: false, sealed: false , since: 0, deps: Vec::new() }));
        }
        // which borrowed parameter is which cell, to check a `-> lent@x` return against
        let lent_params: Vec<(String, Option<String>, Rc<RefCell<Value>>)> = func.params.iter()
            .filter(|p| p.kind != ParamKind::Owned)
            .filter_map(|p| bound.iter().find(|(n, _)| n == &p.name).map(|(_, sl)| (p.name.clone(), p.life.clone(), sl.cell.clone())))
            .collect();
        self.ret_borrow = None;
        let flow = self.run_body(func, bound);
        // this call's blanks are its own; the body's `let 'y': <T>` and the return check use them
        let result = match flow? {
            Flow::Return(v) => v,
            Flow::Normal => Value::Nothing,
            Flow::Break | Flow::Continue => return rt(func.line, "`break`/`continue` outside a loop"),
            Flow::Give(_) => return rt(func.line, "`giveS` outside a case"),
            Flow::Fail(reason, payload) => return self.handle_failure(c.handler.as_ref().unwrap(), c.line, name, reason, payload, value_used),
        };
        // `-> lent@x T`: the borrow must be of a parameter in group x — or of any lent parameter
        // when no group is written (Cortex works it out) — never of something that dies here
        let borrowed = self.ret_borrow.take();
        match (&func.ret_lent, borrowed) {
            (Some((_, life)), Some((cell, m))) => {
                // one list may be lent to two parameters: any of them in the group will do
                let mut sharing = lent_params.iter().filter(|(_, _, pc)| Rc::ptr_eq(pc, &cell));
                let from = match life { Some(l) => lent_params.iter().filter(|(_, _, pc)| Rc::ptr_eq(pc, &cell)).find(|(_, pl, _)| pl.as_ref() == Some(l)).or_else(|| sharing.next()), None => sharing.next() };
                match from {
                    None => return rt(self.ret_line, format!("`{name}` hands back a borrow of something that dies when it returns; a borrow it returns must come from a `lent` parameter")),
                    Some((pn, plife, _)) => if let Some(l) = life { if plife.as_ref() != Some(l) { return rt(self.ret_line, format!("`{name}` returns a borrow of '{pn}', which is not in @{l}: say `lent@{l} '{pn}'`")); } },
                }
                // `@x`: the result lives while everything in the group does, whichever the body returned
                // without a group, whatever the body may hand back, on any path (Cortex reads the body)
                let deps = self.may_return(&func, 0);
                self.call_deps = match life {
                    Some(l) => lent_params.iter().filter(|(_, pl, _)| pl.as_ref() == Some(l)).map(|(_, _, pc)| Rc::as_ptr(pc) as *const () as usize).collect(),
                    None => lent_params.iter().filter(|(n, _, _)| deps.contains(n)).map(|(_, _, pc)| Rc::as_ptr(pc) as *const () as usize).collect(),
                };
                self.call_borrow = Some((cell, m));
            }
            (Some(_), None) => return rt(self.ret_line, format!("`{name}` says it hands back a borrow (`-> lent …`) but returned a value of its own; `return lend '…'`")),
            (None, Some(_)) => return rt(self.ret_line, format!("`{name}` returns a borrow, so its signature must say so: `-> lent@x …`")),
            (None, None) => {}
        }
        match (&func.ret, &result) {
            (Some(t), Value::Nothing) => rt(c.line, format!("`{name}` is declared `-> {}` but returned nothing", ty_name(t))),
            // the answer lands in the return type at the `return` (seed 3761, check claude, 2026-09-29)
            (Some(t), v) => { let k = self.ret_kind.take(); self.kind_clash(k, t, self.ret_line)?; self.check_type(v, t, self.ret_line)?; Ok(result) }
            (None, _) => { self.ret_kind = None; Ok(result) }
        }
    }

    /// Does a type (by shape) keep an interface's promises? Built-in operators
    /// count for the language's types; a user type needs an `op`/`fn` for each entry.
    fn satisfies(&self, shape: &str, iface: &str, line: usize) -> Result<Result<(), String>, RuntimeError> {
        let def = match self.prog.interfaces.iter().find(|i| i.name == iface) { Some(i) => i, None => return rt(line, format!("no interface named `{iface}`")) };
        // a listed type is accepted, nothing else (owner, 2026-10-02). The oracle knows a value's
        // family, not its width, so `int32` here admits any integer: the compiler checks the width.
        if !def.types.is_empty() && !def.types.iter().any(|t| ty_shape(t) == shape) {
            return Ok(Err(format!("{shape} is not `{iface}`: it is not one of the types it lists")));
        }
        let builtin_ops: &[&str] = match shape {
            "integer" | "bin" => &["+", "-", "x", "xx", "div", "mod", "<", ">", "</==", ">/==", "==", "!=="],
            "deci" => &["+", "-", "x", "div", "<", ">", "</==", ">/==", "==", "!=="],
            "bool" => &["and", "or", "==", "!=="],
            "str" => &["++", "==", "!=="],
            "char" | "list" => &["==", "!=="],
            _ => &["==", "!=="],
        };
        for e in &def.entries {
            let found = if e.is_op {
                builtin_ops.contains(&e.name.as_str()) || self.prog.funcs.iter().any(|f| f.is_op && f.name == e.name && f.params[0].ty.as_ref().map(|t| ty_shape(t) == shape).unwrap_or(false))
            } else {
                self.prog.funcs.iter().any(|f| !f.is_op && f.name == e.name && f.params.len() == e.params.len()
                    && f.params.iter().zip(&e.params).all(|(fp, ep)| match (&fp.ty, &ep.ty) { (Some(ft), Some(Ty::Var(_))) => ty_shape(ft) == shape, (Some(ft), Some(et)) => ty_name(ft) == ty_name(et), _ => true }))
            };
            if !found { return Ok(Err(format!("{} is not `{iface}`: no `{} {}` for it", shape, if e.is_op { "op" } else { "fn" }, e.name))); }
        }
        Ok(Ok(()))
    }

    /// `Vec satisfies Ordered;` claims, checked at load.
    fn check_claims(&self) -> Result<(), RuntimeError> {
        for c in &self.prog.claims {
            if let Err(m) = self.satisfies(&ty_shape(&c.ty), &c.iface, c.line)? { return rt(c.line, format!("the claim `{} satisfies {}` is false: {m}", ty_name(&c.ty), c.iface)); }
        }
        Ok(())
    }

    /// `'f'[args]` — call a function value: bind arguments as `user_call` does, then the
    /// captures as variables of the body (lent or owned by how they were taken).
    fn call_value(&mut self, name: &str, args: &Bracket, handler: Option<&Handler>, line: usize) -> Result<Value, RuntimeError> {
        let fv = match self.lookup(name) {
            Some(s) if !s.alive => return rt(line, format!("'{name}' was already given away")),
            Some(s) => match &*s.cell.borrow() { Value::Func(fv) => fv.clone(), other => return rt(line, format!("'{name}' is {}, not a function; `'{name}'[…]` calls, `'{name}'[pos …]` indexes", other.kind())) },
            None => return rt(line, format!("'{name}' is not declared")),
        };
        let func = fv.def.clone();
        let value_used = !self.call_is_stmt;
        self.call_is_stmt = false;
        self.check_handler(handler, &func.fails, name, line)?;
        let saved_tvars = std::mem::take(&mut *self.tvars.borrow_mut());
        let synthetic = Call { handler: handler.cloned(), limb: None, path: vec![func.name.clone()], from: self.module.clone(), args: args.clone(), line, bare: false, written: vec![func.name.clone()] };
        let r = self.call_func_with(&func, &synthetic, name, value_used, fv.captures.clone());
        *self.tvars.borrow_mut() = saved_tvars;
        r
    }

    /// Call a function value from emulated C: arguments already evaluated, by value.
    fn call_func_value(&mut self, fv: &FuncVal, args: Vec<Value>, line: usize) -> Result<Value, RuntimeError> {
        let func = fv.def.clone();
        if args.len() != func.params.len() { return rt(line, format!("`{}` takes {} argument(s), C gave {}", func.name, func.params.len(), args.len())); }
        let mut bound = Vec::new();
        for (v, p) in args.into_iter().zip(&func.params) {
            let v = self.fit(v, Prov::Plain, p.ty.as_ref(), line)?;
            bound.push((p.name.clone(), Slot { cell: Rc::new(RefCell::new(v)), ty: p.ty.clone(), kind: SlotKind::Owned, alive: true, immut: false, sized: false, sealed: false , since: 0, deps: Vec::new() }));
        }
        let saved_depth = self.nocheck_depth;
        let flow = self.run_body(&func, bound);
        self.nocheck_depth = saved_depth;
        match flow? {
            Flow::Return(v) => Ok(v),
            Flow::Normal => Ok(Value::Nothing),
            Flow::Fail(..) => rt(line, "a callback cannot fail"),
            Flow::Break | Flow::Continue => rt(line, "`break`/`continue` outside a loop"),
            Flow::Give(_) => rt(line, "`giveS` outside a case"),
        }
    }

    /// Run a function's body with its parameters bound. A function's scopes are its
    /// own: it cannot see the caller's variables.
    fn run_body(&mut self, func: &Func, bound: Vec<(String, Slot)>) -> Result<Flow, RuntimeError> {
        let saved_ret = std::mem::replace(&mut self.cur_ret, func.ret.clone().filter(|t| !matches!(t, Ty::Var(_) | Ty::Inst(..))));
        let r = self.run_body_inner(func, bound);
        self.cur_ret = saved_ret;
        r
    }

    fn run_body_inner(&mut self, func: &Func, bound: Vec<(String, Slot)>) -> Result<Flow, RuntimeError> {
        let saved = std::mem::replace(&mut self.scopes, vec![HashMap::new()]);
        let saved_module = std::mem::replace(&mut self.module, func.module.clone());
        let saved_fails = std::mem::replace(&mut self.fails, func.fails.clone());
        let flow = (|| {
            for (n, s) in bound { self.declare(&n, s, func.line)?; }
            self.stmts(&func.body)
        })();
        self.scopes = saved;
        self.module = saved_module;
        self.fails = saved_fails;
        flow
    }

    /// A custom operator (decided 2026-09-20): `'u' <+> 'v'` / `'u' dot 'v'`. Operands
    /// are lent unless the `op` says `give`, in which case the caller writes `give` too.
    fn custom_op(&mut self, op: &str, l: &Expr, r: &Expr, line: usize) -> Result<(Value, Prov), RuntimeError> {
        // std's `pow` / `pow.wrap` (owner, 2026-10-03), unless a type of the program's defines its own:
        // the power the language's `xx` / `xx~` was, with its rules
        // `pow.ieee` keeps IEEE's -0 (owner, 2026-10-07: "yea")
        if (op == "pow" || op == "pow.wrap" || op == "pow.ieee") && !self.prog.funcs.iter().any(|f| f.is_op && f.name == op) {
            let inner = match op { "pow" => "xx", "pow.ieee" => "xx.ieee", _ => "xx~" };
            return self.eval(&Expr::Binary(inner.into(), Box::new(l.clone()), Box::new(r.clone()), line));
        }
        let left = self.eval(l)?;
        self.custom_op_with(op, left, l, r, line)
    }

    /// As `custom_op`, with the left operand already evaluated (it is evaluated once).
    fn custom_op_with(&mut self, op: &str, left: (Value, Prov), l: &Expr, r: &Expr, line: usize) -> Result<(Value, Prov), RuntimeError> {
        // pick the definition whose first operand's type matches the left value (an
        // operator may be defined for several types)
        let lshape = shape_of(&left.0);
        // `vecmath::dot`: only that Neuron's `dot` (owner, 2026-10-02: a Neuron's word operator may carry its path)
        let (limb, name) = match op.split_once("::") { Some((l, n)) => (Some(format!("::{l}")), n), None => (None, op) };
        let candidates: Vec<Func> = self.prog.funcs.iter().filter(|f| f.is_op && f.name == name && limb.as_ref().map_or(true, |l| f.module.first() == Some(l))).cloned().collect();
        // the one written for this type, else a generic one (they never overlap: `check_duplicates`)
        let def = match candidates.iter().find(|f| f.params[0].ty.as_ref().map(|t| ty_shape(t) == lshape).unwrap_or(false))
            .or_else(|| candidates.iter().find(|f| f.params[0].ty.as_ref().map(|t| matches!(t, Ty::Var(_))).unwrap_or(true))).or(candidates.first()) {
            Some(f) => f.clone(),
            None => {
                let hint = match op {
                    "*" => "; multiply is `times`, power is cord's `pow`", "^" => "; power is cord's `pow`", "%" => "; remainder is `mod`",
                    "&" | "|" | "!" => "; use `and`, `or`, `!`", _ => "",
                };
                if let Some(l) = &limb { return rt(line, format!("the Neuron `{}` has no operator `{name}` for these values", &l[2..])); }
                return rt(line, format!("`{op}` is not an operator here{hint} — declare it with `op {op} ['a': T, 'b': T] -> R {{ … }}`"));
            }
        };
        // written bare, the same word operator for these types in two Neurons is a clash where it is
        // written, as a bare name is (owner, 2026-10-06: "`a`, and no")
        if limb.is_none() {
            let same: Vec<&Func> = candidates.iter().filter(|f| f.params.len() == def.params.len() && f.params.iter().zip(&def.params).all(|(a, b)| a.ty.as_ref().map(ty_shape) == b.ty.as_ref().map(ty_shape))).collect();
            let mut roots: Vec<String> = same.iter().map(|f| f.module.first().filter(|r| r.starts_with("::")).map(|r| format!("`{}::{name}`", r.trim_start_matches("::"))).unwrap_or_else(|| format!("`{name}` (this Neuron's)"))).collect();
            roots.dedup();
            if roots.len() > 1 { return rt(line, format!("`{name}` could be any of {}: write the path", roots.join(", "))); }
        }
        // another Neuron's word operator follows its `bare` mark (owner, 2026-10-06): `bare.never` needs its
        // path, `bare.only` refuses it; a glyph operator has no path and no mark
        if let Some(root) = def.module.first().filter(|r| r.starts_with("::")) {
            let here = self.module.first().filter(|r| r.starts_with("::"));
            let l = root.trim_start_matches("::");
            match (self.bare_of.get(&bare_key("op", &def.module, &def.name)), &limb) {
                (Some(crate::ast::Bare::Never), None) if here != Some(root) => return rt(line, format!("`{name}` is `{l}`'s and `bare.never`: write `{l}::{name}`")),
                (Some(crate::ast::Bare::Only), Some(_)) => return rt(line, format!("`{l}::{name}` is `bare.only`: `{l}` forbids its path, so write `{name}`")),
                _ => {}
            }
        }
        let mut bound = Vec::new();
        let mut left = Some(left);
        for (expr, param) in [(l, &def.params[0]), (r, &def.params[1])] {
            let (v, prov) = match left.take() { Some(lv) => lv, None => self.eval(expr)? };
            let slot = match param.kind {
                ParamKind::Owned => {
                    match &prov {
                        Prov::Bare(n) => return rt(line, format!("`{op}` takes '{}' by value: write `give '{n}'` (or `copy '{n}'`)", param.name)),
                        Prov::Borrowed(..) => return rt(line, format!("`{op}` takes '{}' by value, not a lend", param.name)),
                        _ => {}
                    }
                    let v = self.fit(v, prov, param.ty.as_ref(), line)?;
                    Slot { cell: Rc::new(RefCell::new(v)), ty: param.ty.clone(), kind: SlotKind::Owned, alive: true, immut: false, sized: false, sealed: false , since: 0, deps: Vec::new() }
                }
                _ => {
                    // lent: an operand is looked at, never consumed; a variable is borrowed in place
                    let is_lit = matches!(prov, Prov::Literal);
                    let cell = match (expr, &prov) {
                        (Expr::Var(name, _), Prov::Bare(_)) => { if let Some(t) = &param.ty { self.check_type(&v, t, line)?; } self.lookup(name).map(|s| s.cell.clone()).unwrap_or_else(|| Rc::new(RefCell::new(v))) }
                        (_, Prov::Borrowed(c, _)) => { if let Some(t) = &param.ty { self.check_type(&v, t, line)?; } c.clone() }
                        _ => { let v = self.fit(v, if is_lit { Prov::Literal } else { Prov::Plain }, param.ty.as_ref(), line)?; Rc::new(RefCell::new(v)) }
                    };
                    Slot { cell, ty: param.ty.clone(), kind: SlotKind::Lent, alive: true, immut: false, sized: false, sealed: false , since: 0, deps: Vec::new() }
                }
            };
            bound.push((param.name.clone(), slot));
        }
        let result = match self.run_body(&def, bound)? {
            Flow::Return(v) => v,
            Flow::Normal => return rt(line, format!("`op {op}` fell off the end without returning a value")),
            Flow::Break | Flow::Continue => return rt(def.line, "`break`/`continue` outside a loop"),
            Flow::Give(_) => return rt(def.line, "`giveS` outside a case"),
            Flow::Fail(..) => return rt(line, "an operator cannot fail"),
        };
        let k = self.ret_kind.take();
        if let Some(t) = &def.ret { self.kind_clash(k, t, line)?; self.check_type(&result, t, line)?; }
        let prov = if result.is_owned() { Prov::Fresh } else { Prov::Plain };
        Ok((result, prov))
    }

    /// The caller's side, checked before anything runs (as the compiler would): a
    /// call that can fail must say what happens; one that cannot must not; arms
    /// cover every reason; `otherwise fail;` needs the reasons in the caller's list.
    fn check_handler(&self, handler: Option<&Handler>, fails: &[FailCase], name: &str, line: usize) -> Result<(), RuntimeError> {
        let list = || fails.iter().map(|f| f.name.as_str()).collect::<Vec<_>>().join(", ");
        match (handler, fails.is_empty()) {
            (None, false) => { let shape = if name == "concur" || name == "whichever" { format!("{name} {{ … }}") } else { format!("{name}[…]") }; return rt(line, format!("`{name}` can fail ({}); say what happens: `{shape} otherwise …`", list())) }
            (Some(_), true) => return rt(line, format!("`{name}` cannot fail; drop the `otherwise`")),
            (Some(Handler::Arms(arms)), false) => {
                for a in arms {
                    match fails.iter().find(|f| f.name == a.name) {
                        None => return rt(a.line, format!("`{name}` never fails with `{}` (it fails with {})", a.name, list())),
                        Some(f) => if a.bind.is_some() && f.ty.is_none() { return rt(a.line, format!("`{}` carries nothing; there is no value to name", a.name)); },
                    }
                }
                let missing: Vec<&str> = fails.iter().filter(|f| !arms.iter().any(|a| a.name == f.name)).map(|f| f.name.as_str()).collect();
                if !missing.is_empty() { return rt(line, format!("`{name}` can also fail with {}; every reason must be handled (or `otherwise fail;` to pass them up)", missing.join(", "))); }
            }
            // inside `must.fail`, the failure climbs out to the test (owner, 2026-09-29)
            (Some(Handler::Propagate), false) if self.must_fail_depth > 0 => {}
            (Some(Handler::Propagate), false) => {
                let missing: Vec<&str> = fails.iter().filter(|f| !self.fails.iter().any(|m| m.name == f.name)).map(|f| f.name.as_str()).collect();
                if !missing.is_empty() { return rt(line, format!("`otherwise fail;` passes {} up, but this function does not list {} in its `fail[…]`", list(), missing.join(", "))); }
            }
            _ => {}
        }
        Ok(())
    }

    /// The callee failed; do what the caller wrote after `otherwise`.
    fn handle_failure(&mut self, handler: &Handler, line: usize, name: &str, reason: String, payload: Option<Value>, value_used: bool) -> Result<Value, RuntimeError> {
        let leave = |f: Flow| RuntimeError { line, msg: String::new(), exit: None, site: false, leave: Some(f) };
        match handler {
            Handler::Value(b) => {
                // only a conversion's own failure: a type left by an earlier conversion that did not fail
                // must not reach another call's fallback
                let fty = self.fallback_ty.take().filter(|_| name.starts_with("to."));
                let saved = if fty.is_some() { Some(std::mem::replace(&mut self.expected, fty.clone())) } else { None };
                let r = self.bracket(b, line);
                if let Some(s) = saved { self.expected = s; }
                let (v, prov) = r?;
                match prov {
                    Prov::Bare(n) => rt(line, format!("the fallback is bare '{n}', which owns a {}: say `give '{n}'` or `copy '{n}'`", v.kind())),
                    _ => match &fty { Some(t) => self.fit(v, prov, Some(t), line), None => Ok(v) },
                }
            }
            Handler::Propagate => Err(leave(Flow::Fail(reason, payload))),
            Handler::Leave(s) => match self.stmt(s)? {
                Flow::Normal => rt(line, "the statement after `otherwise` must leave (return, fail, break, continue)"),
                f => Err(leave(f)),
            },
            Handler::Arms(arms) => {
                let arm = arms.iter().find(|a| a.name == reason).unwrap();
                self.arm_ty = None;
                match self.run_arm(arm, payload, !value_used)? {
                    // a case that gives a value is the value (owner, 2026-09-29)
                    Flow::Give(v) if value_used => Ok(v),
                    Flow::Give(_) => rt(arm.line, format!("the `{reason}` case gives a value, but the value of `{name}` is not used")),
                    Flow::Normal if value_used => rt(arm.line, format!("the `{reason}` arm must leave (return, fail, break, continue, or cord::exit) because the value of `{name}` is used")),
                    Flow::Normal => Ok(Value::Nothing),
                    f => Err(leave(f)),
                }
            }
        }
    }

    fn printrs_format(&mut self, fmt: &str, vals: &[Value], line: usize) -> Result<String, RuntimeError> {
        let chars: Vec<char> = fmt.chars().collect();
        let mut out = String::new();
        let mut i = 0;
        let mut next = 0;
        let mut used = vec![false; vals.len()];
        while i < chars.len() {
            let c = chars[i];
            if c == '\\' && i + 1 < chars.len() {
                match chars[i + 1] {
                    'n' => out.push('\n'), 't' => out.push('\t'), '\\' => out.push('\\'), '"' => out.push('"'),
                    other => return rt(line, format!("rs.print: unknown escape `\\{other}` in the format (it knows \\n \\t \\\\ \\\")")),
                }
                i += 2; continue;
            }
            if c == '{' && chars.get(i + 1) == Some(&'{') { out.push('{'); i += 2; continue; }
            if c == '}' && chars.get(i + 1) == Some(&'}') { out.push('}'); i += 2; continue; }
            if c == '}' { return rt(line, "rs.print: a lone `}` in the format (`}}` prints one)"); }
            if c != '{' { out.push(c); i += 1; continue; }
            // a hole: { [arg] [: spec] }
            i += 1;
            let mut arg = String::new();
            while i < chars.len() && chars[i] != ':' && chars[i] != '}' { arg.push(chars[i]); i += 1; }
            let mut spec = String::new();
            if i < chars.len() && chars[i] == ':' { i += 1; while i < chars.len() && chars[i] != '}' { spec.push(chars[i]); i += 1; } }
            if i >= chars.len() { return rt(line, "rs.print: the format ends inside a `{` hole"); }
            i += 1; // }
            let arg = arg.trim();
            let v: Value = if arg.is_empty() {
                let k = next; next += 1;
                match vals.get(k) { Some(v) => { used[k] = true; v.clone() }, None => return rt(line, format!("rs.print: hole #{} has no value — the format has more holes than values", k + 1)) }
            } else if let Ok(k) = arg.parse::<usize>() {
                match vals.get(k) { Some(v) => { used[k] = true; v.clone() }, None => return rt(line, format!("rs.print: `{{{k}}}` but only {} value(s) were given", vals.len())) }
            } else {
                // a name: a variable in scope (Rust's captured identifier)
                match self.lookup(arg) {
                    Some(slot) if slot.alive => slot.cell.borrow().clone(),
                    Some(_) => return rt(line, format!("rs.print: `{{{arg}}}` names '{arg}', which was given away")),
                    None => return rt(line, format!("rs.print: `{{{arg}}}` names no variable in scope (write it as '{arg}' in the program)")),
                }
            };
            // spec: [[fill]align][sign][#][0][width][.precision][type]
            let sc: Vec<char> = spec.chars().collect();
            let mut j = 0;
            let (mut fill, mut align) = (' ', None);
            if sc.len() >= 2 && matches!(sc[1], '<' | '>' | '^') { fill = sc[0]; align = Some(sc[1]); j = 2; }
            else if !sc.is_empty() && matches!(sc[0], '<' | '>' | '^') { align = Some(sc[0]); j = 1; }
            let mut plus = false; if j < sc.len() && sc[j] == '+' { plus = true; j += 1; }
            let mut alt = false; if j < sc.len() && sc[j] == '#' { alt = true; j += 1; }
            let mut zero = false; if j < sc.len() && sc[j] == '0' { zero = true; j += 1; }
            let mut width = String::new(); while j < sc.len() && sc[j].is_ascii_digit() { width.push(sc[j]); j += 1; }
            let width: Option<usize> = width.parse().ok();
            let mut prec: Option<usize> = None;
            if j < sc.len() && sc[j] == '.' { j += 1; let mut p = String::new(); while j < sc.len() && sc[j].is_ascii_digit() { p.push(sc[j]); j += 1; } prec = Some(p.parse().unwrap_or(0)); }
            let ty: String = sc[j..].iter().collect();
            let body = match (ty.as_str(), &v) {
                ("", Value::Bin(x)) => match prec { Some(p) => format!("{x:.p$}"), None => format!("{x}") },
                ("", Value::Str(t)) => match prec { Some(p) => t.chars().take(p).collect(), None => t.clone() },
                ("", other) => other.show_useful(),
                ("?", Value::Str(t)) => format!("{t:?}"),
                ("?", Value::Char(_, ch)) => format!("'{ch}'"),
                ("?", other) => other.show_useful(),
                ("x", Value::Int(n)) => if alt { format!("{n:#x}") } else { format!("{n:x}") },
                ("X", Value::Int(n)) => if alt { format!("{n:#X}") } else { format!("{n:X}") },
                ("o", Value::Int(n)) => if alt { format!("{n:#o}") } else { format!("{n:o}") },
                ("b", Value::Int(n)) => if alt { format!("{n:#b}") } else { format!("{n:b}") },
                ("e", Value::Bin(x)) => match prec { Some(p) => format!("{x:.p$e}"), None => format!("{x:e}") },
                ("E", Value::Bin(x)) => match prec { Some(p) => format!("{x:.p$E}"), None => format!("{x:E}") },
                ("x" | "X" | "o" | "b", other) => return rt(line, format!("rs.print: `{{:{ty}}}` wants an integer, got {}", other.kind())),
                ("e" | "E", other) => return rt(line, format!("rs.print: `{{:{ty}}}` wants a bin, got {}", other.kind())),
                (other, _) => return rt(line, format!("rs.print: unknown format type `{other}` (it knows ? x X o b e E)")),
            };
            let body = if plus && matches!(v, Value::Int(n) if n >= 0) || plus && matches!(v, Value::Bin(x) if x >= 0.0) { format!("+{body}") } else { body };
            let len = body.chars().count();
            let padded = match width {
                Some(w) if len < w => {
                    let pad = w - len;
                    if zero && matches!(v, Value::Int(_) | Value::Bin(_)) {
                        if let Some(rest) = body.strip_prefix('-') { format!("-{}{rest}", "0".repeat(pad)) }
                        else if let Some(rest) = body.strip_prefix('+') { format!("+{}{rest}", "0".repeat(pad)) }
                        else { format!("{}{body}", "0".repeat(pad)) }
                    } else {
                        let default_align = if matches!(v, Value::Int(_) | Value::Bin(_)) { '>' } else { '<' };
                        match align.unwrap_or(default_align) {
                            '<' => format!("{body}{}", fill.to_string().repeat(pad)),
                            '>' => format!("{}{body}", fill.to_string().repeat(pad)),
                            _ => { let l = pad / 2; format!("{}{body}{}", fill.to_string().repeat(l), fill.to_string().repeat(pad - l)) }
                        }
                    }
                }
                _ => body,
            };
            out.push_str(&padded);
        }
        if let Some(k) = used.iter().position(|u| !u) { return rt(line, format!("rs.print: value #{} has no hole — the format uses fewer values than were given", k + 1)); }
        Ok(out)
    }

    /// C's sizes and alignments; a normal struct gets the same rule (Cortex's real
    /// choice is not modelled), lists and strs are their 24-byte handle.
    fn size_align(&self, ty: &Ty, line: usize) -> Result<(usize, usize), RuntimeError> {
        Ok(match ty {
            Ty::Int(w) | Ty::Uint(w) => { let n = match w { IntWidth::Other(b) => ((*b as usize + 7) / 8).next_power_of_two(), IntWidth::W8 => 1, IntWidth::W16 => 2, IntWidth::W32 => 4, IntWidth::W64 => 8, IntWidth::W128 => 16 }; (n, n) }
            Ty::Bin(b) | Ty::Deci(b) => { let n = (*b as usize) / 8; (n, n) }
            Ty::Brain => (2, 2),
            Ty::Bool => (1, 1),
            Ty::Char(CharUnit::Byte) => (1, 1),
            Ty::Char(CharUnit::Point) => (4, 4),
            Ty::Char(_) | Ty::Str(_) | Ty::List(_) | Ty::Array(_) => (24, 8),
            Ty::Ptr(..) => (8, 8),
            Ty::Var(n) => return rt(line, format!("`size.of[<{n}>]`: a type variable has no size until it is filled")),
            Ty::Ranged(b, _, _) => return self.size_align(b, line),
            Ty::TextRanged(e, _, _, _) => return self.size_align(&Ty::Str(e.clone()), line),
            Ty::Map(..) => (24, 8),
            Ty::Simd(t, n) => { let (sz, _) = self.size_align(t, line)?; (sz * n, sz * n) }
            Ty::Channel(_) => (8, 8),
            Ty::Duration => (8, 8),
            Ty::Fn(..) => (8, 8),
            Ty::Inst(n, _) => return rt(line, format!("`size.of[{n}[…]]`: a generic struct's size is not modelled by the oracle")),
            Ty::Struct(n) => {
                let d = match self.prog.structs.iter().find(|d| &d.name == n) { Some(d) => d, None => return rt(line, format!("no struct named `{n}`")) };
                let (mut off, mut al) = (0usize, 1usize);
                for f in &d.fields {
                    let (s, a) = self.size_align(&f.ty, line)?;
                    off = (off + a - 1) / a * a + s;
                    al = al.max(a);
                }
                ((off + al - 1) / al * al, al)
            }
        })
    }

    /// ptr.read / ptr.write / ptr.at / ptr.is_null (design/ffi.md). The first three
    /// are nocheck sites; a pointer into a lent Cyborg value reads that value's
    /// elements (bytes, for a str).
    fn ptr_op(&mut self, op: &str, args: &Bracket, line: usize) -> Result<Value, RuntimeError> {
        let (pv, _) = self.value(&args.values[0])?;
        let p = match pv { Value::Ptr(p) => p, other => return rt(line, format!("`ptr.{op}` needs a pointer, got {}", other.kind())) };
        if op == "is_null" { return Ok(Value::Bool(p.cell.is_none())); }
        if self.nocheck_depth == 0 && self.marks_needed() { return rt(line, format!("`ptr.{op}` goes through a pointer the compiler cannot vouch for — it is a no_check site")); }
        let Some(cell) = p.cell.clone() else { return site(line, format!("`ptr.{op}` on a null pointer")) };
        match op {
            "at" => {
                let (n, _) = self.value(&args.values[1])?;
                let n = match n { Value::Int(n) => n, other => return rt(line, format!("`ptr.at` moves by an integer number of elements, got {}", other.kind())) };
                Ok(Value::Ptr(Ptr { cell: Some(cell), index: p.index + n, mutable: p.mutable }))
            }
            "read" => {
                let v = cell.borrow();
                let i = p.index;
                match &*v {
                    Value::List(items) => match usize::try_from(i).ok().and_then(|i| items.get(i)) { Some(x) => Ok(x.clone()), None => site(line, format!("`ptr.read` {i} element(s) in: past the end of what the pointer was lent ({} element(s))", items.len())) },
                    Value::Str(s) => match usize::try_from(i).ok().and_then(|i| s.as_bytes().get(i)) { Some(b) => Ok(Value::Int(*b as i128)), None if i as usize == s.len() => Ok(Value::Int(0)), None => site(line, format!("`ptr.read` {i} byte(s) in: past the end of the lent text ({} bytes)", s.len())) },
                    other => rt(line, format!("a pointer into {} cannot be read element-wise", other.kind())),
                }
            }
            "write" => {
                if !p.mutable { return rt(line, "`ptr.write` needs a `ptr.mut` (lend it with `lend.mut`)"); }
                let (nv, _) = self.value(&args.values[1])?;
                let mut v = cell.borrow_mut();
                let i = p.index;
                match &mut *v {
                    Value::List(items) => match usize::try_from(i).ok().filter(|i| *i < items.len()) { Some(i) => { items[i] = nv; Ok(Value::Nothing) }, None => site(line, format!("`ptr.write` {i} element(s) in: past the end of what the pointer was lent ({} element(s))", items.len())) },
                    other => rt(line, format!("a pointer into {} cannot be written element-wise", other.kind())),
                }
            }
            _ => unreachable!(),
        }
    }

    /// The oracle cannot link C. It emulates a few libc functions so extern
    /// programs can run: write, exit, getenv, strlen, malloc, free.
    fn extern_call(&mut self, func: &Func, c: &Call) -> Result<Value, RuntimeError> {
        let mut args = Vec::new();
        for (i, a) in c.args.values.iter().enumerate() {
            let (v, prov) = self.value(a)?;
            let param = func.params.get(i).or(func.params.last());
            let v = match (param.and_then(|p| p.ty.as_ref()), &v, prov) {
                (Some(Ty::Ptr(_, want_mut)), _, Prov::Borrowed(cell, is_mut)) => {
                    if *want_mut && !is_mut { return rt(c.line, format!("`{}` wants a `ptr.mut` for '{}': lend it with `lend.mut`", func.name, param.unwrap().name)); }
                    match &*cell.borrow() { Value::List(_) | Value::Str(_) => {}, other => return rt(c.line, format!("a pointer needs a list, array, or str behind it, not {}", other.kind())) }
                    Value::Ptr(Ptr { cell: Some(cell), index: 0, mutable: is_mut })
                }
                (Some(Ty::Ptr(..)), Value::Ptr(_), _) => v,
                (Some(Ty::Ptr(..)), other, _) => return rt(c.line, format!("`{}` wants a pointer for '{}': `lend` a list/array/str, or pass a pointer from C; got {}", func.name, param.unwrap().name, other.kind())),
                (Some(t @ Ty::Fn(..)), Value::Func(fv), _) => {
                    if !fv.captures.is_empty() { return rt(c.line, format!("`{}` is a callback for C, and `{}` captures [{}]: C has nowhere to keep them — pass them as a pointer argument instead", param.unwrap().name, fv.def.name, fv.captures.iter().map(|(n, _, _)| format!("'{n}'")).collect::<Vec<_>>().join(", "))); }
                    self.check_type(&v, t, c.line)?;
                    v
                }
                (Some(t), _, prov) => self.fit(v, prov, Some(t), c.line)?,
                (None, Value::Str(_) | Value::List(_) | Value::Char(..), _) => return rt(c.line, format!("`{}` is a variadic extern; its extra arguments must be C-shaped, not {}", func.name, v.kind())),
                (None, _, _) => v,
            };
            args.push(v);
        }
        let fixed = func.params.iter().filter(|p| !p.variadic).count();
        if args.len() < fixed || (args.len() > fixed && !func.params.last().map(|p| p.variadic).unwrap_or(false)) {
            return rt(c.line, format!("`{}` takes {} argument(s), given {}", func.name, fixed, args.len()));
        }
        let read_bytes = |p: &Ptr, len: Option<usize>| -> Vec<u8> {
            let Some(cell) = &p.cell else { return Vec::new() };
            let v = cell.borrow();
            let all: Vec<u8> = match &*v { Value::List(items) => items.iter().map(|x| if let Value::Int(i) = x { *i as u8 } else { 0 }).collect(), Value::Str(s) => s.as_bytes().to_vec(), _ => Vec::new() };
            let start = (p.index.max(0) as usize).min(all.len());
            let rest = &all[start..];
            match len { Some(n) => rest[..n.min(rest.len())].to_vec(), None => rest.iter().take_while(|b| **b != 0).cloned().collect() }
        };
        match func.name.as_str() {
            "write" => {
                let (fd, p, len) = match (&args[0], &args[1], &args[2]) { (Value::Int(fd), Value::Ptr(p), Value::Int(len)) => (*fd, p.clone(), *len as usize), _ => return rt(c.line, "write[fd, ptr, len]") };
                let bytes = read_bytes(&p, Some(len));
                let text = String::from_utf8_lossy(&bytes).to_string();
                if fd == 2 { eprint!("{text}"); } else { write!(self.out, "{text}").map_err(|e| RuntimeError { line: c.line, msg: e.to_string(), exit: None, site: false, leave: None })?; }
                Ok(Value::Int(bytes.len() as i128))
            }
            "exit" => match &args[0] { Value::Int(i) => Err(RuntimeError { line: c.line, msg: String::new(), exit: Some((*i & 0xff) as u8), site: false, leave: None }), _ => rt(c.line, "exit[code]") },
            "strlen" => match &args[0] { Value::Ptr(p) => Ok(Value::Int(read_bytes(p, None).len() as i128)), _ => rt(c.line, "strlen[ptr]") },
            "getenv" => {
                let name = match &args[0] { Value::Ptr(p) => String::from_utf8_lossy(&read_bytes(p, None)).to_string(), _ => return rt(c.line, "getenv[ptr]") };
                match std::env::var(&name) {
                    Ok(val) => { let mut bytes: Vec<Value> = val.bytes().map(|b| Value::Int(b as i128)).collect(); bytes.push(Value::Int(0)); Ok(Value::Ptr(Ptr { cell: Some(Rc::new(RefCell::new(Value::List(bytes)))), index: 0, mutable: false })) }
                    Err(_) => Ok(Value::Ptr(Ptr { cell: None, index: 0, mutable: false })),
                }
            }
            "qsort" => {
                // C's qsort, emulated: the comparator is a Cyborg function called with pointers to elements
                let (p, n, cmp) = match (&args[0], &args[1], &args[3]) { (Value::Ptr(p), Value::Int(n), Value::Func(f)) => (p.clone(), *n as usize, f.clone()), _ => return rt(c.line, "qsort[ptr.mut, n, size, cmp]") };
                let Some(cell) = p.cell.clone() else { return site(c.line, "qsort on a null pointer") };
                let start = p.index.max(0) as usize;
                let len = match &*cell.borrow() { Value::List(items) => items.len(), _ => return rt(c.line, "qsort needs a pointer into a list or array") };
                if start + n > len { return site(c.line, format!("qsort over {n} element(s) from {start}: past the end ({len})")); }
                // insertion sort, calling back for each comparison
                for i in (start + 1)..(start + n) {
                    let mut j = i;
                    while j > start {
                        let a = Value::Ptr(Ptr { cell: Some(cell.clone()), index: (j - 1) as i128, mutable: false });
                        let b = Value::Ptr(Ptr { cell: Some(cell.clone()), index: j as i128, mutable: false });
                        let r = self.call_func_value(&cmp, vec![a, b], c.line)?;
                        let order = match r { Value::Int(o) => o, other => return rt(c.line, format!("the comparator returned {}, not an int32", other.kind())) };
                        if order <= 0 { break; }
                        let mut v = cell.borrow_mut();
                        if let Value::List(items) = &mut *v { items.swap(j - 1, j); }
                        j -= 1;
                    }
                }
                Ok(Value::Nothing)
            }
            "malloc" => match &args[0] { Value::Int(n) => Ok(Value::Ptr(Ptr { cell: Some(Rc::new(RefCell::new(Value::List(vec![Value::Int(0); (*n).max(0) as usize])))), index: 0, mutable: true })), _ => rt(c.line, "malloc[bytes]") },
            "free" => Ok(Value::Nothing),
            other => rt(c.line, format!("extern `{other}`: the oracle cannot link C; it emulates write, exit, getenv, strlen, malloc, free")),
        }
    }


    /// Fill a struct's fields by name; every field required, types checked (a `<T>` field binds).
    fn construct_fields(&mut self, def: &StructDef, sname: &str, given: &[(String, Bracket)], line: usize) -> Result<(Value, Prov), RuntimeError> {
        for (k, _) in given { if !def.fields.iter().any(|f| &f.name == k) { return rt(line, format!("{sname} has no field `{k}`")); } }
        let mut out = Vec::new();
        for f in &def.fields {
            let b = match given.iter().find(|(k, _)| k == &f.name) {
                Some((_, b)) => b,
                None => return rt(line, format!("{sname} is missing `{}`: every field must be given — there are no zero values", f.name)),
            };
            if given.iter().filter(|(k, _)| k == &f.name).count() > 1 { return rt(line, format!("`{}` is given twice", f.name)); }
            let (v, prov) = self.bracket(b, line)?;
            match prov {
                Prov::Bare(n) => return rt(line, format!("bare '{n}' as field `{}`: say `give '{n}'` or `copy '{n}'`", f.name)),
                Prov::Borrowed(..) => return rt(line, format!("a struct owns its fields; `{}` cannot hold a lend", f.name)),
                _ => {}
            }
            let v = self.fit(v, prov, Some(&f.ty), line)?;
            out.push((f.name.clone(), v));
        }
        let val = Value::Struct(sname.to_string(), out);
        let prov = if val.is_owned() { Prov::Fresh } else { Prov::Plain };
        Ok((val, prov))

    }

    /// Build an enum value: the case must exist; data exactly when the case carries some.
    fn enum_value(&mut self, tname: &str, case: &str, args: Option<&Bracket>, from: &[String], line: usize) -> Result<(Value, Prov), RuntimeError> {
        let def = match self.prog.enums.iter().find(|e| e.name == tname) {
            Some(d) => d.clone(),
            None => return rt(line, format!("`{tname}` is not an enum (or a function `{tname}.{case}` — neither exists)")),
        };
        let from = if from.is_empty() { self.module.clone() } else { from.to_vec() };
        self.check_path(&from, &def.module, line)?;
        if !self.reachable(&from, &def.module, def.vis) { return rt(line, format!("enum `{tname}` is private to `{}`; mark it `pub` or `pub.neuron`", root_or(&def.module))); }
        let cdef = match def.cases.iter().find(|c| c.name == case) {
            Some(c) => c,
            None => return rt(line, format!("`{tname}` has no case `{case}` (it has {})", def.cases.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", "))),
        };
        let data = match (&cdef.ty, args) {
            (None, None) => None,
            (None, Some(b)) if b.values.is_empty() => None,
            (None, Some(_)) => return rt(line, format!("`{tname}.{case}` carries nothing; write it bare")),
            (Some(t), None) => return rt(line, format!("`{tname}.{case}` carries a {}: `{tname}.{case}[…]`", ty_name(t))),
            (Some(t), Some(b)) => {
                if b.values.len() != 1 { return rt(line, format!("`{tname}.{case}` carries one value")); }
                let (v, prov) = self.bracket(b, line)?;
                match prov {
                    Prov::Bare(n) => return rt(line, format!("bare '{n}' as the data of `{case}`: say `give '{n}'` or `copy '{n}'`")),
                    Prov::Borrowed(..) => return rt(line, "an enum owns its data; it cannot hold a lend"),
                    _ => {}
                }
                Some(Box::new(self.fit(v, prov, Some(t), line)?))
            }
        };
        Ok((Value::Enum(tname.to_string(), case.to_string(), data), Prov::Fresh))
    }

    fn std_call(&mut self, c: &Call) -> Result<Value, RuntimeError> {
        let path = c.path.join(".");
        let value_used = !self.call_is_stmt;
        self.call_is_stmt = false;
        // std's text toolbox (decided 2026-09-20: the toolbox is std's; the oracle
        // emulates a few so examples can run). The ones that can fail say so.
        let fails: Vec<FailCase> = match path.as_str() {
            "str.at.grapheme" | "str.at.point" | "str.at.byte" | "str.slice.grapheme" => vec![FailCase { name: "out_of_range".into(), ty: None }],
            "str.find.grapheme" => vec![FailCase { name: "missing".into(), ty: None }],
            _ => Vec::new(),
        };
        if path.starts_with("str.") {
            // one the oracle does not have says so first, before anything about its `otherwise` (check claude, 2026-10-08)
            let known = matches!(path.as_str(), "str.split" | "str.contains" | "str.starts_with" | "str.ends_with" | "str.trim" | "str.upper" | "str.lower" | "str.replace" | "str.join"
                | "str.at.grapheme" | "str.at.point" | "str.at.byte" | "str.slice.grapheme" | "str.find.grapheme");
            if !known { return rt(c.line, format!("cord::{path} is not in the oracle's cord (it has str.split/contains/starts_with/ends_with/trim/upper/lower/replace/join, str.at.<unit>, str.slice.grapheme, str.find.grapheme)")); }
            self.check_handler(c.handler.as_ref(), &fails, &format!("cord::{path}"), c.line)?;
            let mut vals = Vec::new();
            for a in &c.args.values {
                let (v, prov) = self.value(a)?;
                if let Prov::Bare(n) = prov { return rt(c.line, format!("cord::{path} looks at '{n}': `lend '{n}'`")); }
                vals.push(v);
            }
            let text = |i: usize| -> Result<String, RuntimeError> { match vals.get(i) { Some(Value::Str(s)) => Ok(s.clone()), Some(Value::Char(_, ch)) => Ok(ch.clone()), Some(o) => rt(c.line, format!("cord::{path}: argument {} must be text, got {}", i + 1, o.kind())), None => rt(c.line, format!("cord::{path}: missing argument {}", i + 1)) } };
            let int = |i: usize| -> Result<i128, RuntimeError> { match vals.get(i) { Some(Value::Int(n)) => Ok(*n), Some(o) => rt(c.line, format!("cord::{path}: argument {} must be an integer, got {}", i + 1, o.kind())), None => rt(c.line, format!("cord::{path}: missing argument {}", i + 1)) } };
            let fail = |me: &mut Self, reason: &str| -> Result<Value, RuntimeError> { me.handle_failure(c.handler.as_ref().unwrap(), c.line, &format!("cord::{path}"), reason.into(), None, value_used) };
            return match path.as_str() {
                "str.split" => { let (s, sep) = (text(0)?, text(1)?); if sep.is_empty() { return rt(c.line, "cord::str.split: an empty separator"); } Ok(Value::List(s.split(sep.as_str()).map(|p| Value::Str(p.to_string())).collect())) }
                "str.contains" => Ok(Value::Bool(text(0)?.contains(text(1)?.as_str()))),
                "str.starts_with" => Ok(Value::Bool(text(0)?.starts_with(text(1)?.as_str()))),
                "str.ends_with" => Ok(Value::Bool(text(0)?.ends_with(text(1)?.as_str()))),
                "str.trim" => Ok(Value::Str(text(0)?.trim().to_string())),
                "str.upper" => Ok(Value::Str(text(0)?.to_uppercase())),
                "str.lower" => Ok(Value::Str(text(0)?.to_lowercase())),
                "str.replace" => Ok(Value::Str(text(0)?.replace(text(1)?.as_str(), &text(2)?))),
                "str.join" => match vals.get(0) { Some(Value::List(items)) => { let sep = text(1)?; let mut parts = Vec::new(); for it in items { match it { Value::Str(s) => parts.push(s.clone()), o => return rt(c.line, format!("cord::str.join: the list holds {}, not text", o.kind())) } } Ok(Value::Str(parts.join(&sep))) } _ => rt(c.line, "cord::str.join[lend 'parts', sep]") },
                "str.at.grapheme" | "str.at.point" | "str.at.byte" => {
                    let s = text(0)?; let i = int(1)?;
                    let units: Vec<Value> = match path.as_str() { "str.at.grapheme" => graphemes(&s).into_iter().map(|g| Value::Char(CharUnit::Grapheme, g)).collect(), "str.at.point" => s.chars().map(|ch| Value::Char(CharUnit::Point, ch.to_string())).collect(), _ => s.bytes().map(|b| Value::Int(b as i128)).collect() };
                    match usize::try_from(i - 1).ok().and_then(|k| units.get(k)) { Some(v) if i >= 1 => Ok(v.clone()), _ => fail(self, "out_of_range") }
                }
                "str.slice.grapheme" => {
                    let s = text(0)?; let (a, b) = (int(1)?, int(2)?);
                    let gs = graphemes(&s);
                    if a < 1 || b < a || b as usize > gs.len() { return fail(self, "out_of_range"); }
                    Ok(Value::Str(gs[(a - 1) as usize..b as usize].concat()))
                }
                "str.find.grapheme" => {
                    let (s, sub) = (text(0)?, text(1)?);
                    match s.find(sub.as_str()) { Some(byte_ix) => Ok(Value::Int(grapheme_count(&s[..byte_ix]) as i128 + 1)), None => fail(self, "missing") }
                }
                other => rt(c.line, format!("cord::{other} is not in the oracle's cord (it has str.split/contains/starts_with/ends_with/trim/upper/lower/replace/join, str.at.<unit>, str.slice.grapheme, str.find.grapheme)")),
            };
        }
        if c.handler.is_some() && !path.starts_with("input.") { return rt(c.line, format!("cord::{path} cannot fail; drop the `otherwise`")); }
        match path.as_str() {
            "input.line" | "input.all" => {
                // a boundary (decided 2026-09-20): text as it came; `closed` when there is none
                if !c.args.values.is_empty() { return rt(c.line, format!("cord::{path} takes no arguments")); }
                let fails = vec![FailCase { name: "closed".into(), ty: None }];
                self.check_handler(c.handler.as_ref(), &fails, &format!("cord::{path}"), c.line)?;
                let mut text = String::new();
                let got = if path == "input.line" {
                    match std::io::stdin().read_line(&mut text) { Ok(0) => false, Ok(_) => { if text.ends_with('\n') { text.pop(); if text.ends_with('\r') { text.pop(); } } true }, Err(_) => false }
                } else {
                    use std::io::Read;
                    match std::io::stdin().read_to_string(&mut text) { Ok(0) => false, Ok(_) => true, Err(_) => false }
                };
                if !got { return self.handle_failure(c.handler.as_ref().unwrap(), c.line, &format!("cord::{path}"), "closed".into(), None, value_used); }
                Ok(Value::Str(text))
            }
            "round" | "floor" | "ceil" | "trunc" => {
                // the rounding toolbox (std, provisional names): a bin in, a whole bin out — then `to.int32[…]`
                if c.args.values.len() != 1 { return rt(c.line, format!("cord::{path} takes one bin")); }
                let (a, _) = self.value(&c.args.values[0])?;
                match a {
                    Value::Bin(x) => Ok(Value::Bin(no_new_neg_zero(x, match path.as_str() { "round" => x.round(), "floor" => x.floor(), "ceil" => x.ceil(), _ => x.trunc() }))),
                    other => rt(c.line, format!("cord::{path} takes a bin, got {}", other.kind())),
                }
            }
            "time.wait" => {
                // the oracle does not sleep: time is the outside world, and nothing here observes it
                if c.args.values.len() != 1 { return rt(c.line, "cord::time.wait takes one duration: `cord::time.wait[5 seconds]`"); }
                let (d, _) = self.value(&c.args.values[0])?;
                if !matches!(d, Value::Duration(_)) { return rt(c.line, format!("cord::time.wait takes a duration (`5 seconds`), got {}", d.kind())); }
                Ok(Value::Nothing)
            }
            "time.after" | "time.every" => {
                if c.args.values.len() != 1 { return rt(c.line, format!("cord::{path} takes one duration")); }
                let (d, _) = self.value(&c.args.values[0])?;
                let Value::Duration(n) = d else { return rt(c.line, format!("cord::{path} takes a duration (`5 seconds`), got {}", d.kind())) };
                self.next_chan_id += 1;
                let timer = if path == "time.after" { Timer::After { fired: false, nanos: n } } else { Timer::Every { nanos: n } };
                Ok(Value::Chan(Rc::new(RefCell::new(Chan { queue: std::collections::VecDeque::new(), cap: None, timer: Some(timer), id: self.next_chan_id }))))
            }
            "div.trunc" | "mod.trunc" => {
                // moved into the language as operators (owner, 2026-10-02)
                rt(c.line, format!("`cord::{path}` is an operator in the language now: `'a' {path} 'b'` (owner, 2026-10-02)"))
            }
            "printu.stdout" | "printu.stderr" => rt(c.line, format!("cord::{path} is spelled cord::print.useful.{} now (owner, 2026-09-29)", &path[7..])),
            "printf.stdout" | "printf.stderr" => rt(c.line, format!("cord::{path} is spelled cord::print.format.{} now (owner, 2026-09-29)", &path[7..])),
            // a print says which form it shows (owner, 2026-10-02): `print.stored` or `print.useful`, never bare
            "print.stdout" | "print.stderr" => rt(c.line, format!("say which form `cord::{path}` shows: `cord::print.stored.{0}[…]` (what is stored) or `cord::print.useful.{0}[…]` (the useful value) (owner, 2026-10-02)", &path[6..])),
            "print.weird.stdout" | "print.weird.stderr" => rt(c.line, format!("say which form `cord::{path}` shows: `cord::print.weird.stored.{0}` or `cord::print.weird.useful.{0}` (owner, 2026-10-02)", &path[12..])),
            "print.stored.stdout" | "print.useful.stdout" | "print.stored.stderr" | "print.useful.stderr"
            | "print.useful.auto_newline.stdout" | "print.useful.auto_newline.stderr"
            | "print.weird.stored.stdout" | "print.weird.stored.stderr" | "print.weird.useful.stdout" | "print.weird.useful.stderr"
            | "this_will_print_the_useful_numbers.this_will_go_to_the_standard_output"
            | "this_will_print_the_useful_numbers.this_will_go_to_the_standard_error" => {
                // Since 2026-09-19 print takes any number of values and renders each —
                // `print` in stored form, `print.useful` in useful form (`printu` until the owner
                // renamed it, 2026-09-29) — with nothing between. The long form is the owner's
                // answer to "printu is cryptic" (2026-09-21).
                // `print.weird` is `print` with arrows, `print.weird.useful` is `print.useful` (owner, 2026-10-01)
                let useful = path.starts_with("print.useful") || path.starts_with("print.weird.useful") || path.starts_with("this_will_print_the_useful");
                let path = if path.ends_with("standard_error") { "print.useful.stderr".to_string() } else { path };
                if c.args.values.is_empty() { return rt(c.line, format!("cord::{path} takes at least one value")); }
                let mut text = String::new();
                for a in &c.args.values {
                    // the useful form is the shortest text that gives the value back at its own
                    // width: a bin32 0.1 shows `0.1` (DECISIONS 2026-09-14)
                    let crate::ast::Value::Expr(e) = a;
                    let saved = self.expected.take(); let ty = self.expr_ty(e); self.expected = saved;
                    let (v, _) = self.value_mode(a, !useful)?;
                    if matches!(v, Value::Nothing) { return rt(c.line, "a call that returns nothing cannot be printed"); }
                    text.push_str(&if useful { show_useful_as(&v, ty.as_ref()) } else { v.show_stored() });
                }
                // `.auto_newline` ends the line after the values (owner, 2026-09-30)
                if path.contains(".auto_newline.") { text.push('\n'); }
                if path.ends_with("stderr") { eprint!("{text}"); }
                else { write!(self.out, "{text}").map_err(|e| RuntimeError { line: c.line, msg: e.to_string(), exit: None, site: false, leave: None })?; }
                Ok(Value::Nothing)
            }
            "format" | "print.format.stdout" | "print.format.stderr" => {
                // The formatting print (owner, 2026-09-22; `printf` until 2026-09-29; `{}` holes since
                // 2026-10-01, C's `%` codes before): escapes and holes, checked against the values'
                // types because the format is a literal — a non-literal format is a nocheck site.
                if c.args.values.is_empty() { return rt(c.line, format!("cord::{path} takes a format text first: `cord::{path}[\"{{}} items\\n\", 'n']`")); }
                let fmt = match &c.args.values[0] {
                    crate::ast::Value::Expr(Expr::Text(t, _)) => t.clone(),
                    _ => {
                        if self.nocheck_depth == 0 && self.marks_needed() { return rt(c.line, format!("cord::{path}: the format is not a literal, so its holes cannot be checked against the values — that is a no_check site: `no_check cord::{path}[…]`")); }
                        match self.value(&c.args.values[0])?.0 { Value::Str(t) => t, other => return rt(c.line, format!("cord::{path} takes a text as its format, got {}", other.kind())) }
                    }
                };
                let mut vals = Vec::new();
                for a in &c.args.values[1..] {
                    let crate::ast::Value::Expr(e) = a;
                    let saved = self.expected.take(); let ty = self.expr_ty(e); self.expected = saved;
                    vals.push((self.value(a)?.0, ty));
                }
                let text = hole_format(&fmt, &vals).map_err(|m| RuntimeError { line: c.line, msg: format!("cord::{path}: {m}"), exit: None, site: false, leave: None })?;
                if path == "format" { return Ok(Value::Str(text)); }
                if path.ends_with("stderr") { eprint!("{text}"); }
                else { write!(self.out, "{text}").map_err(|e| RuntimeError { line: c.line, msg: e.to_string(), exit: None, site: false, leave: None })?; }
                Ok(Value::Nothing)
            }
            "printc.stdout" | "printc.stderr" | "printrs.stdout" | "printrs.stderr" | "c.printf" | "c.fprintf.stderr"
            | "rs.print" | "rs.println" | "rs.eprint" | "rs.eprintln" =>
                rt(c.line, format!("cord::{path} was removed (2026-09-20): the borrowed C/Rust prints are gone; use cord::print.stored / cord::print.useful")),
            "exit" => {
                let (v, _) = self.single(&c.args, c.line)?;
                match v {
                    Value::Int(i) if (0..=255).contains(&i) => Err(RuntimeError { line: c.line, msg: String::new(), exit: Some(i as u8), site: false, leave: None }),
                    Value::Int(i) => site(c.line, format!("exit code {i} does not fit uint8 (0 to 255)")),
                    o => rt(c.line, format!("cord::exit takes a uint8, got {}", o.kind())),
                }
            }
            other => rt(c.line, format!("cord::{other} does not exist in the oracle (it knows print, print.useful, print.useful.auto_newln and print.format, each .stdout/.stderr; input.line/all; exit; round/floor/ceil/trunc; div.trunc, mod.trunc; time.after/every/wait; str.*)")),
        }
    }
}

/// A type's shape, matching `shape_of`: the kind, or the struct/enum name.
fn ty_shape(t: &Ty) -> String {
    match t {
        Ty::Int(_) | Ty::Uint(_) => "integer".into(), Ty::Bin(_) => "bin".into(), Ty::Brain => "brain16".into(), Ty::Deci(_) => "deci".into(), Ty::Bool => "bool".into(),
        Ty::Str(_) => "str".into(), Ty::Char(_) => "char".into(), Ty::List(_) | Ty::Array(_) => "list".into(),
        Ty::Ptr(..) => "ptr".into(), Ty::Struct(n) | Ty::Inst(n, _) => n.clone(), Ty::Var(n) => format!("<{n}>"), Ty::Ranged(b, _, _) => ty_shape(b), Ty::TextRanged(..) => "str".into(), Ty::Map(..) => "map".into(), Ty::Simd(..) => "simd".into(), Ty::Channel(_) => "channel".into(), Ty::Duration => "duration".into(), Ty::Fn(..) => "function".into(),
    }
}

/// A value's shape, for binding a `<T>`: the kind, or the struct/enum name.
fn shape_of(v: &Value) -> String {
    match v { Value::Struct(n, _) | Value::Enum(n, _, _) => n.clone(), other => other.kind().to_string() }
}

/// Replace type variables in `ty` with what a generic struct's use filled them with.
fn subst(ty: &Ty, map: &HashMap<String, Ty>) -> Ty {
    match ty {
        Ty::Var(n) => map.get(n).cloned().unwrap_or_else(|| ty.clone()),
        Ty::List(t) => Ty::List(Box::new(subst(t, map))),
        Ty::Array(t) => Ty::Array(Box::new(subst(t, map))),
        Ty::Ptr(t, m) => Ty::Ptr(Box::new(subst(t, map)), *m),
        Ty::Inst(n, args) => Ty::Inst(n.clone(), args.iter().map(|a| subst(a, map)).collect()),
        other => other.clone(),
    }
}

/// IEEE round-to-nearest-even of an f64 to binary16: 11 significant bits, subnormals below
/// 2^-14 in steps of 2^-24; past 65504 it rounds to infinity (65520 and up).
fn round_f16(x: f64) -> f64 {
    if !x.is_finite() || x == 0.0 { return x; }
    let a = x.abs();
    let e = ((a.to_bits() >> 52) & 0x7ff) as i32 - 1023;
    let ulp = if a >= 2f64.powi(-14) { 2f64.powi(e - 10) } else { 2f64.powi(-24) };
    let r = (a / ulp).round_ties_even() * ulp;
    let r = if r > 65504.0 { f64::INFINITY } else { r };
    r.copysign(x)
}

/// The digits of a bin literal as written (with a leading `-`), looking through parentheses.
/// Without ASP a literal is a bit pattern, as the machine's: an `int8` holding `128` holds -128's bits,
/// and one wider than the type's bits is refused (owner, 2026-10-06: "`a`", "Brainstem's, then").
fn raw_land(v: Value, prov: Prov, t: &Ty, line: usize) -> Result<(Value, Prov), RuntimeError> {
    match (&prov, &v, t) {
        (Prov::Raw, Value::Int(i), Ty::Int(w) | Ty::Uint(w)) if int_range(t).is_some() => {
            let bits = match w { IntWidth::W8 => 8, IntWidth::W16 => 16, IntWidth::W32 => 32, IntWidth::W64 => 64, IntWidth::W128 => 128, IntWidth::Other(n) => *n };
            if *i < 0 || (bits < 128 && *i >= 1i128 << bits) {
                return rt(line, format!("`{i}` does not fit the {bits} bits of {}: without ASP a literal is a bit pattern, 0 to {}", ty_name(t), if bits < 128 { ((1i128 << bits) - 1).to_string() } else { "2^128 - 1".into() }));
            }
            let v = if matches!(t, Ty::Int(_)) && bits < 128 && *i >= 1i128 << (bits - 1) { *i - (1i128 << bits) } else { *i };
            Ok((Value::Int(v), Prov::Literal))
        }
        (Prov::Raw, Value::Int(_), Ty::Int(_) | Ty::Uint(_)) => rt(line, format!("a literal landing in {} without ASP is not built in the oracle", ty_name(t))),
        (Prov::Raw, _, Ty::Bin(_) | Ty::Brain | Ty::Deci(_) | Ty::Ranged(..)) => rt(line, format!("what a literal means in {} without ASP is not decided: Brainstem's literals are bit patterns (owner, 2026-10-06), decided for whole-number types only", ty_name(t))),
        (Prov::Raw, _, _) => Ok((v, Prov::Literal)),
        _ => Ok((v, prov)),
    }
}

/// The key of a `pub` thing's `bare` mark: its kind, module and name (`fn:shapes.area`).
pub fn bare_key(kind: &str, module: &[String], name: &str) -> String {
    let mut p = module.join(".");
    if !p.is_empty() { p.push('.'); }
    format!("{kind}:{p}{name}")
}

fn lit_digits(e: &Expr) -> Option<String> {
    match e {
        Expr::Bin(_, t, _) => Some(t.clone()),
        Expr::Paren(inner, _) => lit_digits(inner),
        Expr::Unary(op, inner, _) if op == "-" => lit_digits(inner).map(|t| match t.strip_prefix('-') { Some(p) => p.to_string(), None => format!("-{t}") }),
        _ => None,
    }
}

/// A type as memory lays it out: range bounds and text bounds do not change it.
fn layout_name(t: &Ty) -> String {
    match t {
        Ty::Ranged(b, _, _) => layout_name(b),
        Ty::TextRanged(e, _, _, _) => ty_name(&Ty::Str(e.clone())),
        Ty::List(e) => format!("list[{}]", layout_name(e)),
        Ty::Array(e) => format!("array[{}]", layout_name(e)),
        t => ty_name(t),
    }
}

fn bin_w(t: &Ty) -> Option<u8> { match t { Ty::Bin(w) => Some(*w), Ty::Ranged(b, _, _) => bin_w(b), _ => None } }

/// A decimal with an optional exponent (`2.5e-3`), written out plainly (`0.0025`).
fn plain_decimal(t: &str) -> String {
    let (sign, t) = match t.strip_prefix('-') { Some(r) => ("-", r), None => ("", t.strip_prefix('+').unwrap_or(t)) };
    let (mant, exp) = match t.find(['e', 'E']) { Some(i) => (&t[..i], t[i + 1..].parse::<i64>().unwrap_or(0)), None => (t, 0) };
    let (int, frac) = mant.split_once('.').unwrap_or((mant, ""));
    let digits = format!("{int}{frac}");
    let point = int.len() as i64 + exp;               // where the decimal point falls in `digits`
    let body = if point <= 0 { format!("0.{}{digits}", "0".repeat((-point) as usize)) }
        else if point as usize >= digits.len() { format!("{digits}{}", "0".repeat(point as usize - digits.len())) }
        else { format!("{}.{}", &digits[..point as usize], &digits[point as usize..]) };
    format!("{sign}{body}")
}

/// Compare two plain decimals (digits, one optional `.`) by magnitude.
fn cmp_decimal(a: &str, b: &str) -> std::cmp::Ordering {
    let split = |x: &str| { let x = x.trim_start_matches('-'); let (i, f) = x.split_once('.').unwrap_or((x, "")); (i.trim_start_matches('0').to_string(), f.trim_end_matches('0').to_string()) };
    let ((ai, af), (bi, bf)) = (split(a), split(b));
    ai.len().cmp(&bi.len()).then_with(|| ai.cmp(&bi)).then_with(|| af.cmp(&bf))
}

/// A decimal written in source, rounded once to a bin of width `w` (IEEE nearest-even), not
/// first to 64 bits and then again: the two roundings can land one step apart.
fn round_decimal(d: &str, w: u8) -> f64 {
    let x: f64 = d.parse().unwrap_or(f64::NAN);
    match w {
        32 => d.parse::<f32>().map(|f| f as f64).unwrap_or(x),
        16 if x.is_finite() && x != 0.0 => {
            // reading at 64 bits first misleads only when it lands exactly halfway between two
            // bin16 values; then the written digits decide which way
            let a = x.abs();
            let e = ((a.to_bits() >> 52) & 0x7ff) as i32 - 1023;
            let ulp = if a >= 2f64.powi(-14) { 2f64.powi(e - 10) } else { 2f64.powi(-24) };
            let q = a / ulp;
            if q.fract() == 0.5 {
                let exact = Value::Bin(a).show_stored();
                let r = match cmp_decimal(d, &exact) {
                    std::cmp::Ordering::Greater => (q.floor() + 1.0) * ulp,
                    std::cmp::Ordering::Less => q.floor() * ulp,
                    std::cmp::Ordering::Equal => q.round_ties_even() * ulp,
                };
                let r = if r > 65504.0 { f64::INFINITY } else { r };
                return r.copysign(x);
            }
            round_f16(x)
        }
        16 => x,
        _ => x,
    }
}

/// The useful form of a value whose static type is known: a bin16/bin32 is the shortest decimal
/// that rounds back to the same value at that width, not at 64 bits.
/// The whole quotient of `x / y`, rounded `floor`, `ceil` or `trunc`, and what it leaves over
/// (owner, 2026-10-02). C's `fmod` is exact, so the truncated pair is exact; one step away fixes
/// the others. The quotient is the true one (`1e-20 div.ceil 2.0` is 1, `-1e-20 div.floor 360.0`
/// is -1, as Python's `//`). A remainder that rounds to the divisor itself (`-1e-20 mod.floor
/// 360.0`) is `0` instead, so it stays below the divisor (owner, 2026-10-02: b); the caller
/// checks that again after rounding to a narrower type. A zero `mod.floor` takes the divisor's sign.
/// `div.floor` … `mod.near`: the division operators, each saying how it rounds (owner, 2026-10-02)
pub fn is_div_mod(op: &str) -> bool {
    matches!(op.split_once('.'), Some(("div" | "mod", "floor" | "ceil" | "trunc" | "near")))
}

fn float_div_mod(x: f64, y: f64, mode: &str) -> (f64, f64) {
    let r0 = x % y;
    let q0 = ((x - r0) / y).round();
    if r0 == 0.0 { return (q0, if mode == "floor" { 0.0f64.copysign(y) } else { r0 }); }
    let up = (r0 < 0.0) == (y < 0.0);
    let away = match mode { "floor" => !up, "ceil" => up, _ => false };
    if !away { return (q0, r0); }
    let (q, r) = if up { (q0 + 1.0, r0 - y) } else { (q0 - 1.0, r0 + y) };
    (q, below_divisor(r, y, mode))
}

/// A remainder that rounded to the divisor itself is `0` (owner, 2026-10-02: b).
fn below_divisor(r: f64, y: f64, mode: &str) -> f64 {
    if r.abs() == y.abs() { if mode == "floor" { 0.0f64.copysign(y) } else { 0.0 } } else { r }
}

fn show_useful_as(v: &Value, ty: Option<&Ty>) -> String {
    let ty = match ty { Some(Ty::Ranged(b, _, _)) => Some(&**b), t => t };
    match (v, ty) {
        (Value::Bin(x), Some(Ty::Bin(w @ (16 | 32)))) if x.is_finite() => {
            let back = |y: f64| if *w == 32 { (y as f32) as f64 } else { round_f16(y) };
            for digits in 0..17 {
                let t = format!("{:.*e}", digits, x);
                if let Ok(y) = t.parse::<f64>() { if back(y) == *x { return format!("{y}"); } }
            }
            format!("{x}")
        }
        (Value::List(items), Some(Ty::List(e) | Ty::Array(e))) => format!("[{}]", items.iter().map(|i| show_useful_as(i, Some(e))).collect::<Vec<_>>().join(", ")),
        (v, _) => v.show_useful(),
    }
}

/// Round every bin inside a value to the width its type gives it (bin16, bin32); bin64 is as held.
/// A bin or brain16 -0 made plain 0, lane by lane (owner, 2026-10-07: "`a`", no -0 in any result).
fn no_neg_zero(v: Value) -> Value {
    match v {
        Value::Bin(x) => Value::Bin(x + 0.0),
        Value::Brain(x) => Value::Brain(x + 0.0),
        Value::Simd(ls) => Value::Simd(ls.into_iter().map(no_neg_zero).collect()),
        o => o,
    }
}

/// Rounding never makes a -0 of a number that was not zero (owner, 2026-10-07: "`a`"); a -0 the
/// `.ieee` forms made stays one.
fn no_new_neg_zero(before: f64, after: f64) -> f64 { if after == 0.0 && before != 0.0 { 0.0 } else { after } }

fn round_to_width(t: &Ty, v: Value) -> Value {
    match (t, v) {
        (Ty::Bin(32), Value::Bin(x)) => Value::Bin(no_new_neg_zero(x, (x as f32) as f64)),
        (Ty::Bin(16), Value::Bin(x)) => Value::Bin(no_new_neg_zero(x, round_f16(x))),
        (Ty::Ranged(b, _, _), v) => round_to_width(b, v),
        (Ty::List(e) | Ty::Array(e), Value::List(items)) => Value::List(items.into_iter().map(|i| round_to_width(e, i)).collect()),
        (Ty::Simd(e, _), Value::Simd(lanes)) => Value::Simd(lanes.into_iter().map(|i| round_to_width(e, i)).collect()),
        (_, v) => v,
    }
}

/// Whether an operation on two i128s would overflow i128 (and so needs big numbers).
fn int_overflows(op: &str, a: &Value, b: &Value) -> bool {
    let (Value::Int(x), Value::Int(y)) = (a, b) else { return false };
    match op {
        "+" => x.checked_add(*y).is_none(),
        "-" => x.checked_sub(*y).is_none(),
        "x" => x.checked_mul(*y).is_none(),
        "xx" => *y >= 0 && { let mut acc: i128 = 1; let mut over = false; for _ in 0..*y { match acc.checked_mul(*x) { Some(v) => acc = v, None => { over = true; break; } } if acc == 0 || acc == 1 && (*x == 1) { break; } } over },
        o if crate::interp::is_div_mod(o) => *x == i128::MIN && *y == -1,
        _ => false,
    }
}

/// An integer operation on big numbers (owner, 2026-10-03). None: not an integer operation here.
fn big_binary(op: &str, a: &Value, b: &Value, line: usize) -> Result<Option<Value>, RuntimeError> {
    use crate::bigint::BigInt;
    let (x, y) = (as_big(a).unwrap(), as_big(b).unwrap());
    Ok(Some(match op {
        "+" => from_big(x.add(&y)),
        "-" => from_big(x.sub(&y)),
        "x" => from_big(x.mul(&y)),
        "xx" => {
            if y.is_neg() { return site(line, "a negative exponent on integers would produce a fraction — a check site: Cortex proves the exponent is never negative, or the final build refuses"); }
            let e = y.to_u64().filter(|e| x.bits().saturating_mul(*e) <= 16_777_216 || x.bits() <= 1);
            match e { Some(e) => from_big(x.pow(e)), None => return site(line, format!("{x} to the power {y} is past the widest integer there is (2^23 bits)")) }
        }
        "==" => Value::Bool(x.cmp(&y).is_eq()),
        "!==" => Value::Bool(!x.cmp(&y).is_eq()),
        "<" => Value::Bool(x.cmp(&y).is_lt()),
        ">" => Value::Bool(x.cmp(&y).is_gt()),
        "</==" => Value::Bool(x.cmp(&y).is_le()),
        ">/==" => Value::Bool(x.cmp(&y).is_ge()),
        o if is_div_mod(o) => {
            let (kind, mode) = o.split_once('.').unwrap();
            let Some((mut q, mut r)) = x.divrem_trunc(&y) else { return site(line, if kind == "div" { "division by zero" } else { "`mod` by zero" }) };
            if !r.is_zero() {
                let up = r.is_neg() == y.is_neg();   // the true quotient is above q
                let away = match mode {
                    "floor" => !up, "ceil" => up, "trunc" => false,
                    _ => {
                        let abs = |v: &BigInt| if v.is_neg() { v.neg() } else { v.clone() };
                        match abs(&r.add(&r)).cmp(&abs(&y)) { std::cmp::Ordering::Greater => true, std::cmp::Ordering::Less => false, _ => !q.mod_pow2(1).is_zero() }
                    }
                };
                let one = BigInt::from_i128(1);
                if away { if up { q = q.add(&one); r = r.sub(&y); } else { q = q.sub(&one); r = r.add(&y); } }
            }
            from_big(if kind == "div" { q } else { r })
        }
        _ => return Ok(None),
    }))
}

/// An integer value as a big number (owner, 2026-10-03).
pub fn as_big(v: &Value) -> Option<crate::bigint::BigInt> {
    match v { Value::Int(i) => Some(crate::bigint::BigInt::from_i128(*i)), Value::Big(b) => Some(b.clone()), _ => None }
}
/// A big number back as a value: `Int` whenever it fits i128.
pub fn from_big(b: crate::bigint::BigInt) -> Value { match b.to_i128() { Some(i) => Value::Int(i), None => Value::Big(b) } }

/// An integer type's true width (`int_bits` gives 127 for 128, for the i128 wrapping path).
fn true_bits(w: &IntWidth) -> u32 { match w { IntWidth::W128 => 128, w => int_bits(w) } }
fn int_bits(w: &IntWidth) -> u32 { match w { IntWidth::Other(n) => *n, IntWidth::W8 => 8, IntWidth::W16 => 16, IntWidth::W32 => 32, IntWidth::W64 => 64, IntWidth::W128 => 127 } }

fn kind_word(k: ParamKind) -> &'static str { match k { ParamKind::Owned => "owning", ParamKind::Lent => "lent", ParamKind::LentMut => "lent.mut" } }

/// `min` / `max` in a range type are the base type's own limits (owner, 2026-09-23):
/// `int32[1, max]` is "at least 1", and stays right if the width changes. The word says
/// which limit, not the side it is written on: `int32[max, 5]` is not "up to 5" (compiler
/// session, 2026-09-23 — the side had been read instead of the word).
pub fn resolve_bound(base: &Ty, b: &Bound) -> Bound {
    let high = matches!(b, Bound::Max);
    match b {
        Bound::Min | Bound::Max => match base {
            Ty::Int(_) | Ty::Uint(_) => {
                let (lo, hi) = crate::value::int_range(base).unwrap_or((i128::MIN, i128::MAX));
                Bound::Int(if high { hi } else { lo })
            }
            Ty::Bin(w) => {
                let m = match w { 16 => 65504.0, 32 => f32::MAX as f64, _ => f64::MAX };
                Bound::Bin(if high { m } else { -m })
            }
            // a deci's largest (9.999999e96 for deci32; past a bin64 for the wider ones)
            Ty::Deci(w) => {
                let m = if *w == 32 { 9.999999e96 } else { f64::INFINITY };
                Bound::Bin(if high { m } else { -m })
            }
            // a list's, a map's or a channel's count: never negative, no natural ceiling
            _ => Bound::Int(if high { i128::MAX } else { 0 }),
        },
        other => other.clone(),
    }
}

fn const_key(module: &[String], name: &str) -> String { format!("{}/{name}", module.join(".")) }
fn join_path(module: &[String], name: &str) -> String { if module.is_empty() { name.to_string() } else { format!("{}.{name}", module.join(".")) } }
fn root_or(module: &[String]) -> String { if module.is_empty() { "the root (neuron.nrv)".into() } else { module.join(".") } }

/// The first `std::` call inside a const's value, if any (an effect at compile time).
fn first_std_call(b: &Bracket) -> Option<String> {
    fn in_expr(e: &Expr) -> Option<String> {
        match e {
            Expr::Call(c) => { if c.limb.is_some() { return Some(c.path.join(".")); } in_bracket(&c.args) }
            Expr::Nested(b) => in_bracket(b),
            Expr::Builtin(_, _, b, _) => in_bracket(b),
            Expr::Construct(_, fs, _) => fs.iter().find_map(|(_, b)| in_bracket(b)),
            Expr::Index(_, i, _) => in_expr(i),
            Expr::Unary(_, i, _) | Expr::Paren(i, _) => in_expr(i),
            Expr::Binary(_, l, r, _) => in_expr(l).or_else(|| in_expr(r)),
            _ => None,
        }
    }
    fn in_bracket(b: &Bracket) -> Option<String> {
        b.values.iter().find_map(|v| match v { crate::ast::Value::Expr(e) => in_expr(e) })
    }
    in_bracket(b)
}

fn same_family(a: &Value, b: &Value, line: usize, op: &str) -> Result<(), RuntimeError> {
    let ok = matches!((a, b),
        (Value::Int(_), Value::Int(_)) | (Value::Bin(_), Value::Bin(_)) | (Value::Brain(_), Value::Brain(_)) | (Value::Bool(_), Value::Bool(_))
        | (Value::Str(_), Value::Str(_)) | (Value::Char(..), Value::Char(..)) | (Value::List(_), Value::List(_))
        | (Value::Enum(..), Value::Enum(..)) | (Value::Struct(..), Value::Struct(..)));
    if ok { Ok(()) } else { rt(line, format!("`{op}` between {} and {}: no implicit conversion", a.kind(), b.kind())) }
}

/// Grapheme clusters, approximated: a base character plus any combining
/// marks, ZWJ sequences, variation selectors, and skin-tone modifiers. Enough
/// for the oracle to tell "one" from "several"; the real runtime needs UAX #29.
/// Split into graphemes with the same approximation as `grapheme_count`.
fn graphemes(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut prev_zwj = false;
    for ch in s.chars() {
        let cp = ch as u32;
        let combining = (0x0300..=0x036F).contains(&cp) || (0x0E31..=0x0E3A).contains(&cp) || (0x0E47..=0x0E4E).contains(&cp)
            || (0x1AB0..=0x1AFF).contains(&cp) || (0x20D0..=0x20FF).contains(&cp) || (0xFE00..=0xFE0F).contains(&cp)
            || (0x1F3FB..=0x1F3FF).contains(&cp);
        if cp == 0x200D { prev_zwj = true; if let Some(last) = out.last_mut() { last.push(ch); } continue; }
        if (combining || prev_zwj) && !out.is_empty() { out.last_mut().unwrap().push(ch); } else { out.push(ch.to_string()); }
        prev_zwj = false;
    }
    out
}

pub fn grapheme_count(s: &str) -> usize {
    let mut n = 0;
    let mut prev_zwj = false;
    for ch in s.chars() {
        let cp = ch as u32;
        let combining = (0x0300..=0x036F).contains(&cp) || (0x0E31..=0x0E3A).contains(&cp) || (0x0E47..=0x0E4E).contains(&cp)
            || (0x1AB0..=0x1AFF).contains(&cp) || (0x20D0..=0x20FF).contains(&cp) || (0xFE00..=0xFE0F).contains(&cp)
            || (0x1F3FB..=0x1F3FF).contains(&cp);
        if cp == 0x200D { prev_zwj = true; continue; }
        if !combining && !prev_zwj { n += 1; }
        prev_zwj = false;
    }
    n
}



/// C's format language, as `std::print.format` accepts it (design/strings.md, 2026-09-22): the
/// escapes that mean a character, and `%` with flags `-`, `0`, a width, `.precision`, and the
/// conversions d i s f x X c %. Every `%` is checked against its argument's type and the
/// count — the mismatch C leaves undefined is an error here.
/// `std::print.format`'s holes (owner, 2026-10-01; C's `%` codes until then): `{}` takes the next
/// value in its readable form, `{1}`, `{2}`… a numbered one (counted from 1, never mixed with `{}`),
/// and `:` adds a shape — `[[fill]align][+][0][width][.precision][kind]`, kind one of x X o b e `?`
/// (`?` is the stored form). `\{` and `\}` are literal braces; unknown escapes are errors.
fn hole_format(fmt: &str, vals: &[(Value, Option<Ty>)]) -> Result<String, String> {
    let chars: Vec<char> = fmt.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    let mut next = 0usize;
    let mut used = vec![false; vals.len()];
    let (mut plain, mut numbered) = (false, false);
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\\' {
            i += 1;
            let e = *chars.get(i).ok_or("a `\\` at the end of the format escapes nothing")?;
            match e {
                'n' => out.push('\n'), 't' => out.push('\t'), 'r' => out.push('\r'), '\\' => out.push('\\'),
                '"' => out.push('"'), '\'' => out.push('\''), '0' => out.push('\0'),
                '{' => out.push('{'), '}' => out.push('}'),
                '%' => return Err("`\\%` is not an escape: `%` is plain text in a format, so write `%` (and `\\\\%` for a backslash and a percent)".into()),
                'x' => {
                    let hex: String = chars[i + 1..].iter().take(2).collect();
                    let b = u8::from_str_radix(&hex, 16).map_err(|_| format!("`\\x` needs two hex digits, found `\\x{hex}`"))?;
                    out.push(b as char); i += hex.len();
                }
                'u' | 'U' => {
                    let n = if e == 'u' { 4 } else { 8 };
                    let hex: String = chars[i + 1..].iter().take(n).collect();
                    let cp = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32).ok_or_else(|| format!("`\\{e}` needs {n} hex digits naming a character, found `\\{e}{hex}`"))?;
                    out.push(cp); i += n;
                }
                other => return Err(format!("`\\{other}` is not an escape (n t r \\\\ \\\" \\' 0 {{ }} xHH uHHHH UHHHHHHHH are)")),
            }
            i += 1; continue;
        }
        if ch == '}' { return Err("a `}` with no `{` before it; a literal brace is `\\}`".into()); }
        if ch != '{' { out.push(ch); i += 1; continue; }
        // a hole: {[n][:shape]}
        i += 1;
        let close = chars[i..].iter().position(|c| *c == '}').ok_or("a `{` that never closes; a literal brace is `\\{`")?;
        let inner: String = chars[i..i + close].iter().collect();
        i += close + 1;
        let (index, shape) = match inner.split_once(':') { Some((a, b)) => (a.to_string(), Some(b.to_string())), None => (inner.clone(), None) };
        let at = if index.is_empty() {
            plain = true; next += 1; next
        } else {
            numbered = true;
            let n: usize = index.parse().map_err(|_| format!("`{{{inner}}}`: a hole is `{{}}`, `{{1}}`, `{{2}}`…, optionally with `:` and a shape"))?;
            if n == 0 { return Err("`{0}` does not exist: holes count from `{1}`, as positions do".into()); }
            n
        };
        if plain && numbered { return Err("this format mixes `{}` with numbered holes; use one or the other".into()); }
        let (v, ty) = vals.get(at - 1).ok_or_else(|| if numbered { format!("`{{{at}}}` names value {at}, but only {} were given", vals.len()) } else { format!("the format has more holes than values (value {at} is missing)") })?;
        used[at - 1] = true;
        out.push_str(&fill_hole(v, ty.as_ref(), shape.as_deref().unwrap_or(""))?);
    }
    if let Some(k) = used.iter().position(|u| !u) {
        return Err(if numbered { format!("value {} is never used by a hole", k + 1) } else { format!("the format has fewer holes than values ({} value(s) unused)", vals.len() - next) });
    }
    Ok(out)
}

/// One hole's text: the value's readable form, or its shape.
fn fill_hole(v: &Value, ty: Option<&Ty>, shape: &str) -> Result<String, String> {
    // a brain16 shapes as the bin it exactly is; its plain `{}` is its own shortest form (2026-10-02)
    if let Value::Brain(x) = v {
        if shape.is_empty() { return Ok(crate::value::brain_useful(*x)); }
        if shape == "?" { return Ok(v.show_stored()); }
        return fill_hole(&Value::Bin(*x), Some(&Ty::Bin(32)), shape);
    }
    let sc: Vec<char> = shape.chars().collect();
    let mut j = 0;
    let (mut fill, mut align) = (' ', None::<char>);
    if sc.len() >= 2 && matches!(sc[1], '<' | '^' | '>') { fill = sc[0]; align = Some(sc[1]); j = 2; }
    else if sc.first().map_or(false, |c| matches!(c, '<' | '^' | '>')) { align = Some(sc[0]); j = 1; }
    let plus = sc.get(j) == Some(&'+'); if plus { j += 1; }
    if sc.get(j) == Some(&'#') { return Err("`#` is not a shape here: type the prefix in the text, `0x{:x}`".into()); }
    let zero = sc.get(j) == Some(&'0') && align.is_none(); if zero { j += 1; }
    let mut width = String::new(); while let Some(d) = sc.get(j) { if d.is_ascii_digit() { width.push(*d); j += 1; } else { break; } }
    let mut prec: Option<usize> = None;
    if sc.get(j) == Some(&'.') { j += 1; let mut p = String::new(); while let Some(d) = sc.get(j) { if d.is_ascii_digit() { p.push(*d); j += 1; } else { break; } } if p.is_empty() { return Err(format!("`{{:{shape}}}`: a `.` needs a number of places")); } prec = Some(p.parse().unwrap()); }
    let kind: String = sc[j..].iter().collect();
    if !matches!(kind.as_str(), "" | "x" | "X" | "o" | "b" | "e" | "?") { return Err(format!("`{{:{shape}}}`: `{kind}` is not a shape kind (x X o b e ? are)")); }
    let ty = match ty { Some(Ty::Ranged(b, _, _)) => Some(&**b), t => t };
    let numeric = matches!(v, Value::Int(_) | Value::Bin(_) | Value::Deci(_));
    let refuse = |what: &str| Err(format!("{what} does not apply to a {}", v.kind()));
    if (plus || zero) && !numeric { return refuse(if plus { "`+`" } else { "`0` padding" }); }
    let body = match (v, kind.as_str()) {
        (_, "?") => { if prec.is_some() { return refuse("a precision with `?`"); } v.show_stored() }
        (Value::Int(n), k) => {
            if prec.is_some() { return refuse("a precision"); }
            let bits = match ty { Some(Ty::Int(w) | Ty::Uint(w)) => match w { IntWidth::Other(n) => *n as _, IntWidth::W8 => 8, IntWidth::W16 => 16, IntWidth::W32 => 32, IntWidth::W64 => 64, IntWidth::W128 => 128 }, _ => 64 };
            let raw = if *n < 0 && bits < 128 { (*n as u128) & ((1u128 << bits) - 1) } else { *n as u128 };
            match k { "" => n.to_string(), "x" => format!("{raw:x}"), "X" => format!("{raw:X}"), "o" => format!("{raw:o}"), "b" => format!("{raw:b}"), _ => return refuse("`e`") }
        }
        (Value::Bin(x), k) => match (k, prec) {
            ("", None) => show_useful_as(v, ty),
            ("", Some(p)) => format!("{:.*}", p, x),
            ("e", None) => format!("{:e}", x),
            ("e", Some(p)) => format!("{:.*e}", p, x),
            _ => return refuse(&format!("`{k}`")),
        },
        (Value::Deci(d), k) => match (k, prec) {
            ("", None) => d.show_useful(),
            ("", Some(p)) => deci_fixed(d, p),
            ("e", p) => deci_sci(d, p),
            _ => return refuse(&format!("`{k}`")),
        },
        (Value::Str(t), "") => match prec { Some(p) => t.chars().take(p).collect(), None => t.clone() },
        (Value::Char(_, t), "") => { if prec.is_some() { return refuse("a precision"); } t.clone() }
        (_, "") => { if prec.is_some() { return refuse("a precision"); } show_useful_as(v, ty) }
        (_, k) => return refuse(&format!("`{k}`")),
    };
    let body = if plus && !body.starts_with('-') { format!("+{body}") } else { body };
    let w: usize = width.parse().unwrap_or(0);
    let n = body.chars().count();
    if n >= w { return Ok(body); }
    let pad = w - n;
    if zero {
        let (sign, rest) = if body.starts_with('-') || body.starts_with('+') { body.split_at(1) } else { ("", body.as_str()) };
        return Ok(format!("{sign}{}{rest}", "0".repeat(pad)));
    }
    let f = fill.to_string();
    Ok(match align.unwrap_or(if numeric { '>' } else { '<' }) {
        '<' => format!("{body}{}", f.repeat(pad)),
        '>' => format!("{}{body}", f.repeat(pad)),
        _ => { let l = pad / 2; format!("{}{body}{}", f.repeat(l), f.repeat(pad - l)) }
    })
}

/// A deci's digits rounded half to even at `keep` digits from the left; returns the digits and
/// whether rounding carried into a new leading digit.
fn round_digits(digits: &[u8], keep: usize) -> (Vec<u8>, bool) {
    if keep >= digits.len() { let mut d = digits.to_vec(); d.resize(keep, 0); return (d, false); }
    let mut kept = digits[..keep].to_vec();
    let first = digits[keep];
    let rest_nonzero = digits[keep + 1..].iter().any(|d| *d != 0);
    let last_odd = kept.last().map_or(false, |d| d % 2 == 1);
    let up = first > 5 || (first == 5 && (rest_nonzero || last_odd));
    if !up { return (kept, false); }
    let mut k = kept.len();
    loop {
        if k == 0 { kept.insert(0, 1); return (kept, true); }
        k -= 1;
        if kept[k] == 9 { kept[k] = 0; } else { kept[k] += 1; return (kept, false); }
    }
}

/// `{:.p}` of a deci: exactly p places, from its decimal digits, half to even (`deci`'s own rule).
fn deci_fixed(d: &crate::deci::Deci, p: usize) -> String {
    // value = coef × 10^exp; the digits we want are coef × 10^(exp + p), rounded to a whole number
    let shift = d.exp as i64 + p as i64;
    let mut whole: Vec<u8> = if shift >= 0 {
        let mut c = d.coef.clone(); c.extend(std::iter::repeat(0).take(shift as usize)); c
    } else {
        let drop = (-shift) as usize;
        if drop > d.coef.len() {
            let mut padded = vec![0u8; drop - d.coef.len()]; padded.extend(&d.coef);
            let (r, carry) = round_digits(&padded, 0);
            if carry { r } else { vec![0] }
        } else {
            let keep = d.coef.len() - drop;
            let (r, _) = round_digits(&d.coef, keep);
            if r.is_empty() { vec![0] } else { r }
        }
    };
    while whole.len() > 1 && whole[0] == 0 { whole.remove(0); }
    if whole.len() <= p { let mut z = vec![0u8; p + 1 - whole.len()]; z.extend(&whole); whole = z; }
    let text: String = whole.iter().map(|x| (b'0' + x) as char).collect();
    let (a, b) = text.split_at(text.len() - p);
    let body = if p == 0 { a.to_string() } else { format!("{a}.{b}") };
    if d.neg && whole.iter().any(|x| *x != 0) { format!("-{body}") } else { body }
}

/// `{:e}` of a deci: `d.ddde±n`, all its significant digits, or p after the point, half to even.
fn deci_sci(d: &crate::deci::Deci, p: Option<usize>) -> String {
    let mut digits = d.coef.clone();
    while digits.len() > 1 && digits[0] == 0 { digits.remove(0); }
    if digits.iter().all(|x| *x == 0) { return match p { Some(p) if p > 0 => format!("0.{}e0", "0".repeat(p)), _ => "0e0".into() }; }
    let mut e = digits.len() as i64 - 1 + d.exp as i64;
    let keep = match p { Some(p) => p + 1, None => { let mut k = digits.len(); while k > 1 && digits[k - 1] == 0 { k -= 1; } k } };
    let (mut r, carry) = round_digits(&digits, keep);
    if carry { r.pop(); e += 1; }
    let t: String = r.iter().map(|x| (b'0' + x) as char).collect();
    let body = if t.len() > 1 { format!("{}.{}e{e}", &t[..1], &t[1..]) } else { format!("{t}e{e}") };
    if d.neg { format!("-{body}") } else { body }
}

/// A statement's own line: every `Stmt` variant ends with `line`, so it is the last one printed.
fn stmt_line(s: &Stmt) -> usize {
    let t = format!("{s:?}");
    t.rfind("line: ").and_then(|i| t[i + 6..].split(|c: char| !c.is_ascii_digit()).next().and_then(|d| d.parse().ok())).unwrap_or(0)
}

/// Equality by value, for map keys: a deci compares by its value (`1.50` is `1.5`), everything else
/// as held, field by field and item by item.
fn value_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Deci(x), Value::Deci(y)) => x.cmp_value(y).is_eq(),
        (Value::List(x), Value::List(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| value_eq(p, q)),
        (Value::Struct(n, xs), Value::Struct(m, ys)) => n == m && xs.len() == ys.len() && xs.iter().zip(ys).all(|((k, p), (l, q))| k == l && value_eq(p, q)),
        (Value::Enum(t, c, d), Value::Enum(u, e, f)) => t == u && c == e && match (d, f) { (Some(p), Some(q)) => value_eq(p, q), (None, None) => true, _ => false },
        _ => a == b,
    }
}

/// Which Neuron a module path belongs to: `""` for the Neuron being built, `"::geometry"` for a dependency.
fn limb_of(path: &[String]) -> &str {
    match path.first() { Some(p) if p.starts_with("::") => p.as_str(), _ => "" }
}

/// Why an item cannot be reached: across a Neuron's edge only `pub` passes (limbs.md).
fn private_msg(from: &[String], owner: &[String], what: &str) -> String {
    let what = what.replace("::", "").replacen('.', "::", if limb_of(owner).is_empty() { 0 } else { 1 });
    if limb_of(from) != limb_of(owner) {
        format!("{what} is not `pub`, and another Neuron reaches only `pub` items")
    } else {
        format!("{what} is private to `{}`; mark it `pub` or `pub.neuron`", root_or(owner))
    }
}

/// A bare integer literal landing in a `bin` element slot takes the bin family, as it does in a
/// declaration (types.md: a literal takes the family of its context). A deci literal is made
/// deci where it is read (`deci_ctx`); an integer fits an integer slot by its range check.
fn elem_literal(elem: Option<&Ty>, v: Value, prov: &Prov) -> Value {
    let bin = match elem { Some(Ty::Bin(_)) => true, Some(Ty::Ranged(b, _, _)) => matches!(**b, Ty::Bin(_)), _ => false };
    let brain = matches!(elem, Some(Ty::Brain));
    match (v, prov) {
        (Value::Int(i), Prov::Literal) if bin => Value::Bin(i as f64),
        // a literal in a brain16 slot is the nearest brain16 (owner, 2026-10-02)
        (Value::Int(i), Prov::Literal) if brain => Value::Brain(crate::value::round_brain(i as f64)),
        (Value::Bin(x), Prov::Literal) if brain => Value::Brain(crate::value::round_brain(x)),
        (v, _) => v,
    }
}
