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
}

fn rt<T>(line: usize, msg: impl Into<String>) -> Result<T, RuntimeError> {
    Err(RuntimeError { line, msg: msg.into(), exit: None })
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum SlotKind { Owned, Lent, LentMut, Const }

#[derive(Clone)]
struct Slot {
    cell: Rc<RefCell<Value>>,
    ty: Option<Ty>,
    kind: SlotKind,
    alive: bool,     // false after `give`
    immut: bool,     // stated `immut`: assignment is an error
}

enum Flow { Normal, Break, Continue, Return(Value) }

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
    /// Limb-level `const`s: visible everywhere, never assignable, every read a fresh value.
    consts: HashMap<String, Slot>,
    scopes: Vec<HashMap<String, Slot>>,
    out: Box<dyn Write>,
}

impl<'p> Interp<'p> {
    pub fn new(prog: &'p Program, out: Box<dyn Write>) -> Self {
        Interp { prog, consts: HashMap::new(), scopes: vec![HashMap::new()], out }
    }

    /// Ok(code): the program ran; Err: it failed.
    pub fn run(&mut self) -> Result<u8, RuntimeError> {
        // Consts first, in order: each may use the ones above it. Precog would
        // fold these at compile time; the oracle evaluates them before MAIN.
        for c in &self.prog.consts {
            if self.consts.contains_key(&c.name) { return rt(c.line, format!("const '{}' is declared twice", c.name)); }
            if let Some(call) = first_std_call(&c.value) {
                return rt(c.line, format!("a const's value must be known at compile time; `std::{call}` is an effect, not a value"));
            }
            if c.value.values.is_empty() { return rt(c.line, format!("const '{}' needs a value", c.name)); }
            let (v, prov) = self.bracket(&c.value, c.line)?;
            if matches!(prov, Prov::Borrowed(..)) { return rt(c.line, format!("const '{}' cannot hold a lend", c.name)); }
            let v = self.fit(v, prov, c.ty.as_ref(), c.line)?;
            self.consts.insert(c.name.clone(), Slot { cell: Rc::new(RefCell::new(v)), ty: c.ty.clone(), kind: SlotKind::Const, alive: true, immut: true });
        }
        let main = match &self.prog.main {
            Some(m) => m,
            None => return rt(0, "this Limb has no MAIN, so it is a library, not a program"),
        };
        match self.block(main) {
            Ok(Flow::Normal) => Ok(0),
            Ok(Flow::Return(_)) => rt(0, "`return` inside MAIN: MAIN is a block, not a function; use std::exit[…] for a code"),
            Ok(Flow::Break) | Ok(Flow::Continue) => rt(0, "`break`/`continue` outside a loop"),
            Err(RuntimeError { exit: Some(code), .. }) => Ok(code),
            Err(e) => Err(e),
        }
    }

    // ---- scopes -----------------------------------------------------------

    fn push(&mut self) { self.scopes.push(HashMap::new()); }
    fn pop(&mut self) { self.scopes.pop(); }
    fn lookup(&self, name: &str) -> Option<&Slot> { self.scopes.iter().rev().find_map(|s| s.get(name)).or_else(|| self.consts.get(name)) }
    fn lookup_mut(&mut self, name: &str) -> Option<&mut Slot> { self.scopes.iter_mut().rev().find_map(|s| s.get_mut(name)) }
    fn is_const(&self, name: &str) -> bool { self.consts.contains_key(name) && !self.scopes.iter().any(|s| s.contains_key(name)) }

    fn declare(&mut self, name: &str, slot: Slot, line: usize) -> Result<(), RuntimeError> {
        self.declare_shadow(name, slot, false, line)
    }

    /// Decided 2026-09-14: a visible name may only be redeclared with `$:=` /
    /// `$=`, and never if the visible one is `immut`.
    fn declare_shadow(&mut self, name: &str, slot: Slot, shadow: bool, line: usize) -> Result<(), RuntimeError> {
        match (self.lookup(name), shadow) {
            (Some(_), false) => return rt(line, format!("'{name}' is already declared in an enclosing scope; to shadow it deliberately write `$:=` or `$=`")),
            (None, true) => return rt(line, format!("`$` shadows an existing '{name}', but none is visible here")),
            (Some(existing), true) if existing.kind == SlotKind::Const => return rt(line, format!("'{name}' is a const and cannot be shadowed")),
            (Some(existing), true) if existing.immut => return rt(line, format!("'{name}' is `immut` and cannot be shadowed")),
            _ => {}
        }
        self.scopes.last_mut().unwrap().insert(name.to_string(), slot);
        Ok(())
    }

    // ---- statements -------------------------------------------------------

    fn block(&mut self, stmts: &[Stmt]) -> Result<Flow, RuntimeError> {
        self.push();
        let r = self.stmts(stmts);
        self.pop();
        r
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
        match s {
            Stmt::Decl { name, mutability, ty, value, shadow, line } => {
                let (v, prov) = if value.values.is_empty() {
                    match ty {
                        Some(Ty::List(_)) => (Value::List(Vec::new()), Prov::Fresh),
                        Some(Ty::Array(_)) => return rt(*line, format!("an empty `array` '{name}' could never be filled; make it a `list`")),
                        Some(t) => return rt(*line, format!("`[]` is an empty list, but '{name}' is {}", ty_name(t))),
                        None => return rt(*line, format!("`[]` with `:=` says nothing about what '{name}' is; write `list.empty`")),
                    }
                } else { self.bracket(value, *line)? };
                let immut = *mutability == Some(Mutability::Immut);
                let slot = match prov {
                    Prov::Bare(n) => return rt(*line, format!("declaring '{name}' from bare '{n}', which owns a {}: say `give '{n}'`, `copy '{n}'`, or `lend '{n}'`", v.kind())),
                    Prov::Borrowed(cell, is_mut) => {
                        if ty.is_some() { return rt(*line, format!("'{name}' would hold a lend; a lent variable takes no explicit type here (use `:=`)")); }
                        Slot { cell, ty: None, kind: if is_mut { SlotKind::LentMut } else { SlotKind::Lent }, alive: true, immut }
                    }
                    _ => {
                        let v = self.fit(v, prov, ty.as_ref(), *line)?;
                        Slot { cell: Rc::new(RefCell::new(v)), ty: ty.clone(), kind: SlotKind::Owned, alive: true, immut }
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
                if immut { return rt(*line, format!("'{name}' cannot be assigned to: it is `immut` (a loop's counter or element belongs to the loop)")); }
                if kind == SlotKind::Lent { return rt(*line, format!("'{name}' is lent (shared); it cannot be assigned to — a `lent.mut` could")); }
                let (v, prov) = self.bracket(value, *line)?;
                match prov {
                    Prov::Bare(n) => return rt(*line, format!("assigning bare '{n}' to '{name}': '{n}' owns a {} — say `give '{n}'`, `copy '{n}'`, or `lend '{n}'`", v.kind())),
                    Prov::Borrowed(..) => return rt(*line, format!("assigning a lend to '{name}': a lend is temporary and cannot be stored in an owned variable (`copy` it or `give` it)")),
                    _ => {}
                }
                let v = self.fit(v, prov, ty.as_ref(), *line)?;
                let slot = self.lookup_mut(name).unwrap();
                slot.alive = true; // assigning to a given-away variable revives it
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
                let (i, _) = self.eval(index)?;
                let i = match i { Value::Int(i) => i, other => return rt(*line, format!("a position is an integer, got {}", other.kind())) };
                let (v, prov) = self.bracket(value, *line)?;
                match prov {
                    Prov::Bare(n) => return rt(*line, format!("storing bare '{n}' into '{name}': say `give '{n}'` or `copy '{n}'`")),
                    Prov::Borrowed(..) => return rt(*line, "a list owns its elements; store a `copy` or `give`, not a lend"),
                    _ => {}
                }
                let elem_ty = match &ty { Some(Ty::List(t)) | Some(Ty::Array(t)) => Some((**t).clone()), _ => None };
                let v = self.fit(v, prov, elem_ty.as_ref(), *line)?;
                let mut cur = cell.borrow_mut();
                let items = match &mut *cur { Value::List(items) => items, other => return rt(*line, format!("'{name}' is {}, not a list or array", other.kind())) };
                if i < 1 || i as usize > items.len() {
                    return rt(*line, format!("position {i} is out of range for '{name}', which has {} element(s) (1 to {}) — default check tier", items.len(), items.len()));
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
                let sname = match &*cell.borrow() { Value::Struct(n, _) => n.clone(), other => return rt(*line, format!("'{name}' is {}, not a struct; it has no field '{field}'", other.kind())) };
                let def = self.prog.structs.iter().find(|d| d.name == sname).cloned().unwrap();
                let fdef = match def.fields.iter().find(|f| &f.name == field) { Some(f) => f.clone(), None => return rt(*line, format!("{sname} has no field '{field}'")) };
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
                    Flow::Return(v) => return Ok(Flow::Return(v)),
                    Flow::Normal | Flow::Continue => {}
                }
            },
            Stmt::While { cond, body, line } => {
                while self.cond(cond, *line)? {
                    match self.block(body)? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Normal | Flow::Continue => {}
                    }
                }
                Ok(Flow::Normal)
            }
            Stmt::For { name, perm, over, body, line, .. } => {
                let (items, kind) = self.for_source(over, *line)?;
                // perm: the variable outlives the loop, holding whatever the loop last gave it.
                let cell = Rc::new(RefCell::new(Value::Nothing));
                let mut ran = false;
                for item in items {
                    ran = true;
                    *cell.borrow_mut() = item;
                    self.push();
                    let r = self.declare(name, Slot { cell: cell.clone(), ty: None, kind, alive: true, immut: true }, *line)
                        .and_then(|_| self.stmts(body));
                    self.pop();
                    match r? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Normal | Flow::Continue => {}
                    }
                }
                if *perm {
                    if !ran { return rt(*line, format!("`perm` '{name}' would outlive a loop over an empty list, but was never given a value")); }
                    self.declare(name, Slot { cell, ty: None, kind: SlotKind::Owned, alive: true, immut: false }, *line)?;
                }
                Ok(Flow::Normal)
            }
            Stmt::Range { name, perm, ty, bounds, body, line, .. } => {
                let mut nums = Vec::new();
                for v in &bounds.values {
                    let (val, prov) = self.value(v)?;
                    let val = self.fit(val, prov, ty.as_ref(), *line)?;
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
                    let r = self.declare(name, Slot { cell: cell.clone(), ty: ty.clone(), kind: SlotKind::Owned, alive: true, immut: true }, *line)
                        .and_then(|_| self.stmts(body));
                    self.pop();
                    match r? {
                        Flow::Break => { broke = true; break; }
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Normal | Flow::Continue => {}
                    }
                    match i.checked_add(step) { Some(n) => i = n, None => break }
                }
                let _ = broke;
                if *perm {
                    // 'i' keeps whatever the loop last gave it: the last value the body ran
                    // with, or the start if the body never ran.
                    self.declare(name, Slot { cell, ty: ty.clone(), kind: SlotKind::Owned, alive: true, immut: false }, *line)?;
                }
                Ok(Flow::Normal)
            }
            Stmt::Break { .. } => Ok(Flow::Break),
            Stmt::Continue { .. } => Ok(Flow::Continue),
            Stmt::Return { value, line } => {
                let v = match value {
                    None => Value::Nothing,
                    Some(b) => {
                        let (v, prov) = self.bracket(b, *line)?;
                        match prov {
                            Prov::Bare(n) => return rt(*line, format!("returning bare '{n}', which owns a {}: say `give '{n}'` (or `copy`)", v.kind())),
                            Prov::Borrowed(..) => return rt(*line, "returning a lend: the oracle does not model lifetimes, so a function cannot return a borrow"),
                            _ => v,
                        }
                    }
                };
                Ok(Flow::Return(v))
            }
            Stmt::Call { call, .. } => { self.call(call)?; Ok(Flow::Normal) }
            Stmt::BuiltinStmt { head, op, args, line } => { self.builtin(head, op, args, *line)?; Ok(Flow::Normal) }
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
        let expr = match &b.values[0] { crate::ast::Value::Expr(e) => e, crate::ast::Value::Pieces(_) => return rt(line, "`for` needs a list, not text pieces") };
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
    fn fit(&self, v: Value, prov: Prov, ty: Option<&Ty>, line: usize) -> Result<Value, RuntimeError> {
        let Some(t) = ty else { return Ok(v) };
        let v = match (t, &v, &prov) {
            (Ty::Bin(_), Value::Int(i), Prov::Literal) => Value::Bin(*i as f64),
            _ => v,
        };
        self.check_type(&v, t, line)?;
        Ok(v)
    }

    fn check_type(&self, v: &Value, ty: &Ty, line: usize) -> Result<(), RuntimeError> {
        let name = ty_name(ty);
        match (ty, v) {
            (Ty::Int(_) | Ty::Uint(_), Value::Int(i)) => {
                let (lo, hi) = int_range(ty).unwrap();
                if *i < lo || *i > hi { return rt(line, format!("{i} does not fit {name} (range {lo} to {hi})")); }
                Ok(())
            }
            (Ty::Int(_) | Ty::Uint(_), Value::Bin(b)) => rt(line, format!("{} is a bin, not {name}; there is no implicit conversion between numeric families", exact_decimal(*b))),
            (Ty::Bin(w), Value::Bin(b)) => {
                match w {
                    16 => if b.abs() > 65504.0 { return rt(line, format!("{b} does not fit bin16 (max 65504)")); },
                    32 => if b.abs() > f32::MAX as f64 { return rt(line, format!("{b} does not fit bin32")); },
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
            (Ty::Struct(want), Value::Struct(have, _)) => {
                if want == have { Ok(()) } else { rt(line, format!("expected a {want}, got a {have}")) }
            }
            (Ty::Struct(want), v) => {
                if !self.prog.structs.iter().any(|sd| &sd.name == want) { return rt(line, format!("no struct named `{want}` is declared")); }
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
                let mut items = Vec::new();
                for v in &b.values {
                    let (val, prov) = self.value(v)?;
                    match prov {
                        Prov::Bare(name) => return rt(line, format!("bare '{name}' inside a list: say `give`, `copy`, or `lend`")),
                        Prov::Borrowed(..) => return rt(line, "a lend inside a list: a list owns its elements (`copy` or `give`)"),
                        _ => {}
                    }
                    items.push(val);
                }
                Ok((Value::List(items), Prov::Fresh))
            }
        }
    }

    fn value(&mut self, v: &crate::ast::Value) -> Result<(Value, Prov), RuntimeError> {
        self.value_mode(v, true)
    }

    /// A number piece becomes text in its stored form, everywhere (decided
    /// 2026-09-14). `printu` is the one place that renders its own pieces in
    /// the useful form (`stored = false`).
    fn value_mode(&mut self, v: &crate::ast::Value, stored: bool) -> Result<(Value, Prov), RuntimeError> {
        match v {
            crate::ast::Value::Expr(e) => self.eval(e),
            crate::ast::Value::Pieces(pieces) => {
                // Whitespace joins pieces into one text. Provisional (open):
                // whitespace is a separator only — no implicit space.
                let mut out = String::new();
                for p in pieces {
                    let (val, _) = self.eval(p)?;
                    match val {
                        Value::Nothing => return rt(p.line(), "a call that returns nothing cannot be a text piece"),
                        other => out.push_str(&if stored { other.show_stored() } else { other.show_useful() }),
                    }
                }
                Ok((Value::Str(out), Prov::Fresh))
            }
        }
    }

    fn read_var(&self, name: &str, line: usize) -> Result<Value, RuntimeError> {
        match self.lookup(name) {
            None => rt(line, format!("'{name}' is not declared")),
            Some(s) if !s.alive => rt(line, format!("'{name}' was given away and no longer owns anything")),
            Some(s) => Ok(s.cell.borrow().clone()),
        }
    }

    fn eval(&mut self, e: &Expr) -> Result<(Value, Prov), RuntimeError> {
        let line = e.line();
        match e {
            Expr::Int(i, _) => Ok((Value::Int(*i), Prov::Literal)),
            Expr::Bin(b, _) => Ok((Value::Bin(*b), Prov::Literal)),
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
                let (kind, alive, cell) = match self.lookup(name) {
                    Some(s) => (s.kind, s.alive, s.cell.clone()),
                    None => return rt(line, format!("'{name}' is not declared")),
                };
                if !alive { return rt(line, format!("'{name}' was already given away")); }
                if kind == SlotKind::Const { return rt(line, format!("'{name}' is a const: nobody owns it, every use is a fresh value — write it bare, no `give`/`copy`/`lend`")); }
                let v = cell.borrow().clone();
                if !v.is_owned() {
                    return rt(line, format!("'{name}' is a plain {} and owns nothing; it copies by itself — no `give`/`copy`/`lend`", v.kind()));
                }
                match verb {
                    Verb::Give => {
                        if kind != SlotKind::Owned { return rt(line, format!("'{name}' is lent, not owned; you cannot give away what you do not own")); }
                        self.lookup_mut(name).unwrap().alive = false;
                        Ok((v, Prov::Given))
                    }
                    Verb::Copy => Ok((v, Prov::Copied)),
                    Verb::Lend => Ok((v, Prov::Borrowed(cell, false))),
                    Verb::LendMut => {
                        if kind == SlotKind::Lent { return rt(line, format!("'{name}' is lent (shared); it cannot be lent mutably")); }
                        Ok((v, Prov::Borrowed(cell, true)))
                    }
                }
            }
            Expr::Call(c) => Ok((self.call(c)?, Prov::Fresh)),
            Expr::Nested(b) => self.bracket(b, line),
            Expr::Index(name, idx, _) => {
                let v = self.read_var(name, line)?;
                let items = match v { Value::List(items) => items, other => return rt(line, format!("'{name}' is {}, not a list or array; it cannot be indexed", other.kind())) };
                let (i, _) = self.eval(idx)?;
                let i = match i { Value::Int(i) => i, other => return rt(line, format!("an index is an integer, got {}", other.kind())) };
                // 1-based (decided 2026-09-17): 'xs'[pos 1] is the first; 'xs'[pos 0] does not exist.
                if i < 1 || i as usize > items.len() {
                    return rt(line, format!("position {i} is out of range for '{name}', which has {} element(s) (1 to {}) — default check tier", items.len(), items.len()));
                }
                let item = items[(i - 1) as usize].clone();
                let prov = if item.is_owned() { Prov::Bare(format!("{name}'[{i}]'")) } else { Prov::Plain };
                Ok((item, prov))
            }
            Expr::Field(name, field, _) => {
                let v = self.read_var(name, line)?;
                let (sname, fields) = match v { Value::Struct(n, f) => (n, f), other => return rt(line, format!("'{name}' is {}, not a struct; it has no field '{field}'", other.kind())) };
                match fields.into_iter().find(|(k, _)| k == field) {
                    Some((_, fv)) => {
                        let prov = if fv.is_owned() { Prov::Bare(format!("{name}'.{field} (write `copy '{name}'.{field}` or `lend '{name}'.{field}`; the field is owned by '{name}")) } else { Prov::Plain };
                        Ok((fv, prov))
                    }
                    None => rt(line, format!("{sname} has no field '{field}'")),
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
                        None => return rt(line, format!("{sname} has no field '{field}'")),
                    },
                    other => return rt(line, format!("'{name}' is {}, not a struct", other.kind())),
                };
                if !fv.is_owned() { return rt(line, format!("'{name}'.{field} is a plain {} and owns nothing; it copies by itself — no `give`/`copy`/`lend`", fv.kind())); }
                match verb {
                    Verb::Give => {
                        // Decided 2026-09-18: giving a field away consumes the whole variable.
                        if kind != SlotKind::Owned { return rt(line, format!("'{name}' is lent, not owned; you cannot give away part of what you do not own")); }
                        self.lookup_mut(name).unwrap().alive = false;
                        Ok((fv, Prov::Given))
                    }
                    Verb::Copy => Ok((fv, Prov::Copied)),
                    Verb::Lend | Verb::LendMut => {
                        if *verb == Verb::LendMut && kind == SlotKind::Lent { return rt(line, format!("'{name}' is lent (shared); its fields cannot be lent mutably")); }
                        // The oracle lends a field as a snapshot; write-through into a field via lend.mut is not modelled.
                        Ok((fv.clone(), Prov::Borrowed(Rc::new(RefCell::new(fv)), *verb == Verb::LendMut)))
                    }
                }
            }
            Expr::Construct(sname, given, _) => {
                let def = match self.prog.structs.iter().find(|d| d.name == *sname) {
                    Some(d) => d.clone(),
                    None => return rt(line, format!("no struct named `{sname}` is declared (a call to a function is `{sname}[…]` with plain values, not `'field' = …`)")),
                };
                for (k, _) in given { if !def.fields.iter().any(|f| &f.name == k) { return rt(line, format!("{sname} has no field '{k}'")); } }
                let mut out = Vec::new();
                for f in &def.fields {
                    let b = match given.iter().find(|(k, _)| k == &f.name) {
                        Some((_, b)) => b,
                        None => return rt(line, format!("{sname} is missing '{}': every field must be given — there are no zero values", f.name)),
                    };
                    if given.iter().filter(|(k, _)| k == &f.name).count() > 1 { return rt(line, format!("'{}' is given twice", f.name)); }
                    let (v, prov) = self.bracket(b, line)?;
                    match prov {
                        Prov::Bare(n) => return rt(line, format!("bare '{n}' as field '{}': say `give '{n}'` or `copy '{n}'`", f.name)),
                        Prov::Borrowed(..) => return rt(line, format!("a struct owns its fields; '{}' cannot hold a lend", f.name)),
                        _ => {}
                    }
                    let v = self.fit(v, prov, Some(&f.ty), line)?;
                    out.push((f.name.clone(), v));
                }
                let val = Value::Struct(sname.clone(), out);
                let prov = if val.is_owned() { Prov::Fresh } else { Prov::Plain };
                Ok((val, prov))
            }
            Expr::Builtin(head, op, args, _) => {
                let v = self.builtin(head, op, args, line)?;
                let prov = if v.is_owned() { Prov::Fresh } else { Prov::Plain };
                Ok((v, prov))
            }
            Expr::Paren(inner, _) => self.eval(inner),
            Expr::Unary(op, inner, _) => {
                let (v, prov) = self.eval(inner)?;
                let keep = if matches!(prov, Prov::Literal) { Prov::Literal } else { Prov::Plain };
                match (op.as_str(), v) {
                    ("not", Value::Bool(b)) => Ok((Value::Bool(!b), Prov::Plain)),
                    ("not", o) => rt(line, format!("`not` needs a bool, got {}", o.kind())),
                    ("-", Value::Int(i)) => Ok((Value::Int(i.checked_neg().ok_or_else(|| RuntimeError { line, msg: "negation overflow".into(), exit: None })?), keep)),
                    ("-", Value::Bin(b)) => Ok((Value::Bin(-b), keep)),
                    ("-", o) => rt(line, format!("`-` needs a number, got {}", o.kind())),
                    _ => unreachable!(),
                }
            }
            Expr::Binary(op, l, r, _) => {
                let (a, pa) = self.eval(l)?;
                let (b, pb) = self.eval(r)?;
                // A bare literal adopts the other side's family.
                let (a, b) = match (&a, &b, &pa, &pb) {
                    (Value::Bin(_), Value::Int(i), _, Prov::Literal) => (a.clone(), Value::Bin(*i as f64)),
                    (Value::Int(i), Value::Bin(_), Prov::Literal, _) => (Value::Bin(*i as f64), b.clone()),
                    _ => (a, b),
                };
                let both_lit = matches!(pa, Prov::Literal) && matches!(pb, Prov::Literal);
                self.binary(op, a, b, line).map(|v| (v, if both_lit { Prov::Literal } else { Prov::Plain }))
            }
        }
    }

    fn binary(&self, op: &str, a: Value, b: Value, line: usize) -> Result<Value, RuntimeError> {
        use Value::*;
        let ovf = |what: &str| RuntimeError { line, msg: format!("integer overflow in `{what}` (default check tier: there is no Precog here to prove it away)"), exit: None };
        Ok(match (op, a, b) {
            ("+", Int(x), Int(y)) => Int(x.checked_add(y).ok_or_else(|| ovf("+"))?),
            ("-", Int(x), Int(y)) => Int(x.checked_sub(y).ok_or_else(|| ovf("-"))?),
            ("x", Int(x), Int(y)) => Int(x.checked_mul(y).ok_or_else(|| ovf("x"))?),
            ("/", Int(_), Int(0)) => return rt(line, "division by zero"),
            ("/", Int(x), Int(y)) => Int(x.checked_div(y).ok_or_else(|| ovf("/"))?), // provisional: truncates (open question)
            ("mod", Int(_), Int(0)) => return rt(line, "`mod` by zero"),
            ("mod", Int(x), Int(y)) => Int(x.checked_rem(y).ok_or_else(|| ovf("mod"))?), // provisional: sign follows the dividend
            ("xx", Int(x), Int(y)) => {
                if y < 0 { return rt(line, "a negative exponent on integers would produce a fraction"); }
                let mut acc: i128 = 1;
                for _ in 0..y { acc = acc.checked_mul(x).ok_or_else(|| ovf("xx"))?; }
                Int(acc)
            }
            ("+", Bin(x), Bin(y)) => Bin(x + y),
            ("-", Bin(x), Bin(y)) => Bin(x - y),
            ("x", Bin(x), Bin(y)) => Bin(x * y),
            ("/", Bin(x), Bin(y)) => Bin(x / y),
            ("mod", Bin(x), Bin(y)) => Bin(x % y),
            ("xx", Bin(x), Bin(y)) => Bin(x.powf(y)),
            ("+" | "-" | "x" | "/" | "mod" | "xx", Int(_), Bin(_)) | ("+" | "-" | "x" | "/" | "mod" | "xx", Bin(_), Int(_)) =>
                return rt(line, format!("`{op}` between an integer and a bin: there is no implicit conversion between numeric families")),
            ("==", x, y) => { same_family(&x, &y, line, "==")?; Bool(x == y) }
            ("!==", x, y) => { same_family(&x, &y, line, "!==")?; Bool(x != y) }
            ("<", Int(x), Int(y)) => Bool(x < y),
            (">", Int(x), Int(y)) => Bool(x > y),
            ("<==", Int(x), Int(y)) => Bool(x <= y),
            (">==", Int(x), Int(y)) => Bool(x >= y),
            ("<", Bin(x), Bin(y)) => Bool(x < y),
            (">", Bin(x), Bin(y)) => Bool(x > y),
            ("<==", Bin(x), Bin(y)) => Bool(x <= y),
            (">==", Bin(x), Bin(y)) => Bool(x >= y),
            ("and", Bool(x), Bool(y)) => Bool(x && y),
            ("or", Bool(x), Bool(y)) => Bool(x || y),
            (op, x, y) => return rt(line, format!("`{op}` cannot combine {} and {}", x.kind(), y.kind())),
        })
    }

    // ---- list / array operations (decided 2026-09-17) ---------------------

    fn builtin(&mut self, head: &str, op: &str, args: &Bracket, line: usize) -> Result<Value, RuntimeError> {
        if op == "empty" { return Ok(Value::List(Vec::new())); }
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
                if op == "append" {
                    match aprov {
                        Prov::Bare(n) => return rt(line, format!("appending bare '{n}': say `give '{n}'` or `copy '{n}'`")),
                        Prov::Borrowed(..) => return rt(line, "a list owns its elements; append a `copy` or `give`, not a lend"),
                        _ => {}
                    }
                    if let Some(Ty::List(inner)) = self.lookup(&var_name).and_then(|s| s.ty.clone()) { self.check_type(&arg, &inner, line)?; }
                    items.push(arg);
                } else {
                    let i = match arg { Value::Int(i) => i, other => return rt(line, format!("`list.remove` takes a 1-based position, got {}", other.kind())) };
                    if i < 1 || i as usize > items.len() { return rt(line, format!("position {i} is out of range for '{var_name}', which has {} element(s)", items.len())); }
                    items.remove((i - 1) as usize);
                }
                *cell.borrow_mut() = Value::List(items);
                Ok(Value::Nothing)
            }
            _ => unreachable!(),
        }
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
        if c.path.len() != 1 { return rt(c.line, format!("`{}`: a function in this Limb is a bare name; `.` narrows types and keywords, not calls", c.path.join("."))); }
        let name = &c.path[0];
        let func = match self.prog.funcs.iter().find(|f| &f.name == name) {
            Some(f) => f,
            None => return rt(c.line, format!("no function `{name}` in this Limb")),
        };
        if c.args.values.len() != func.params.len() {
            return rt(c.line, format!("`{name}` takes {} argument(s), given {}", func.params.len(), c.args.values.len()));
        }

        // Arguments against the parameter contract (design/ownership-syntax.md).
        let mut bound: Vec<(String, Slot)> = Vec::new();
        for (arg, param) in c.args.values.iter().zip(&func.params) {
            let (v, prov) = self.value(arg)?;
            let pn = &param.name;
            let slot = match param.kind {
                ParamKind::Owned => {
                    match &prov {
                        Prov::Bare(n) => return rt(c.line, format!("`{name}` takes ownership of '{pn}' — pass `give '{n}'` or `copy '{n}'`")),
                        Prov::Borrowed(..) => return rt(c.line, format!("`{name}` takes ownership of '{pn}' but you lent it; `give` or `copy` instead")),
                        _ => {}
                    }
                    let v = self.fit(v, prov, param.ty.as_ref(), c.line)?;
                    Slot { cell: Rc::new(RefCell::new(v)), ty: param.ty.clone(), kind: SlotKind::Owned, alive: true, immut: param.mutability == Some(Mutability::Immut) }
                }
                ParamKind::Lent | ParamKind::LentMut => {
                    let want_mut = param.kind == ParamKind::LentMut;
                    let kind = if want_mut { SlotKind::LentMut } else { SlotKind::Lent };
                    match prov {
                        Prov::Borrowed(cell, is_mut) => {
                            if want_mut && !is_mut { return rt(c.line, format!("`{name}` may modify '{pn}': pass `lend.mut`, not `lend`")); }
                            if !want_mut && is_mut { return rt(c.line, format!("`{name}` only looks at '{pn}': pass `lend`, not `lend.mut`")); }
                            if let Some(t) = &param.ty { self.check_type(&v, t, c.line)?; }
                            Slot { cell, ty: param.ty.clone(), kind, alive: true, immut: false }
                        }
                        Prov::Bare(n) => return rt(c.line, format!("`{name}` only looks at '{pn}' — pass `lend '{n}'`")),
                        Prov::Given => return rt(c.line, format!("`{name}` only looks at '{pn}': it is `lent`, so the caller writes `lend`, not `give`")),
                        Prov::Copied => return rt(c.line, format!("`{name}` only looks at '{pn}': it is `lent`, so the caller writes `lend`, not `copy`")),
                        // A plain value, or a fresh temporary (literal, call result): nothing owns
                        // it, so it lives for the call and is lent by the call itself (decided 2026-09-14).
                        other => {
                            let v = self.fit(v, other, param.ty.as_ref(), c.line)?;
                            Slot { cell: Rc::new(RefCell::new(v)), ty: param.ty.clone(), kind, alive: true, immut: false }
                        }
                    }
                }
            };
            bound.push((pn.clone(), slot));
        }

        // A function's scopes are its own: it cannot see the caller's variables.
        let saved = std::mem::replace(&mut self.scopes, vec![HashMap::new()]);
        let flow = (|| {
            for (n, s) in bound { self.declare(&n, s, func.line)?; }
            self.stmts(&func.body)
        })();
        self.scopes = saved;

        let result = match flow? {
            Flow::Return(v) => v,
            Flow::Normal => Value::Nothing,
            Flow::Break | Flow::Continue => return rt(func.line, "`break`/`continue` outside a loop"),
        };
        match (&func.ret, &result) {
            (Some(t), Value::Nothing) => rt(c.line, format!("`{name}` is declared `-> {}` but returned nothing", ty_name(t))),
            (Some(t), v) => { self.check_type(v, t, c.line)?; Ok(result) }
            (None, _) => Ok(result),
        }
    }

    fn std_call(&mut self, c: &Call) -> Result<Value, RuntimeError> {
        let path = c.path.join(".");
        match path.as_str() {
            "print.stdout" | "printu.stdout" | "print.stderr" | "printu.stderr" => {
                let useful = path.starts_with("printu");
                if c.args.values.len() != 1 { return rt(c.line, format!("std::{path} takes one value")); }
                let (v, _) = self.value_mode(&c.args.values[0], !useful)?;
                let text = if useful { v.show_useful() } else { v.show_stored() };
                if path.ends_with("stderr") { eprint!("{text}"); }
                else { write!(self.out, "{text}").map_err(|e| RuntimeError { line: c.line, msg: e.to_string(), exit: None })?; }
                Ok(Value::Nothing)
            }
            "exit" => {
                let (v, _) = self.single(&c.args, c.line)?;
                match v {
                    Value::Int(i) if (0..=255).contains(&i) => Err(RuntimeError { line: c.line, msg: String::new(), exit: Some(i as u8) }),
                    Value::Int(i) => rt(c.line, format!("exit code {i} does not fit uint8 (0 to 255)")),
                    o => rt(c.line, format!("std::exit takes a uint8, got {}", o.kind())),
                }
            }
            other => rt(c.line, format!("std::{other} does not exist in the oracle (it knows print.stdout, printu.stdout, print.stderr, printu.stderr, exit)")),
        }
    }
}

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
        b.values.iter().find_map(|v| match v { crate::ast::Value::Expr(e) => in_expr(e), crate::ast::Value::Pieces(ps) => ps.iter().find_map(in_expr) })
    }
    in_bracket(b)
}

fn same_family(a: &Value, b: &Value, line: usize, op: &str) -> Result<(), RuntimeError> {
    let ok = matches!((a, b),
        (Value::Int(_), Value::Int(_)) | (Value::Bin(_), Value::Bin(_)) | (Value::Bool(_), Value::Bool(_))
        | (Value::Str(_), Value::Str(_)) | (Value::Char(..), Value::Char(..)) | (Value::List(_), Value::List(_)));
    if ok { Ok(()) } else { rt(line, format!("`{op}` between {} and {}: no implicit conversion", a.kind(), b.kind())) }
}

/// Grapheme clusters, approximated: a base character plus any combining
/// marks, ZWJ sequences, variation selectors, and skin-tone modifiers. Enough
/// for the oracle to tell "one" from "several"; the real runtime needs UAX #29.
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
