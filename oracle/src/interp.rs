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
//!     every division checks for zero — there is no Precog here to remove them
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
    Plain,                               // a plain value: copies freely
    Fresh,                               // a newly made owned value (literal, call result): storable
    Given,                               // `give 'x'`: ownership transferred here
    Copied,                              // `copy 'x'`: an independent duplicate
    Borrowed(Rc<RefCell<Value>>, bool),  // lend / lend.mut of a variable (bool = mutable)
    Bare(String),                        // a bare reference to an owned variable: readable, not storable
}

pub struct Interp<'p> {
    prog: &'p Program,
    /// Writes to borrowed things (check claude, 2026-09-29): each owner's cell → (write count, line, what).
    /// A lent slot made before the latest write is stale, and using it is refused: a borrow never
    /// aliases a mutation and never outlives its owner (core-features.md).
    writes: HashMap<usize, (u64, usize, &'static str)>,
    write_count: u64,
    /// the group cells of the last returned `-> lent@x` borrow, until a declaration takes them
    call_deps: Vec<usize>,
    pending_deps: Vec<usize>,
    /// the line of the `return` that ended the last call
    ret_line: usize,
    /// the matched case's data type, for the name an arm binds
    arm_ty: Option<Ty>,
    /// Limb-level `const`s by canonical key (`module/name`): never assignable, every read a fresh value.
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
    scopes: Vec<HashMap<String, Slot>>,
    out: Box<dyn Write>,
}

impl<'p> Interp<'p> {
    pub fn new(prog: &'p Program, out: Box<dyn Write>) -> Self {
        Interp { prog, writes: HashMap::new(), write_count: 0, call_deps: Vec::new(), pending_deps: Vec::new(), ret_line: 0, arm_ty: None, consts: HashMap::new(), module: Vec::new(), fails: Vec::new(), call_is_stmt: false, expected: None, last_kind: None, last_lit: None, ret_kind: None, ret_borrow: None, call_borrow: None, tasks: Vec::new(), next_chan_id: 0, nocheck_depth: 0, trust_c_depth: 0, tvars: std::cell::RefCell::new(HashMap::new()), field_borrows: Vec::new(), scopes: vec![HashMap::new()], out }
    }

    /// Ok(code): the program ran; Err: it failed.
    pub fn run(&mut self) -> Result<u8, RuntimeError> {
        // Consts first, in order: each may use the ones above it. Precog would
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
                return rt(c.line, format!("a const's value must be known at compile time; `std::{call}` is an effect, not a value"));
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
        let main = match &self.prog.main {
            Some(m) => m,
            None => return rt(0, "this Limb has no MAIN, so it is a library, not a program"),
        };
        match self.block(main) {
            Ok(Flow::Normal) => Ok(0),
            Ok(Flow::Return(_)) => rt(0, "`return` inside MAIN: MAIN is a block, not a function; use std::exit[…] for a code"),
            Ok(Flow::Fail(n, _)) => rt(0, format!("MAIN cannot fail (`{n}`); it exits — handle it, or std::exit[…]")),
            Ok(Flow::Break) | Ok(Flow::Continue) => rt(0, "`break`/`continue` outside a loop"),
            Ok(Flow::Give(_)) => rt(0, "`gives` outside a case"),
            Err(RuntimeError { exit: Some(code), .. }) => Ok(code),
            Err(e) => Err(e),
        }
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
        for f in &self.prog.funcs {
            let k = format!("fn {}", join_path(&f.module, &f.name));
            if let Some(l) = seen.insert(k.clone(), f.line) { return rt(f.line, format!("`{}` is defined twice (first at line {l})", &k[3..])); }
        }
        for d in &self.prog.structs {
            let k = format!("struct {}", d.name);
            if let Some(l) = seen.insert(k.clone(), d.line) { return rt(d.line, format!("struct `{}` is defined twice (first at line {l})", d.name)); }
        }
        for e in &self.prog.enums {
            let k = format!("struct {}", e.name);   // structs and enums share the type namespace
            if let Some(l) = seen.insert(k.clone(), e.line) { return rt(e.line, format!("type `{}` is defined twice (first at line {l})", e.name)); }
        }
        for m in &self.prog.modules {
            let k = format!("module {}", m.path.join("."));
            if let Some(l) = seen.insert(k.clone(), m.line) { return rt(m.line, format!("module `{}` is defined twice (first at line {l}) — a file and a block cannot both be it", m.path.join("."))); }
            // One name, one thing, per Limb (decided 2026-09-21): a module and a type cannot share a name.
            let path = m.path.join(".");
            let clash = self.prog.structs.iter().map(|d| (&d.name, d.line, "struct")).chain(self.prog.enums.iter().map(|e| (&e.name, e.line, "enum"))).chain(self.prog.interfaces.iter().map(|i| (&i.name, i.line, "interface"))).find(|(n, _, _)| **n == path);
            if let Some((_, l, what)) = clash {
                let where_ = if m.line == 0 { format!("the file `{path}.cpls` and a {what} at line {l}") } else { format!("a module block at line {} and a {what} at line {}", m.line, l) };
                return rt(l, format!("`{path}` is both a module and a {what} ({where_}); one name is one thing in a Limb — rename one"));
            }
        }
        Ok(())
    }

    /// design/ffi.md: only C-shaped types cross to C; a struct that crosses is `layout.c`.
    fn c_shaped(&self, ty: &Ty) -> Result<(), String> {
        match ty {
            Ty::Int(_) | Ty::Uint(_) | Ty::Bin(_) | Ty::Bool => Ok(()),
            Ty::Ptr(inner, _) => self.c_shaped(inner),
            Ty::Struct(n) => match self.prog.structs.iter().find(|d| &d.name == n) {
                Some(d) if d.layout_c => Ok(()),
                Some(_) => Err(format!("struct `{n}` is not `layout.c`; its layout is Precog's, not C's")),
                None => match self.prog.enums.iter().find(|e| &e.name == n) {
                    Some(e) if e.c_values.is_some() => Ok(()),
                    Some(_) => Err(format!("enum `{n}` is not `layout.c`; a C enum is `layout.c enum {n} {{ a = 0 }}`")),
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
            other => Err(format!("{} is not C-shaped (numbers, bool, ptr, layout.c structs are)", ty_name(other))),
        }
    }

    fn check_boundary(&self) -> Result<(), RuntimeError> {
        for d in self.prog.structs.iter().filter(|d| d.layout_c) {
            for f in &d.fields {
                // a field may be a range type (owner, 2026-09-27: a): C sees the plain number, and
                // a struct that comes from C is checked at the door, `out_of_range`
                let laid = match &f.ty { Ty::Ranged(b, _, _) => &**b, t => t };
                if let Err(m) = self.c_shaped(laid) { return rt(d.line, format!("`layout.c struct {}`: field `{}` — {m}", d.name, f.name)); }
            }
        }
        // `extern let` (decided 2026-09-21): C-shaped, like an extern parameter
        for x in &self.prog.externs {
            if let Err(m) = self.c_shaped(&x.ty) { return rt(x.line, format!("`extern '{}'`: {m}", x.name)); }
            // a C variable is read by its name, and a read never fails, so it holds C's plain
            // number; convert it with `to.Key[…] otherwise …` (owner, 2026-09-28: b)
            if let Some(n) = self.c_checked(&x.ty) { return rt(x.line, format!("`extern '{}'`: a C variable cannot be `{n}` — reading a variable never fails, so it holds C's plain number: declare it `int32` and convert, `to.{n}['{}'] otherwise …`", x.name, x.name)); }
        }
        for f in self.prog.funcs.iter().filter(|f| f.is_extern) {
            for p in &f.params {
                match &p.ty {
                    None if !p.variadic => return rt(f.line, format!("`extern fn {}`: parameter '{}' needs a type; C has no inference", f.name, p.name)),
                    Some(t) => if let Err(m) = self.c_shaped(t) { return rt(f.line, format!("`extern fn {}`: parameter '{}' — {m}", f.name, p.name)); },
                    None => {}
                }
                if p.kind != ParamKind::Owned { return rt(f.line, format!("`extern fn {}`: C receives values and pointers, not `lent`; a lend is written at the call, against a `ptr` parameter", f.name)); }
            }
            // A ranged return (`-> uint64[0, 1_000_000]`) is a contract checked at the door: the call
            // fails `out_of_range` when C returns outside it (decided 2026-09-22). The base must be C-shaped.
            if let Some(t) = &f.ret {
                let base = match t { Ty::Ranged(b, _, _) => &**b, t => t };
                if let Err(m) = self.c_shaped(base) { return rt(f.line, format!("`extern fn {}`: return type — {m}", f.name)); }
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
                        if who == "MAIN" { return rt(*line, "MAIN cannot fail; it exits (`std::exit[…]`) or handles the failure"); }
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
    /// is reachable from module `from` if it is `pub` / `pub.limb` (one Limb here, so
    /// both count) or `from` is `owner` or a descendant of it. Every module on the
    /// way is checked the same way.
    fn reachable(&self, from: &[String], owner: &[String], vis: Vis) -> bool {
        vis != Vis::Private || from.starts_with(owner)
    }

    fn check_path(&self, from: &[String], module: &[String], line: usize) -> Result<(), RuntimeError> {
        for i in 1..=module.len() {
            let path = &module[..i];
            let Some(m) = self.prog.modules.iter().find(|m| m.path == path) else {
                return rt(line, format!("no module `{}` in this Limb", path.join(".")));
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
            (Some(_), false) => return rt(line, format!("'{name}' is already declared in an enclosing scope; to shadow it deliberately write `$:=` or `$=`")),
            (None, true) => return rt(line, format!("`$` shadows an existing '{name}', but none is visible here")),
            (Some(existing), true) if existing.kind == SlotKind::Const => return rt(line, format!("'{name}' is a const and cannot be shadowed")),
            // `immut` only forbids `change` now; `!$` forbids shadowing (owner, 2026-09-25)
            (Some(existing), true) if existing.sealed => return rt(line, format!("'{name}' cannot be shadowed: it is `!$` (or a loop's own counter)")),
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
            Ok(Flow::Give(_)) => rt(task.line, "`gives` outside a case"),
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
                if kind == SlotKind::ExternMut && self.nocheck_depth == 0 { return rt(*line, format!("'{name}' is extern: writing C's variable is a nocheck site: `nocheck '{name}' = …;`")); }
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
                let items = match &mut *cur { Value::List(items) => items, other => return rt(*line, format!("'{name}' is {}, not a list or array", other.kind())) };
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
                let mut ran = false;
                for (k, item) in items.into_iter().enumerate() {
                    ran = true;
                    *cell.borrow_mut() = item;
                    *pos_cell.borrow_mut() = Value::Int(k as i128 + 1);
                    self.push();
                    let r = match index {
                        Some(ix) => self.declare(ix, Slot { cell: pos_cell.clone(), ty: Some(Ty::Uint(crate::ast::IntWidth::W64)), kind: SlotKind::Owned, alive: true, immut: true, sized: false, sealed: true , since: 0, deps: Vec::new() }, *line),
                        None => Ok(()),
                    }
                    .and_then(|_| self.declare(name, Slot { cell: cell.clone(), ty: elem_ty.clone(), kind, alive: true, immut: true, sized: false, sealed: true , since: 0, deps: Vec::new() }, *line))
                    .and_then(|_| self.stmts(body));
                    self.pop();
                    let r = r?;
                    if let Some((n, key)) = &walked {
                        if let Some((w, at, what)) = self.writes.get(key) {
                            if *w > started { return rt(*line, format!("'{n}' was {what} on line {at} while this loop was going through it: a loop over `lend '{n}'` borrows it until the loop ends")); }
                        }
                    }
                    match r {
                        Flow::Break => break,
                        f @ (Flow::Return(_) | Flow::Fail(..) | Flow::Give(_)) => return Ok(f),
                        Flow::Normal | Flow::Continue => {}
                    }
                }
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
                        let (v, prov) = self.bracket(b, *line)?;
                        self.ret_kind = self.last_kind.take();
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
                if arm.value.is_some() { return rt(arm.line, format!("case `{}` gives a value, but this `match` is not used as one", arm.name)); }
                match self.run_arm(arm, data)? {
                    Flow::Give(_) => rt(arm.line, format!("case `{}` gives a value, but this `match` is not used as one", arm.name)),
                    f => Ok(f),
                }
            }
            Stmt::Gives { value, line } => {
                let (v, _) = self.bracket(value, *line)?;
                Ok(Flow::Give(v))
            }
            Stmt::Asm { text, binds, line } => {
                if self.nocheck_depth == 0 { return rt(*line, "an `asm` block is a nocheck site: the compiler cannot see inside it"); }
                for b in binds {
                    match b {
                        AsmBind::In(v, _) | AsmBind::InOut(v, _) => { if self.lookup(v).is_none() { return rt(*line, format!("'{v}' is not declared")); } }
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
            // The tiers (decided 2026-09-19). The oracle has no Precog, so it cannot
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
                            Tier::Check => format!("`check` at line {line} fails: {} — the guardrail is not enough (Precog would refuse to compile; this value is the counterexample)", e.msg),
                            Tier::NoCheck => format!("`nocheck` at line {line} lied: {} — the compiled program would be undefined here", e.msg),
                            Tier::NoChecksAtAll => format!("`NO_FUCKING_CHECKS` at line {line} lied: {} — the compiled program would be undefined here", e.msg),
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
            Prov::Borrowed(_, true) => SlotKind::LentMut, // elements are copies; element write-back is not modelled
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
        let written = v.clone();
        // a bin literal landing in a bin16/bin32 is rounded once, from its digits
        let v = match (&v, &prov, &lit, bin_w(t)) {
            (Value::Bin(_), Prov::Literal, Some(d), Some(w @ (16 | 32))) => Value::Bin(round_decimal(d, w)),
            _ => v,
        };
        let shown = |a: f64| lit.clone().filter(|_| matches!(prov, Prov::Literal)).unwrap_or_else(|| format!("{a}"));
        if matches!(prov, Prov::Given | Prov::Copied | Prov::Fresh) { self.kind_clash(have, t, line)?; }
        let v = match (t, &v, &prov) {
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
            (Ty::Int(_) | Ty::Uint(_), Value::Int(i)) => {
                let (lo, hi) = int_range(ty).unwrap();
                if *i < lo || *i > hi { return site(line, format!("{i} does not fit {name} (range {lo} to {hi})")); }
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
            (Ty::Deci(_), _) => rt(line, "deci is not implemented in the oracle"),
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
                let saved = std::mem::replace(&mut self.expected, elem);
                let mut items = Vec::new();
                for v in &b.values {
                    let r = self.value(v);
                    let (val, prov) = match r { Ok(x) => x, Err(e) => { self.expected = saved; return Err(e); } };
                    match prov {
                        Prov::Bare(name) => { self.expected = saved; return rt(line, format!("bare '{name}' inside a list: say `give`, `copy`, or `lend`")); }
                        Prov::Borrowed(..) => { self.expected = saved; return rt(line, "a lend inside a list: a list owns its elements (`copy` or `give`)"); }
                        _ => {}
                    }
                    items.push(val);
                }
                self.expected = saved;
                Ok((Value::List(items), Prov::Fresh))
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
        let t = match e {
            Expr::Var(name, _) => self.lookup(name).and_then(|s| s.ty.clone()),
            Expr::Paren(inner, _) => self.expr_ty(inner),
            Expr::Convert(t, _, _, _) => Some(t.clone()),
            Expr::Unary(_, inner, _) => self.expr_ty(inner),
            Expr::Binary(_, l, r, _) => self.expr_ty(l).or_else(|| self.expr_ty(r)),
            _ => None,
        };
        let t = t.or_else(|| self.expected.clone());
        match t { Some(Ty::Ranged(b, _, _)) => Some(*b), t => t }
    }

    /// `return lend 'x';` is checked where it is written, called or not (check claude, 2026-09-29):
    /// a borrow of an owned local or an owned parameter dies at the return, one outside the
    /// signature's group is refused, and one returned without `-> lent` must say so. A borrow
    /// that comes from a call or another lend is followed when the function runs.
    fn check_returned_borrows(&self) -> Result<(), RuntimeError> {
        fn walk<'a>(b: &'a [Stmt], out: &mut Vec<(&'a str, bool, usize)>, owned_locals: &mut Vec<&'a str>) {
            for s in b {
                match s {
                    Stmt::Return { value: Some(v), line } => if let [crate::ast::Value::Expr(Expr::Owned(verb @ (Verb::Lend | Verb::LendMut), n, _))] = v.values.as_slice() { out.push((n.as_str(), *verb == Verb::LendMut, *line)); },
                    Stmt::Decl { name, value, .. } => {
                        let borrowish = matches!(value.values.as_slice(), [crate::ast::Value::Expr(Expr::Owned(Verb::Lend | Verb::LendMut, ..) | Expr::Call(_) | Expr::CallValue(..) | Expr::Var(..))]);
                        if !borrowish { owned_locals.push(name.as_str()); }
                    }
                    Stmt::If { arms, otherwise, .. } => { for (_, b) in arms { walk(b, out, owned_locals); } if let Some(b) = otherwise { walk(b, out, owned_locals); } }
                    Stmt::Loop { body, .. } | Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::Range { body, .. } | Stmt::Block { body, .. } => walk(body, out, owned_locals),
                    Stmt::Match { arms, .. } => for a in arms { walk(&a.body, out, owned_locals); },
                    Stmt::Tier { body, .. } => walk(std::slice::from_ref(body), out, owned_locals),
                    _ => {}
                }
            }
        }
        for f in &self.prog.funcs {
            let (mut rets, mut locals) = (Vec::new(), Vec::new());
            walk(&f.body, &mut rets, &mut locals);
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
            for (i, (_, _, b)) in ev.iter().enumerate() {
                let Some((r, froms, via_call)) = b else { continue };
                let whose = if *via_call { "something its result depends on" } else { "what it borrows" };
                for s in froms {
                    let mut wrote: Option<(usize, &str)> = None;
                    for (line, t, nb) in &ev[i + 1..] {
                        if matches!(nb, Some((n, _, _)) if n == r) { break; }   // a new 'r': the old borrow is over
                        if let Some((at, what)) = wrote { if uses(t, r) {
                            return rt(*line, format!("'{r}' is a borrow, and {whose} was {what} on line {at} while '{r}' was still in use: a borrow never aliases a change and never outlives its owner"));
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
            self.writes.insert(key, (n, line, what));
        }
    }

    /// A borrow used after what it borrows was changed or given away (check claude's finding, 2026-09-29).
    fn stale(&self, name: &str, s: &Slot, line: usize) -> Result<(), RuntimeError> {
        if !matches!(s.kind, SlotKind::Lent | SlotKind::LentMut) { return Ok(()); }
        let own = Rc::as_ptr(&s.cell) as *const () as usize;
        for key in std::iter::once(&own).chain(s.deps.iter()) {
            if let Some((n, at, what)) = self.writes.get(key) {
                if *n > s.since {
                    let whose = if *key == own { "what it borrows" } else { "something its result depends on" };
                    return rt(line, format!("'{name}' is a borrow, and {whose} was {what} on line {at} while '{name}' was still in use: a borrow never aliases a change and never outlives its owner"));
                }
            }
        }
        Ok(())
    }

    fn read_var(&self, name: &str, line: usize) -> Result<Value, RuntimeError> {
        match self.lookup(name) {
            None => rt(line, format!("'{name}' is not declared")),
            Some(s) if matches!(s.kind, SlotKind::Extern | SlotKind::ExternMut) && self.nocheck_depth == 0 =>
                rt(line, format!("'{name}' is extern: C's variable, which nothing can be proven about — reading it is a nocheck site: `nocheck … '{name}' …`")),
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
        let simd_shape = matches!(e, Expr::Builtin(h, op, ..) if h == "simd" && (op == "load" || op == "fill" || op == "load.some"));
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
    /// `None` when it is neither, or when nothing fixes it (a literal, a `:=` Precog picks).
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
            Expr::Binary(op, l, r, _) if matches!(op.as_str(), "+" | "-" | "x" | "xx" | "div" | "mod") => {
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
    fn as_index(&self, name: &str, args: &Bracket, has_handler: bool, line: usize) -> Result<Option<Expr>, RuntimeError> {
        let holds_list = self.lookup(name).map(|s| matches!(&*s.cell.borrow(), Value::List(_) | Value::Str(_))).unwrap_or(false);
        if !holds_list { return Ok(None); }
        if self.prog.strict_modules.iter().any(|m| *m == self.module) { return rt(line, format!("`#strict`: a list read says it is a position: `'{name}'[pos 2]`")); }
        if has_handler { return rt(line, format!("reading '{name}'[…] cannot fail — a position past the end is a check site — so it takes no `otherwise`")); }
        match args.values.as_slice() {
            [crate::ast::Value::Expr(e)] => Ok(Some(e.clone())),
            _ => rt(line, format!("an index is one position: `'{name}'[3]`")),
        }
    }

    /// A type C's numbers are checked against when they come in: a `layout.c enum`, a range,
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
    fn run_arm(&mut self, arm: &Arm, data: Option<Value>) -> Result<Flow, RuntimeError> {
        self.push();
        if let (Some(b), Some(d)) = (&arm.bind, data) {
            let ty = self.arm_ty.take();
            let r = self.declare(b, Slot { cell: Rc::new(RefCell::new(d)), ty, kind: SlotKind::Owned, alive: true, immut: false, sized: false, sealed: false , since: 0, deps: Vec::new() }, arm.line);
            if let Err(e) = r { self.pop(); return Err(e); }
        }
        let r = match &arm.value {
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
            Expr::Int(i, _) => Ok((Value::Int(*i), Prov::Literal)),
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
                // (provisional — Precog would also look at use).
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
                    Value::List(items) => items,
                    Value::Str(_) => return rt(line, format!("'{name}' is a text: `[pos …]` would not say which unit — the 3rd byte or the 3rd grapheme; use `std::str.at.grapheme[lend '{name}', pos …]` (decided 2026-09-20)")),
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
                    return rt(line, format!("struct `{sname}` is private to `{}`; mark it `pub` or `pub.limb`", root_or(&def.module)));
                }
                // a generic struct's blanks are filled by the values given (Precog: from use)
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
                    return rt(line, format!("const '{name}' is private to `{}`; mark it `pub` or `pub.limb`", root_or(module)));
                }
                let v = self.consts[&const_key(module, name)].cell.borrow().clone();
                let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                Ok((v, prov))
            }
            // `to.array[give 'l']` / `to.list[give 'a']` (owner, 2026-09-23): the elements are
            // handed over as they are; only whether it can grow changes
            Expr::Convert(Ty::List(_) | Ty::Array(_), b, _, _) => {
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
            Expr::Convert(Ty::Struct(tname), b, handler, _) => {
                let def = match self.prog.enums.iter().find(|e| &e.name == tname) {
                    Some(e) if e.c_values.is_some() => e.clone(),
                    Some(_) => return rt(line, format!("no conversion to the enum `{tname}`; only a `layout.c enum`'s numbers convert")),
                    None => return rt(line, format!("no conversion to {tname}; conversions are between numbers, text, lists and arrays, and into a `layout.c enum`")),
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
            Expr::Convert(ty, b, handler, _) => {
                let (v, _) = self.single(b, line)?;
                let target = ty_name(ty);
                let full_ty = ty;
                let text_base;
                let (base, ranged): (&Ty, bool) = match ty {
                    Ty::Ranged(b, _, _) => (b, true),
                    Ty::TextRanged(e, _, _, _) => { text_base = Ty::Str(e.clone()); (&text_base, true) }   // a bounded text: out of it is `out_of_range` (2026-09-22)
                    t => (t, false),
                };
                let can_fail = matches!(v, Value::Str(_)) && matches!(base, Ty::Int(_) | Ty::Uint(_) | Ty::Bin(_));
                // text → number fails `not_a_number`, and a real number too big for the type fails
                // `out_of_range`: text comes from outside, so it is a failure, not a check (owner, 2026-09-24: c)
                let mut fails = if can_fail { vec![FailCase { name: "not_a_number".into(), ty: None }, FailCase { name: "out_of_range".into(), ty: None }] } else { Vec::new() };
                let from_text = can_fail;
                // bin → int fails unless the bin is whole (decided 2026-09-21): rounding is said first, in std
                if matches!(v, Value::Bin(_)) && matches!(base, Ty::Int(_) | Ty::Uint(_)) { fails.push(FailCase { name: "not_whole".into(), ty: None }); }
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
                    (Ty::Str(_), other) => Value::Str(other.show_stored()),
                    (Ty::Int(_) | Ty::Uint(_), Value::Int(i)) => { fit_base!(Value::Int(*i)); Value::Int(*i) }
                    (Ty::Bin(_), Value::Int(i)) => { let b = Value::Bin(*i as f64); fit_base!(b.clone()); b }
                    (Ty::Bin(_), Value::Bin(x)) => { fit_base!(Value::Bin(*x)); Value::Bin(*x) }
                    (Ty::Int(_) | Ty::Uint(_), Value::Bin(x)) => {
                        if !x.is_finite() || x.fract() != 0.0 {
                            let v = self.handle_failure(handler.as_ref().unwrap(), line, &format!("to.{target}"), "not_whole".into(), None, true)?;
                            let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                            return Ok((v, prov));
                        }
                        let i = Value::Int(*x as i128);
                        fit_base!(i.clone());
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
                        let Some(e) = self.prog.enums.iter().find(|e| &e.name == tn && e.c_values.is_some()) else { return rt(line, format!("only a `layout.c enum` has numbers; `{tn}` is a Cyborg enum")) };
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
                    let inside = match (&out, lo, hi) { (Value::Int(i), Bound::Int(a), Bound::Int(b)) => a <= i && i <= b, (Value::Bin(x), Bound::Bin(a), Bound::Bin(b)) => a <= x && x <= b, _ => true };
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
                match self.run_arm(arm, data)? {
                    Flow::Give(v) => { let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain }; Ok((v, prov)) }
                    Flow::Normal => rt(arm.line, format!("case `{}` neither gives a value nor leaves (return, fail, break, continue)", arm.name)),
                    f => Err(RuntimeError { line, msg: String::new(), exit: None, site: false, leave: Some(f) }),
                }
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
            Expr::Builtin(head, op, args, _) => {
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
                    let width = self.expr_ty(inner);
                    let (bits, signed) = match width { Some(Ty::Int(w)) => (int_bits(&w), true), Some(Ty::Uint(w)) => (int_bits(&w), false), _ => return rt(line, "`not.bit` flips a type's bits, so it needs to know how many: give the value a type, or say it in the declaration (`let 'x': uint32 = not.bit 'v';`)") };
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
                    ("-", Value::Int(i)) => Ok((Value::Int(i.checked_neg().ok_or_else(|| RuntimeError { line, msg: "negation overflow".into(), exit: None, site: true, leave: None })?), keep)),
                    ("-", Value::Bin(b)) => Ok((Value::Bin(-b), keep)),
                    ("-", o) => rt(line, format!("`-` needs a number, got {}", o.kind())),
                    _ => unreachable!(),
                }
            }
            // Wrapping arithmetic (decided 2026-09-20): `+~ -~ x~ xx~` are always defined —
            // modulo the type's size. The oracle wraps at the left operand's declared
            // integer width (Precog knows every width; the oracle only knows declared ones).
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
                let (bits, signed) = match width {
                    Some(Ty::Int(w)) => (int_bits(&w), true),
                    Some(Ty::Uint(w)) => (int_bits(&w), false),
                    // the width can come from a name, a conversion, either side, or the type
                    // stored into (2026-09-23); a bare literal with none of those has no size to wrap in
                    Some(t) => return rt(line, format!("`{op}` wraps modulo an integer's size, and {} is not an integer", ty_name(&t))),
                    None => return rt(line, format!("`{op}` wraps modulo a type's size, so it needs to know which: give a side a type, or say it in the declaration (`let 'x': uint64 = 'a' {op} 'b';`)")),
                };
                let (a, _) = self.eval(l)?;
                let (b, _) = self.eval(r)?;
                let (x, y) = match (a, b) { (Value::Int(x), Value::Int(y)) => (x, y), (a, b) => return rt(line, format!("`{op}` is integer arithmetic, got {} and {}", a.kind(), b.kind())) };
                let raw = match op.as_str() {
                    "+~" => x.wrapping_add(y),
                    "-~" => x.wrapping_sub(y),
                    "x~" => x.wrapping_mul(y),
                    _ => { if y < 0 { return site(line, "a negative exponent on integers would produce a fraction — a check site: Precog proves the exponent is never negative, or the final build refuses"); } let mut acc: i128 = 1; for _ in 0..y { acc = acc.wrapping_mul(x); } acc }
                };
                let m: i128 = 1i128 << bits;
                let mut v = raw.rem_euclid(m);
                if signed && v >= m / 2 { v -= m; }
                Ok((Value::Int(v), Prov::Plain))
            }
            // Bitwise (decided 2026-09-21): integers only; `shl` past the width is a check site.
            Expr::Binary(op, l, r, _) if matches!(op.as_str(), "and.bit" | "or.bit" | "xor" | "shl" | "shr") => {
                let (a, _) = self.eval(l)?;
                let (b, _) = self.eval(r)?;
                // a vector is N of the same operation, lane by lane (design/simd.md)
                if matches!(a, Value::Simd(_)) || matches!(b, Value::Simd(_)) {
                    let v = self.simd_bitwise(op, a, b, line)?;
                    return Ok((v, Prov::Plain));
                }
                let (x, y) = match (a, b) { (Value::Int(x), Value::Int(y)) => (x, y), (a, b) => return rt(line, format!("`{op}` works on integers, got {} and {}", a.kind(), b.kind())) };
                let v = match op.as_str() {
                    "and.bit" => x & y,
                    "or.bit" => x | y,
                    "xor" => x ^ y,
                    "shl" | "shr" => {
                        if y < 0 { return rt(line, format!("`{op}` by a negative count")); }
                        let width_ty = self.expr_ty(l);
                        let bits = match &width_ty { Some(Ty::Int(w)) | Some(Ty::Uint(w)) => int_bits(w), _ => 127 };
                        // compare before narrowing: `y as u32` would wrap a huge count to a small one (seed 1125)
                        if y >= bits as i128 {
                            let what = if width_ty.is_some() { format!("a {bits}-bit value") } else { "a value of any width".to_string() };
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
            Expr::Binary(op, l, r, _) if !matches!(op.as_str(), "+" | "-" | "x" | "xx" | "div" | "mod" | "++" | "==" | "!==" | "<" | ">" | "</==" | ">/==" | "and" | "or") => {
                self.custom_op(op, l, r, line)
            }
            Expr::Binary(op, l, r, _) => {
                // A comparison's operands take their width from each other (2026-09-23): the
                // stored `bool` says nothing, but a typed side sizes a literal group on the other
                // (`'n' > (0 -~ 1)` wraps in uint64; verification session, seed 90).
                let cmp = matches!(op.as_str(), "==" | "!==" | "<" | ">" | "</==" | ">/==");
                let saved_expected = self.expected.clone();
                if cmp { self.expected = None; let other = self.expr_ty(r); self.expected = other; }
                let r_a = self.eval(l);
                let la = self.last_lit.take();
                if cmp { self.expected = saved_expected.clone(); }
                let (a, pa) = r_a?;
                // the language's operator, defined for a user type (2026-09-20)
                if matches!(a, Value::Struct(..) | Value::Enum(..)) {
                    let shape = shape_of(&a);
                    if self.prog.funcs.iter().any(|f| f.is_op && f.name == *op && f.params[0].ty.as_ref().map(|t| ty_shape(t) == shape).unwrap_or(false)) {
                        return self.custom_op_with(op, (a, pa), l, r, line);
                    }
                }
                if cmp { self.expected = None; let other = self.expr_ty(l); self.expected = other; }
                let r_b = self.eval(r);
                let lb = self.last_lit.take();
                if cmp { self.expected = saved_expected; }
                let (b, pb) = r_b?;
                // a bin literal meeting a bin16/bin32 value in arithmetic is at that width, rounded
                // once from its digits (owner, 2026-09-24: bin math at its values' width)
                let (a, b) = if cmp || op == "++" { (a, b) } else {
                    let a = match (&a, &pa, &la, self.bin_width(r)) { (Value::Bin(_), Prov::Literal, Some(d), Some(w @ (16 | 32))) => Value::Bin(round_decimal(d, w)), _ => a };
                    let b = match (&b, &pb, &lb, self.bin_width(l)) { (Value::Bin(_), Prov::Literal, Some(d), Some(w @ (16 | 32))) => Value::Bin(round_decimal(d, w)), _ => b };
                    (a, b)
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
                let v = self.binary(op, a, b, line)?;
                // bin16/bin32 math rounds after every operation, at its values' width, as the
                // hardware does (owner, 2026-09-24: a — "the programmer could've chose a better one")
                if let Value::Bin(x) = v {
                    let w = match (self.bin_width(l), self.bin_width(r)) {
                        (Some(a), Some(b)) => Some(a.max(b)),
                        (a, b) => a.or(b).or_else(|| match &self.expected { Some(Ty::Bin(w)) => Some(*w), Some(Ty::Ranged(b, _, _)) => match **b { Ty::Bin(w) => Some(w), _ => None }, _ => None }),
                    };
                    if let Some(w @ (16 | 32)) = w {
                        let r = round_to_width(&Ty::Bin(w), Value::Bin(x));
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
                if matches!(op.as_str(), "+" | "-" | "x" | "xx") && !both_lit {
                    if let Value::Int(n) = v {
                        if n < i64::MIN as i128 || n > u64::MAX as i128 {
                            return site(line, format!("{n} goes past 64 bits in between — `{op}` is exact, but no 64-bit type holds this"));
                        }
                    }
                }
                Ok((v, if both_lit { Prov::Literal } else { Prov::Plain }))
            }
        }
    }

    fn binary(&self, op: &str, a: Value, b: Value, line: usize) -> Result<Value, RuntimeError> {
        // A vector is N of what the operator already does (design/simd.md, 2026-09-22):
        // lane by lane, the scalar rule. A comparison gives a simd of bools, never one bool.
        match (&a, &b) {
            (Value::Simd(x), Value::Simd(y)) => {
                if x.len() != y.len() { return rt(line, format!("`{op}` between a simd of {} lanes and one of {}", x.len(), y.len())); }
                let mut out = Vec::with_capacity(x.len());
                for (l, r) in x.iter().zip(y) { out.push(self.binary(op, l.clone(), r.clone(), line)?); }
                return Ok(Value::Simd(out));
            }
            (Value::Simd(x), scalar) if matches!(scalar, Value::Int(_) | Value::Bin(_) | Value::Bool(_)) => {
                let mut out = Vec::with_capacity(x.len());
                for l in x { out.push(self.binary(op, l.clone(), scalar.clone(), line)?); }
                return Ok(Value::Simd(out));
            }
            (scalar, Value::Simd(y)) if matches!(scalar, Value::Int(_) | Value::Bin(_) | Value::Bool(_)) => {
                let mut out = Vec::with_capacity(y.len());
                for r in y { out.push(self.binary(op, scalar.clone(), r.clone(), line)?); }
                return Ok(Value::Simd(out));
            }
            _ => {}
        }
        use Value::*;
        let ovf = |what: &str| RuntimeError { line, msg: format!("integer overflow in `{what}`"), exit: None, site: true, leave: None };
        Ok(match (op, a, b) {
            ("+", Int(x), Int(y)) => Int(x.checked_add(y).ok_or_else(|| ovf("+"))?),
            ("-", Int(x), Int(y)) => Int(x.checked_sub(y).ok_or_else(|| ovf("-"))?),
            ("x", Int(x), Int(y)) => Int(x.checked_mul(y).ok_or_else(|| ovf("x"))?),
            ("div", Int(_), Int(0)) => return site(line, "division by zero"),
            // Decided 2026-09-20: floor division and the mathematician's mod
            // (-7 / 2 = -4, -7 mod 2 = 1); hardware truncation is std::div.trunc / std::mod.trunc.
            ("div", Int(x), Int(y)) => { let q = x.checked_div(y).ok_or_else(|| ovf("div"))?; Int(if x % y != 0 && ((x < 0) != (y < 0)) { q - 1 } else { q }) }
            ("mod", Int(_), Int(0)) => return site(line, "`mod` by zero"),
            ("mod", Int(x), Int(y)) => { let r = x.checked_rem(y).ok_or_else(|| ovf("mod"))?; Int(if r != 0 && ((r < 0) != (y < 0)) { r + y } else { r }) }
            ("xx", Int(x), Int(y)) => {
                if y < 0 { return site(line, "a negative exponent on integers would produce a fraction — a check site: Precog proves the exponent is never negative, or the final build refuses"); }
                let mut acc: i128 = 1;
                for _ in 0..y { acc = acc.checked_mul(x).ok_or_else(|| ovf("xx"))?; }
                Int(acc)
            }
            ("+", Bin(x), Bin(y)) => Bin(x + y),
            ("-", Bin(x), Bin(y)) => Bin(x - y),
            ("x", Bin(x), Bin(y)) => Bin(x * y),
            ("div", Bin(x), Bin(y)) => Bin(x / y),
            ("mod", Bin(x), Bin(y)) => Bin(x % y),
            ("xx", Bin(x), Bin(y)) => Bin(x.powf(y)),
            ("+" | "-" | "x" | "div" | "mod" | "xx", Int(_), Bin(_)) | ("+" | "-" | "x" | "div" | "mod" | "xx", Bin(_), Int(_)) =>
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
            ("<" | ">" | "</==" | ">/==", Str(_), Str(_)) => return rt(line, format!("`{op}` on text has no meaning yet: byte order is never what anyone means; a collation rule is std's (decided 2026-09-20)")),
            ("and", Bool(x), Bool(y)) => Bool(x && y),
            ("or", Bool(x), Bool(y)) => Bool(x || y),
            (op, x, y) => return rt(line, format!("`{op}` cannot combine {} and {}", x.kind(), y.kind())),
        })
    }

    /// The static type of a single-expression bracket, for the few expressions the
    /// oracle knows one for: `list.length[…]` / `array.length[…]` are `uint64`
    /// (decided 2026-09-21). Everything else: unknown (a provisional gap, see README).
    /// Is the length of what this bracket produces known at this point? A literal list,
    /// `list.empty`, or a value of a bounded type `list[T][lo, hi]` — the cases Precog
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
            [crate::ast::Value::Expr(Expr::Convert(t @ (Ty::List(_) | Ty::Array(_)), b, _, _))] => {
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
        if op == "slice" {
            // A slice owns its elements, as a list always does (owner, 2026-09-23): the copy is
            // the meaning, and Precog elides it where nobody can tell. 1-based and inclusive,
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
                let (arg, aprov) = self.value(&args.values[1])?;
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
                else if let Some(s) = self.lookup_mut(&var_name) { s.sized = false; }   // Precog would track it; the oracle does not
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
    // every comparison it makes; Precog proves it).

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
        if !self.custom_key(a) { return Ok(a == b); }
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
            if ha != hb { return site(line, format!("`op ==` says two {shape} keys are equal but `hash` gives {} and {}; a map cannot trust this key type (prove or `nocheck`)", ha.show_useful(), hb.show_useful())); }
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
        let mut rest = Vec::new();
        for a in &args.values[1..] {
            let (v, prov) = self.value(a)?;
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
                        return rt(line, format!("'{n}' is a `map.unordered`, which promises never to be walked; `map.{op}` needs a `map.ordered` (or a plain `map`, and Precog keeps the order)"));
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
                let (v, _) = self.value(&args.values[0])?;
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
                Ok(acc)
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
        if c.borrow().timer.is_some() { return rt(line, "a timer channel (`std::time.after`/`every`) only receives from time; nothing can send into it"); }
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

    fn call(&mut self, c: &Call) -> Result<Value, RuntimeError> {
        match &c.limb {
            Some(l) if l == "std" => self.std_call(c),
            Some(l) => rt(c.line, format!("no Limb named `{l}` is loaded (the oracle only knows `std`)")),
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
            return rt(c.line, format!("`{}` is private to `{}`; mark it `pub` or `pub.limb`", join_path(&func.module, name), root_or(&func.module)));
        }
        if func.is_op { return rt(c.line, format!("`{name}` is an operator; it goes between its operands (`'a' {name} 'b'`), never called with `[…]`")); }
        if func.is_extern {
            if self.nocheck_depth == 0 { return rt(c.line, format!("`{name}` is extern; nothing about it can be proven — the call is a nocheck site: `nocheck … {name}[…]`")); }
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
            let (v, prov) = match arg { Arg::Ast(a) => self.value(a)?, Arg::Val(v, p) => (v, p) };
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
                if let Err(m) = self.satisfies(&shape, iface, c.line)? { return rt(c.line, format!("`{name}` needs `<{var}: {iface}>`, but {m}")); }
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
            Flow::Give(_) => return rt(func.line, "`gives` outside a case"),
            Flow::Fail(reason, payload) => return self.handle_failure(c.handler.as_ref().unwrap(), c.line, name, reason, payload, value_used),
        };
        // `-> lent@x T`: the borrow must be of a parameter in group x — or of any lent parameter
        // when no group is written (Precog works it out) — never of something that dies here
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
                // without a group, whatever the body may hand back, on any path (Precog reads the body)
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
            (Some(t), v) => { let k = self.ret_kind.take(); self.kind_clash(k, t, c.line)?; self.check_type(v, t, c.line)?; Ok(result) }
            (None, _) => { self.ret_kind = None; Ok(result) }
        }
    }

    /// Does a type (by shape) keep an interface's promises? Built-in operators
    /// count for the language's types; a user type needs an `op`/`fn` for each entry.
    fn satisfies(&self, shape: &str, iface: &str, line: usize) -> Result<Result<(), String>, RuntimeError> {
        let def = match self.prog.interfaces.iter().find(|i| i.name == iface) { Some(i) => i, None => return rt(line, format!("no interface named `{iface}`")) };
        let builtin_ops: &[&str] = match shape {
            "integer" | "bin" => &["+", "-", "x", "xx", "div", "mod", "<", ">", "</==", ">/==", "==", "!=="],
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
        let synthetic = Call { handler: handler.cloned(), limb: None, path: vec![func.name.clone()], from: self.module.clone(), args: args.clone(), line };
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
            Flow::Give(_) => rt(line, "`gives` outside a case"),
        }
    }

    /// Run a function's body with its parameters bound. A function's scopes are its
    /// own: it cannot see the caller's variables.
    fn run_body(&mut self, func: &Func, bound: Vec<(String, Slot)>) -> Result<Flow, RuntimeError> {
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
        let left = self.eval(l)?;
        self.custom_op_with(op, left, l, r, line)
    }

    /// As `custom_op`, with the left operand already evaluated (it is evaluated once).
    fn custom_op_with(&mut self, op: &str, left: (Value, Prov), l: &Expr, r: &Expr, line: usize) -> Result<(Value, Prov), RuntimeError> {
        // pick the definition whose first operand's type matches the left value (an
        // operator may be defined for several types)
        let lshape = shape_of(&left.0);
        let candidates: Vec<Func> = self.prog.funcs.iter().filter(|f| f.is_op && f.name == op).cloned().collect();
        let def = match candidates.iter().find(|f| f.params[0].ty.as_ref().map(|t| ty_shape(t) == lshape || matches!(t, Ty::Var(_))).unwrap_or(true)).or(candidates.first()) {
            Some(f) => f.clone(),
            None => {
                let hint = match op {
                    "*" => "; multiply is `x`, power is `xx`", "^" => "; power is `xx`", "%" => "; remainder is `mod`",
                    "&" | "|" | "!" => "; use `and`, `or`, `not`", _ => "",
                };
                return rt(line, format!("`{op}` is not an operator here{hint} — declare it with `op {op} ['a': T, 'b': T] -> R {{ … }}`"));
            }
        };
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
            Flow::Give(_) => return rt(def.line, "`gives` outside a case"),
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
                let (v, prov) = self.bracket(b, line)?;
                match prov {
                    Prov::Bare(n) => rt(line, format!("the fallback is bare '{n}', which owns a {}: say `give '{n}'` or `copy '{n}'`", v.kind())),
                    _ => Ok(v),
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
                match self.run_arm(arm, payload)? {
                    // a case that gives a value is the value (owner, 2026-09-29)
                    Flow::Give(v) if value_used => Ok(v),
                    Flow::Give(_) => rt(arm.line, format!("the `{reason}` case gives a value, but the value of `{name}` is not used")),
                    Flow::Normal if value_used => rt(arm.line, format!("the `{reason}` arm must leave (return, fail, break, continue, or std::exit) because the value of `{name}` is used")),
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

    /// C's sizes and alignments; a normal struct gets the same rule (Precog's real
    /// choice is not modelled), lists and strs are their 24-byte handle.
    fn size_align(&self, ty: &Ty, line: usize) -> Result<(usize, usize), RuntimeError> {
        Ok(match ty {
            Ty::Int(w) | Ty::Uint(w) => { let n = match w { IntWidth::W8 => 1, IntWidth::W16 => 2, IntWidth::W32 => 4, IntWidth::W64 => 8, IntWidth::W128 => 16 }; (n, n) }
            Ty::Bin(b) | Ty::Deci(b) => { let n = (*b as usize) / 8; (n, n) }
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
        if self.nocheck_depth == 0 { return rt(line, format!("`ptr.{op}` goes through a pointer the compiler cannot vouch for — it is a nocheck site")); }
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
        if !self.reachable(&from, &def.module, def.vis) { return rt(line, format!("enum `{tname}` is private to `{}`; mark it `pub` or `pub.limb`", root_or(&def.module))); }
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
            self.check_handler(c.handler.as_ref(), &fails, &format!("std::{path}"), c.line)?;
            let mut vals = Vec::new();
            for a in &c.args.values {
                let (v, prov) = self.value(a)?;
                if let Prov::Bare(n) = prov { return rt(c.line, format!("std::{path} looks at '{n}': `lend '{n}'`")); }
                vals.push(v);
            }
            let text = |i: usize| -> Result<String, RuntimeError> { match vals.get(i) { Some(Value::Str(s)) => Ok(s.clone()), Some(Value::Char(_, ch)) => Ok(ch.clone()), Some(o) => rt(c.line, format!("std::{path}: argument {} must be text, got {}", i + 1, o.kind())), None => rt(c.line, format!("std::{path}: missing argument {}", i + 1)) } };
            let int = |i: usize| -> Result<i128, RuntimeError> { match vals.get(i) { Some(Value::Int(n)) => Ok(*n), Some(o) => rt(c.line, format!("std::{path}: argument {} must be an integer, got {}", i + 1, o.kind())), None => rt(c.line, format!("std::{path}: missing argument {}", i + 1)) } };
            let fail = |me: &mut Self, reason: &str| -> Result<Value, RuntimeError> { me.handle_failure(c.handler.as_ref().unwrap(), c.line, &format!("std::{path}"), reason.into(), None, value_used) };
            return match path.as_str() {
                "str.split" => { let (s, sep) = (text(0)?, text(1)?); if sep.is_empty() { return rt(c.line, "std::str.split: an empty separator"); } Ok(Value::List(s.split(sep.as_str()).map(|p| Value::Str(p.to_string())).collect())) }
                "str.contains" => Ok(Value::Bool(text(0)?.contains(text(1)?.as_str()))),
                "str.starts_with" => Ok(Value::Bool(text(0)?.starts_with(text(1)?.as_str()))),
                "str.ends_with" => Ok(Value::Bool(text(0)?.ends_with(text(1)?.as_str()))),
                "str.trim" => Ok(Value::Str(text(0)?.trim().to_string())),
                "str.upper" => Ok(Value::Str(text(0)?.to_uppercase())),
                "str.lower" => Ok(Value::Str(text(0)?.to_lowercase())),
                "str.replace" => Ok(Value::Str(text(0)?.replace(text(1)?.as_str(), &text(2)?))),
                "str.join" => match vals.get(0) { Some(Value::List(items)) => { let sep = text(1)?; let mut parts = Vec::new(); for it in items { match it { Value::Str(s) => parts.push(s.clone()), o => return rt(c.line, format!("std::str.join: the list holds {}, not text", o.kind())) } } Ok(Value::Str(parts.join(&sep))) } _ => rt(c.line, "std::str.join[lend 'parts', sep]") },
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
                other => rt(c.line, format!("std::{other} is not in the oracle's std (it has str.split/contains/starts_with/ends_with/trim/upper/lower/replace/join, str.at.<unit>, str.slice.grapheme, str.find.grapheme)")),
            };
        }
        if c.handler.is_some() && !path.starts_with("input.") { return rt(c.line, format!("std::{path} cannot fail; drop the `otherwise`")); }
        match path.as_str() {
            "input.line" | "input.all" => {
                // a boundary (decided 2026-09-20): text as it came; `closed` when there is none
                if !c.args.values.is_empty() { return rt(c.line, format!("std::{path} takes no arguments")); }
                let fails = vec![FailCase { name: "closed".into(), ty: None }];
                self.check_handler(c.handler.as_ref(), &fails, &format!("std::{path}"), c.line)?;
                let mut text = String::new();
                let got = if path == "input.line" {
                    match std::io::stdin().read_line(&mut text) { Ok(0) => false, Ok(_) => { if text.ends_with('\n') { text.pop(); if text.ends_with('\r') { text.pop(); } } true }, Err(_) => false }
                } else {
                    use std::io::Read;
                    match std::io::stdin().read_to_string(&mut text) { Ok(0) => false, Ok(_) => true, Err(_) => false }
                };
                if !got { return self.handle_failure(c.handler.as_ref().unwrap(), c.line, &format!("std::{path}"), "closed".into(), None, value_used); }
                Ok(Value::Str(text))
            }
            "round" | "floor" | "ceil" | "trunc" => {
                // the rounding toolbox (std, provisional names): a bin in, a whole bin out — then `to.int32[…]`
                if c.args.values.len() != 1 { return rt(c.line, format!("std::{path} takes one bin")); }
                let (a, _) = self.value(&c.args.values[0])?;
                match a {
                    Value::Bin(x) => Ok(Value::Bin(match path.as_str() { "round" => x.round(), "floor" => x.floor(), "ceil" => x.ceil(), _ => x.trunc() })),
                    other => rt(c.line, format!("std::{path} takes a bin, got {}", other.kind())),
                }
            }
            "time.wait" => {
                // the oracle does not sleep: time is the outside world, and nothing here observes it
                if c.args.values.len() != 1 { return rt(c.line, "std::time.wait takes one duration: `std::time.wait[5 seconds]`"); }
                let (d, _) = self.value(&c.args.values[0])?;
                if !matches!(d, Value::Duration(_)) { return rt(c.line, format!("std::time.wait takes a duration (`5 seconds`), got {}", d.kind())); }
                Ok(Value::Nothing)
            }
            "time.after" | "time.every" => {
                if c.args.values.len() != 1 { return rt(c.line, format!("std::{path} takes one duration")); }
                let (d, _) = self.value(&c.args.values[0])?;
                let Value::Duration(n) = d else { return rt(c.line, format!("std::{path} takes a duration (`5 seconds`), got {}", d.kind())) };
                self.next_chan_id += 1;
                let timer = if path == "time.after" { Timer::After { fired: false, nanos: n } } else { Timer::Every { nanos: n } };
                Ok(Value::Chan(Rc::new(RefCell::new(Chan { queue: std::collections::VecDeque::new(), cap: None, timer: Some(timer), id: self.next_chan_id }))))
            }
            "div.trunc" | "mod.trunc" => {
                // hardware truncation, by name (decided 2026-09-20); the language's `/` and `mod` floor
                if c.args.values.len() != 2 { return rt(c.line, format!("std::{path} takes two integers")); }
                let (a, _) = self.value(&c.args.values[0])?;
                let (b, _) = self.value(&c.args.values[1])?;
                match (a, b) {
                    (Value::Int(_), Value::Int(0)) => site(c.line, "division by zero"),
                    (Value::Int(x), Value::Int(y)) => Ok(Value::Int(if path == "div.trunc" { x.checked_div(y) } else { x.checked_rem(y) }.ok_or_else(|| RuntimeError { line: c.line, msg: "integer overflow".into(), exit: None, site: true, leave: None })?)),
                    (a, b) => rt(c.line, format!("std::{path} takes two integers, got {} and {}", a.kind(), b.kind())),
                }
            }
            "print.stdout" | "printu.stdout" | "print.stderr" | "printu.stderr"
            | "this_will_print_the_useful_numbers.this_will_go_to_the_standard_output"
            | "this_will_print_the_useful_numbers.this_will_go_to_the_standard_error" => {
                // Since 2026-09-19 print takes any number of values and renders each —
                // `print` in stored form, `printu` in useful form — with nothing between.
                // The long form is the owner's answer to "printu is cryptic" (2026-09-21).
                let useful = path.starts_with("printu") || path.starts_with("this_will_print_the_useful");
                let path = if path.ends_with("standard_error") { "printu.stderr".to_string() } else { path };
                if c.args.values.is_empty() { return rt(c.line, format!("std::{path} takes at least one value")); }
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
                if path.ends_with("stderr") { eprint!("{text}"); }
                else { write!(self.out, "{text}").map_err(|e| RuntimeError { line: c.line, msg: e.to_string(), exit: None, site: false, leave: None })?; }
                Ok(Value::Nothing)
            }
            "printf.stdout" | "printf.stderr" => {
                // C's print, in full (owner, 2026-09-22): escapes and `%` formatting, checked
                // against the arguments' types because the format is a literal — a
                // non-literal format is a nocheck site.
                if c.args.values.is_empty() { return rt(c.line, format!("std::{path} takes a format text first: `std::{path}[\"%d items\\n\", 'n']`")); }
                let fmt = match &c.args.values[0] {
                    crate::ast::Value::Expr(Expr::Text(t, _)) => t.clone(),
                    _ => {
                        if self.nocheck_depth == 0 { return rt(c.line, format!("std::{path}: the format is not a literal, so its `%`s cannot be checked against the arguments — that is a nocheck site: `nocheck std::{path}[…]`")); }
                        match self.value(&c.args.values[0])?.0 { Value::Str(t) => t, other => return rt(c.line, format!("std::{path} takes a text as its format, got {}", other.kind())) }
                    }
                };
                let mut vals = Vec::new();
                for a in &c.args.values[1..] { vals.push(self.value(a)?.0); }
                let text = c_format(&fmt, &vals).map_err(|m| RuntimeError { line: c.line, msg: format!("std::{path}: {m}"), exit: None, site: false, leave: None })?;
                if path.ends_with("stderr") { eprint!("{text}"); }
                else { write!(self.out, "{text}").map_err(|e| RuntimeError { line: c.line, msg: e.to_string(), exit: None, site: false, leave: None })?; }
                Ok(Value::Nothing)
            }
            "printc.stdout" | "printc.stderr" | "printrs.stdout" | "printrs.stderr" | "c.printf" | "c.fprintf.stderr"
            | "rs.print" | "rs.println" | "rs.eprint" | "rs.eprintln" =>
                rt(c.line, format!("std::{path} was removed (2026-09-20): the borrowed C/Rust prints are gone; use std::print / std::printu")),
            "exit" => {
                let (v, _) = self.single(&c.args, c.line)?;
                match v {
                    Value::Int(i) if (0..=255).contains(&i) => Err(RuntimeError { line: c.line, msg: String::new(), exit: Some(i as u8), site: false, leave: None }),
                    Value::Int(i) => site(c.line, format!("exit code {i} does not fit uint8 (0 to 255)")),
                    o => rt(c.line, format!("std::exit takes a uint8, got {}", o.kind())),
                }
            }
            other => rt(c.line, format!("std::{other} does not exist in the oracle (it knows print/printu .stdout/.stderr, exit, div.trunc, mod.trunc, str.*)")),
        }
    }
}

/// A type's shape, matching `shape_of`: the kind, or the struct/enum name.
fn ty_shape(t: &Ty) -> String {
    match t {
        Ty::Int(_) | Ty::Uint(_) => "integer".into(), Ty::Bin(_) | Ty::Deci(_) => "bin".into(), Ty::Bool => "bool".into(),
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
fn round_to_width(t: &Ty, v: Value) -> Value {
    match (t, v) {
        (Ty::Bin(32), Value::Bin(x)) => Value::Bin((x as f32) as f64),
        (Ty::Bin(16), Value::Bin(x)) => Value::Bin(round_f16(x)),
        (Ty::Ranged(b, _, _), v) => round_to_width(b, v),
        (Ty::List(e) | Ty::Array(e), Value::List(items)) => Value::List(items.into_iter().map(|i| round_to_width(e, i)).collect()),
        (Ty::Simd(e, _), Value::Simd(lanes)) => Value::Simd(lanes.into_iter().map(|i| round_to_width(e, i)).collect()),
        (_, v) => v,
    }
}

fn int_bits(w: &IntWidth) -> u32 { match w { IntWidth::W8 => 8, IntWidth::W16 => 16, IntWidth::W32 => 32, IntWidth::W64 => 64, IntWidth::W128 => 127 } }

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
            // a list's, a map's or a channel's count: never negative, no natural ceiling
            _ => Bound::Int(if high { i128::MAX } else { 0 }),
        },
        other => other.clone(),
    }
}

fn const_key(module: &[String], name: &str) -> String { format!("{}/{name}", module.join(".")) }
fn join_path(module: &[String], name: &str) -> String { if module.is_empty() { name.to_string() } else { format!("{}.{name}", module.join(".")) } }
fn root_or(module: &[String]) -> String { if module.is_empty() { "the root (limb.cpls)".into() } else { module.join(".") } }

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
        (Value::Int(_), Value::Int(_)) | (Value::Bin(_), Value::Bin(_)) | (Value::Bool(_), Value::Bool(_))
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

fn grapheme_count(s: &str) -> usize {
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



/// C's format language, as `std::printf` accepts it (design/strings.md, 2026-09-22): the
/// escapes that mean a character, and `%` with flags `-`, `0`, a width, `.precision`, and the
/// conversions d i s f x X c %. Every `%` is checked against its argument's type and the
/// count — the mismatch C leaves undefined is an error here.
fn c_format(fmt: &str, vals: &[Value]) -> Result<String, String> {
    let chars: Vec<char> = fmt.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    let mut next = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\\' {
            i += 1;
            let e = *chars.get(i).ok_or("a `\\` at the end of the format escapes nothing")?;
            match e {
                'n' => out.push('\n'), 't' => out.push('\t'), 'r' => out.push('\r'), '\\' => out.push('\\'),
                '"' => out.push('"'), '\'' => out.push('\''), '0' => out.push('\0'),
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
                other => return Err(format!("`\\{other}` is not an escape (n t r \\\\ \\\" \\' 0 xHH uHHHH UHHHHHHHH are)")),
            }
            i += 1; continue;
        }
        if ch != '%' { out.push(ch); i += 1; continue; }
        i += 1;
        if chars.get(i) == Some(&'%') { out.push('%'); i += 1; continue; }
        let mut left = false; let mut zero = false;
        while let Some(&f) = chars.get(i) { match f { '-' => left = true, '0' => zero = true, '+' | ' ' | '#' => {}, _ => break } i += 1; }
        let mut width = String::new(); while let Some(&d) = chars.get(i) { if d.is_ascii_digit() { width.push(d); i += 1; } else { break; } }
        let mut prec: Option<usize> = None;
        if chars.get(i) == Some(&'.') { i += 1; let mut p = String::new(); while let Some(&d) = chars.get(i) { if d.is_ascii_digit() { p.push(d); i += 1; } else { break; } } prec = Some(p.parse().unwrap_or(0)); }
        while let Some(&l) = chars.get(i) { if l == 'l' || l == 'h' || l == 'z' { i += 1; } else { break; } }   // C's length modifiers: ignored, types are known
        let conv = *chars.get(i).ok_or("a `%` at the end of the format converts nothing")?; i += 1;
        let v = vals.get(next).ok_or_else(|| format!("the format has more `%`s than arguments (argument {} is missing)", next + 1))?; next += 1;
        let body = match (conv, v) {
            ('d' | 'i', Value::Int(n)) => n.to_string(),
            ('u', _) => return Err("`%u` is refused; `%d` prints any integer, unsigned included".into()),
            ('x', Value::Int(n)) => format!("{:x}", n), ('X', Value::Int(n)) => format!("{:X}", n), ('o', Value::Int(n)) => format!("{:o}", n),
            ('f' | 'F', Value::Bin(b)) => format!("{:.*}", prec.unwrap_or(6), b),
            ('e', Value::Bin(b)) => format!("{:.*e}", prec.unwrap_or(6), b),
            ('g', Value::Bin(b)) => format!("{}", b),
            ('s', Value::Str(t)) => match prec { Some(p) => t.chars().take(p).collect(), None => t.clone() },
            ('s', Value::Char(_, t)) => t.clone(),
            ('c', Value::Char(_, t)) => t.clone(),
            ('c', Value::Int(n)) => char::from_u32(*n as u32).map(|c| c.to_string()).ok_or("`%c` of a number that is not a character")?,
            ('p' | 'n', _) => return Err(format!("`%{conv}` does not exist here (no pointers to print, no `%n`)")),
            (c, v) => return Err(format!("`%{c}` does not take a {} (argument {})", v.kind(), next)),
        };
        let w: usize = width.parse().unwrap_or(0);
        if body.chars().count() >= w { out.push_str(&body); }
        else if left { out.push_str(&body); out.push_str(&" ".repeat(w - body.chars().count())); }
        else if zero && matches!(conv, 'd' | 'i' | 'x' | 'X' | 'f' | 'F' | 'o') {
            let (sign, digits) = if let Some(rest) = body.strip_prefix('-') { ("-", rest.to_string()) } else { ("", body.clone()) };
            out.push_str(sign); out.push_str(&"0".repeat(w - body.chars().count())); out.push_str(&digits);
        }
        else { out.push_str(&" ".repeat(w - body.chars().count())); out.push_str(&body); }
    }
    if next < vals.len() { return Err(format!("the format has fewer `%`s than arguments ({} argument(s) unused)", vals.len() - next)); }
    Ok(out)
}

/// A statement's own line: every `Stmt` variant ends with `line`, so it is the last one printed.
fn stmt_line(s: &Stmt) -> usize {
    let t = format!("{s:?}");
    t.rfind("line: ").and_then(|i| t[i + 6..].split(|c: char| !c.is_ascii_digit()).next().and_then(|d| d.parse().ok())).unwrap_or(0)
}
