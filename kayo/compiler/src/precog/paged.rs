//! Which lists get page-aligned memory (owner, 2026-10-25; `design/gpu.md`):
//! Metal shares a list with an `inline.MSL` block with no copy only when its
//! memory starts on a page and is whole pages long. Precog gives that memory
//! to the lists it finds can reach a block's binding, and, where it cannot
//! follow a list (into a struct's field, a list of lists), to every list of
//! that list's type. A program with no block gives none.
//!
//! Nothing here decides what a program does: the runtime looks at a bound
//! list's memory when the block runs and copies it when it is not
//! page-aligned. What Precog decides is only which lists are made so.

use std::collections::{HashMap, HashSet};

use crate::ast::Verb;
use crate::mir::*;
use crate::types::{SeqKind, Ty};

#[derive(Default, Debug)]
pub struct Paged {
    /// (function, local): a list that can reach a binding.
    pub locals: HashSet<(usize, LocalId)>,
    /// The list types Precog could not follow somewhere on the way to a
    /// binding: every list of one is given page-aligned memory.
    pub types: Vec<Ty>,
    /// Each block's bound lists, for the report: the function, the local.
    pub bound: Vec<(usize, LocalId)>,
    /// A plain list's name that may hold a narrowed one (owner,
    /// 2026-10-25: a narrowed list goes anywhere a plain one does and
    /// keeps its kind): whether a page-aligned one, a not-page-aligned
    /// one. Where its type is the narrowing, the type says it; this is
    /// where it went by `change`, or back from a call.
    pub kinds: HashMap<(usize, LocalId), (bool, bool)>,
}

impl Paged {
    /// Whether this local's list is given page-aligned memory: `Some(true)`
    /// at every size (`std::list.page_aligned`, the writer's word),
    /// `Some(false)` from a page's size on (Precog's), `None` not
    /// (`std::list.not_page_aligned`, or a list that reaches no block).
    pub fn local(&self, f: usize, l: LocalId, ty: &Ty) -> Option<bool> {
        match (narrowed(ty), self.kinds.get(&(f, l))) {
            (Some(aligned), _) => aligned.then_some(true),
            (None, Some((true, _))) => Some(true),
            (None, Some((false, true))) => None,
            _ => (self.locals.contains(&(f, l)) || (is_list(ty) && self.types.iter().any(|t| t == ty.base()))).then_some(false),
        }
    }

    /// Whether a bound list is always copied: `std::list.not_page_aligned`,
    /// or a name that may hold one ("never" is the writer's word).
    pub fn never(&self, f: usize, l: LocalId, ty: &Ty) -> bool {
        narrowed(ty) == Some(false) || self.kinds.get(&(f, l)).is_some_and(|k| k.1)
    }
}

/// What a list's type says of its memory, if it says.
pub fn narrowed(ty: &Ty) -> Option<bool> {
    let mut t = ty;
    loop {
        match t {
            Ty::Pages { aligned, .. } => return Some(*aligned),
            Ty::Bounded { inner, .. } | Ty::Places { inner, .. } | Ty::Percent(inner) => t = inner,
            _ => return None,
        }
    }
}

/// A list Precog may decide for: one whose type does not say.
fn is_list(t: &Ty) -> bool {
    matches!(t.base(), Ty::Seq { kind: SeqKind::List, .. }) && narrowed(t).is_none()
}

/// What Precog finds for this program.
pub fn decide(p: &Program) -> Paged {
    let mut out = Paged::default();
    out.kinds = kinds(p);
    let live = |f: &Function| !f.dropped;
    // The seeds: every list a block binds.
    for (fi, f) in p.functions.iter().enumerate().filter(|(_, f)| live(f)) {
        for b in &f.blocks {
            for s in &b.stmts {
                if let StmtKind::Asm { msl: Some(_), lends, .. } = &s.kind {
                    for (place, _, _) in lends {
                        let ty = &f.locals[place.local.0 as usize].ty;
                        if place.is_local() && is_list(ty) {
                            out.locals.insert((fi, place.local));
                            out.bound.push((fi, place.local));
                        } else if !place.is_local() {
                            unfollowed(&mut out, &place_ty(p, f, place));
                        }
                    }
                }
            }
        }
    }
    if out.locals.is_empty() && out.types.is_empty() {
        return out;
    }
    // Back along where each one's memory came from, to a fixed point.
    loop {
        let before = (out.locals.len(), out.types.len());
        for (fi, f) in p.functions.iter().enumerate().filter(|(_, f)| live(f)) {
            let reaches = |out: &Paged, l: LocalId| out.locals.contains(&(fi, l));
            for b in &f.blocks {
                for s in &b.stmts {
                    match &s.kind {
                        StmtKind::Assign { place, rvalue } if place.is_local() && reaches(&out, place.local) => match rvalue {
                            Rvalue::Use(Operand::Move(src) | Operand::Read(src)) | Rvalue::Lend { place: src, .. } => from(&mut out, p, fi, f, src),
                            _ => {}
                        },
                        StmtKind::Task { body, captures, .. } => {
                            let g = &p.functions[body.0 as usize];
                            for (k, (verb, place)) in captures.iter().enumerate() {
                                let Some(param) = g.params.get(k) else { continue };
                                let into = out.locals.contains(&(body.0 as usize, *param));
                                if into && *verb != Verb::Copy {
                                    from(&mut out, p, fi, f, place);
                                }
                                if *verb == Verb::LendMut && place.is_local() && reaches(&out, place.local) {
                                    out.locals.insert((body.0 as usize, *param));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                if let Terminator::Call { callee: Callee::Fn(g), args, dest, .. } = &b.term {
                    let gi = g.0 as usize;
                    let callee = &p.functions[gi];
                    // A result that reaches a binding: where the callee
                    // makes it.
                    if let Some(d) = dest {
                        if d.is_local() && reaches(&out, d.local) {
                            if let Some(r) = callee.ret_local {
                                out.locals.insert((gi, r));
                            }
                        }
                    }
                    for (k, arg) in args.iter().enumerate() {
                        let Some(param) = callee.params.get(k) else { continue };
                        let into = out.locals.contains(&(gi, *param));
                        match arg {
                            // The callee binds or keeps what it was handed:
                            // the memory is the caller's.
                            Arg::Give(place) | Arg::Lend { place, .. } | Arg::Value(Operand::Move(place) | Operand::Read(place)) if into => from(&mut out, p, fi, f, place),
                            _ => {}
                        }
                        // A list lent to change that reaches a binding may
                        // be grown in the callee: its growth is paged too.
                        if let Arg::Lend { place, mutable: true } = arg {
                            if place.is_local() && reaches(&out, place.local) {
                                out.locals.insert((gi, *param));
                            }
                        }
                    }
                }
            }
        }
        if (out.locals.len(), out.types.len()) == before {
            break;
        }
    }
    out
}

/// Where narrowed lists go under a plain list's name: moved by `change`
/// or a `let`, or handed back by a call. A kind once there stays there.
fn kinds(p: &Program) -> HashMap<(usize, LocalId), (bool, bool)> {
    let mut out: HashMap<(usize, LocalId), (bool, bool)> = HashMap::new();
    let kind_of = |out: &HashMap<(usize, LocalId), (bool, bool)>, fi: usize, f: &Function, l: LocalId| -> (bool, bool) {
        match narrowed(&f.locals[l.0 as usize].ty) {
            Some(true) => (true, false),
            Some(false) => (false, true),
            None => out.get(&(fi, l)).copied().unwrap_or((false, false)),
        }
    };
    loop {
        let mut changed = false;
        for (fi, f) in p.functions.iter().enumerate().filter(|(_, f)| !f.dropped) {
            let mut flows: Vec<(LocalId, (bool, bool))> = Vec::new();
            for b in &f.blocks {
                for s in &b.stmts {
                    if let StmtKind::Assign { place, rvalue: Rvalue::Use(Operand::Move(src) | Operand::Read(src)) } = &s.kind {
                        if place.is_local() && src.is_local() {
                            flows.push((place.local, kind_of(&out, fi, f, src.local)));
                        }
                    }
                }
                if let Terminator::Call { callee: Callee::Fn(g), dest: Some(d), .. } = &b.term {
                    let gi = g.0 as usize;
                    if let (true, Some(r)) = (d.is_local(), p.functions[gi].ret_local) {
                        flows.push((d.local, kind_of(&out, gi, &p.functions[gi], r)));
                    }
                }
            }
            for (l, (a, n)) in flows {
                if !(a || n) || narrowed(&f.locals[l.0 as usize].ty).is_some() || !matches!(f.locals[l.0 as usize].ty.base(), Ty::Seq { kind: SeqKind::List, .. }) {
                    continue;
                }
                let k = out.entry((fi, l)).or_insert((false, false));
                if (k.0 || !a) && (k.1 || !n) {
                    continue;
                }
                *k = (k.0 || a, k.1 || n);
                changed = true;
            }
        }
        if !changed {
            return out;
        }
    }
}

/// The memory of a list read from `src` reaches a binding.
fn from(out: &mut Paged, p: &Program, fi: usize, f: &Function, src: &Place) {
    if src.is_local() {
        if is_list(&f.locals[src.local.0 as usize].ty) {
            out.locals.insert((fi, src.local));
        }
    } else {
        unfollowed(out, &place_ty(p, f, src));
    }
}

fn unfollowed(out: &mut Paged, ty: &Ty) {
    if is_list(ty) && !out.types.iter().any(|t| t == ty.base()) {
        out.types.push(ty.base().clone());
    }
}

/// A place's type, through its fields and elements.
fn place_ty(p: &Program, f: &Function, place: &Place) -> Ty {
    let mut ty = f.locals[place.local.0 as usize].ty.clone();
    for proj in &place.proj {
        ty = match (proj, ty.base()) {
            (Proj::Index(_), Ty::Seq { elem, .. }) => (**elem).clone(),
            (Proj::Field(i), Ty::Struct { id, .. }) => match p.items.structs[id.0 as usize].fields.get(*i) {
                Some(field) => field.ty.clone(),
                None => return ty,
            },
            _ => return ty,
        };
    }
    ty
}

/// The report's notes (owner, 2026-10-25: "the report notes which lists it
/// made page-aligned"): each named list and each type so given.
pub fn notes(p: &Program, sources: &crate::source::SourceMap) -> Vec<String> {
    let d = decide(p);
    let spell = |t: &Ty| crate::types::TySpelling { ty: t, structs: &p.items.structs, enums: &p.items.enums }.to_string();
    let mut out = Vec::new();
    let mut seen: Vec<(usize, LocalId)> = d.locals.iter().copied().collect();
    seen.sort_by_key(|(f, l)| (*f, l.0));
    for (fi, l) in seen {
        let f = &p.functions[fi];
        let local = &f.locals[l.0 as usize];
        let Some(name) = &local.name else { continue };
        let why = if d.bound.contains(&(fi, l)) { "a GPU block binds it" } else { "its memory reaches a GPU block's binding" };
        let note = crate::diag::Diagnostic::note(format!("`${}` is given page-aligned memory once it holds a page (16 KB here) or more: {}, and Metal shares page-aligned memory with no copy", name, why));
        out.push(note.at(f.span(local.span)).render(sources));
    }
    for t in &d.types {
        out.push(crate::diag::Diagnostic::note(format!("every `{}` is given page-aligned memory once it holds a page or more: one reaches a GPU block's binding through a field or an element, where Precog does not follow it", spell(t))).render(sources));
    }
    out
}
