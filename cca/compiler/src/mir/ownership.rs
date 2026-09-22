//! Where a value goes, and when it is gone.
//!
//! The checker settled what each verb *means*; this is the flow question
//! it cannot answer from one statement: a value given away is gone, and
//! using it afterwards is the error `ownership-syntax.md` describes —
//! "ownership transfers; `'t'` is dead after this". Because it is a flow
//! question it belongs on the graph, where a `give` inside a loop is
//! visibly a `give` on every pass but the first.
//!
//! The same walk elaborates the drops. MIR emits a `Drop` for every owned
//! local when its scope ends, because the scope is where the decision
//! belongs; the ones whose value has already been handed away are removed
//! here, which is why `return give 'best';` does not free what it just
//! returned.

use std::collections::HashMap;

use crate::diag::Diagnostic;
use crate::source::SourceMap;

use super::*;

/// What is known about a local at a point in the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// It holds its value.
    Here,
    /// It was given away on every path to here.
    Gone,
    /// Given away on some paths and not others — a `give` in a loop, or in
    /// one arm of an `if`.
    Maybe,
}

impl State {
    fn join(self, other: State) -> State {
        if self == other {
            self
        } else {
            State::Maybe
        }
    }
}

type Env = HashMap<LocalId, State>;

pub struct Outcome {
    pub problems: Vec<String>,
    /// Drops removed because the value had already been handed away.
    pub drops_removed: usize,
    /// Drops that survive on some paths only: they need a flag at run time
    /// to say whether the value is still there.
    pub conditional_drops: usize,
}

pub fn check(p: &mut Program, sources: &SourceMap) -> Outcome {
    let mut out = Outcome { problems: Vec::new(), drops_removed: 0, conditional_drops: 0 };
    for i in 0..p.functions.len() {
        walk(p, i, sources, &mut out);
    }
    out
}

fn walk(p: &mut Program, f: usize, sources: &SourceMap, out: &mut Outcome) {
    let n = p.functions[f].blocks.len();
    let mut entry: Vec<Option<Env>> = vec![None; n];
    entry[0] = Some(Env::new());
    let mut work = vec![0usize];
    // Where each local was given away, for the message that says so.
    let mut given_at: HashMap<LocalId, (SpanId, bool)> = HashMap::new();
    let mut reported: Vec<(LocalId, SpanId)> = Vec::new();
    let mut rounds = 0usize;

    while let Some(b) = work.pop() {
        rounds += 1;
        if rounds > n * 64 {
            break;
        }
        let Some(mut env) = entry[b].clone() else { continue };
        let stmts = p.functions[f].blocks[b].stmts.clone();
        for (si, s) in stmts.iter().enumerate() {
            match &s.kind {
                StmtKind::Assign { place, rvalue } => {
                    for (pl, how) in rvalue_uses(rvalue) {
                        use_place(p, f, &env, &pl, how, s.span, &given_at, &mut reported, out, sources);
                    }
                    for pl in rvalue_moves(rvalue) {
                        mark_gone(&mut env, &pl, s.span, &mut given_at);
                    }
                    if place.proj.is_empty() {
                        env.insert(place.local, State::Here);
                    }
                }
                StmtKind::Op { dest, args, .. } => {
                    for a in args {
                        if let Some(pl) = a.place() {
                            let how = if matches!(a, Operand::Move(_)) { Use::Give } else { Use::Read };
                            use_place(p, f, &env, pl, how, s.span, &given_at, &mut reported, out, sources);
                        }
                        if let Operand::Move(pl) = a {
                            mark_gone(&mut env, pl, s.span, &mut given_at);
                        }
                    }
                    if let Some(d) = dest {
                        if d.proj.is_empty() {
                            env.insert(d.local, State::Here);
                        }
                    }
                }
                StmtKind::Drop(pl) => {
                    let state = state_of(&env, pl.local);
                    match state {
                        State::Gone => {
                            p.functions[f].blocks[b].stmts[si].kind = StmtKind::Nop;
                            out.drops_removed += 1;
                        }
                        State::Maybe => out.conditional_drops += 1,
                        State::Here => {}
                    }
                }
                StmtKind::Task { captures, .. } => {
                    for (verb, pl) in captures {
                        let how = match verb {
                            crate::ast::Verb::Give => Use::Give,
                            _ => Use::Read,
                        };
                        use_place(p, f, &env, pl, how, s.span, &given_at, &mut reported, out, sources);
                        if *verb == crate::ast::Verb::Give {
                            mark_gone(&mut env, pl, s.span, &mut given_at);
                        }
                    }
                }
                StmtKind::EndLend(_) | StmtKind::Site(_) | StmtKind::Asm { .. } | StmtKind::Nop => {}
            }
        }
        let term = p.functions[f].blocks[b].term.clone();
        let tspan = p.functions[f].blocks[b].term_span;
        match &term {
            Terminator::Call { callee, args, dest, .. } => {
                if let Callee::Value(o) = callee {
                    if let Some(pl) = o.place() {
                        use_place(p, f, &env, pl, Use::Read, tspan, &given_at, &mut reported, out, sources);
                    }
                }
                for a in args {
                    match a {
                        Arg::Give(pl) => {
                            use_place(p, f, &env, pl, Use::Give, tspan, &given_at, &mut reported, out, sources);
                            mark_gone(&mut env, pl, tspan, &mut given_at);
                        }
                        Arg::Value(Operand::Move(pl)) => {
                            use_place(p, f, &env, pl, Use::Give, tspan, &given_at, &mut reported, out, sources);
                            mark_gone(&mut env, pl, tspan, &mut given_at);
                        }
                        Arg::Lend { place, .. } | Arg::Copy(place) | Arg::Value(Operand::Read(place)) => {
                            use_place(p, f, &env, place, Use::Read, tspan, &given_at, &mut reported, out, sources);
                        }
                        // `...give 'xs'` empties the list; it does not take
                        // it away (`functions.md`: a give never leaves holes).
                        Arg::Spread { place, .. } => {
                            use_place(p, f, &env, place, Use::Read, tspan, &given_at, &mut reported, out, sources);
                        }
                        Arg::Value(Operand::Const(_)) => {}
                    }
                }
                if let Some(d) = dest {
                    if d.proj.is_empty() {
                        env.insert(d.local, State::Here);
                    }
                }
            }
            Terminator::Fail { payload: Some(o), .. } => {
                if let Operand::Move(pl) = o {
                    mark_gone(&mut env, pl, tspan, &mut given_at);
                }
            }
            _ => {}
        }
        for t in crate::mir::verify::successors(&term) {
            let i = t.0 as usize;
            if i >= n {
                continue;
            }
            let merged = match &entry[i] {
                None => env.clone(),
                Some(old) => {
                    let m = join(old, &env);
                    if m == *old {
                        continue;
                    }
                    m
                }
            };
            entry[i] = Some(merged);
            if !work.contains(&i) {
                work.push(i);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Use {
    Read,
    Give,
}

fn state_of(env: &Env, l: LocalId) -> State {
    env.get(&l).copied().unwrap_or(State::Here)
}

/// A `give` of a whole value, or of a field — which consumes the whole
/// variable (`structs.md`: "there is no half-empty struct"). Taking an
/// element out of a list is the lowering's own business and takes nothing
/// away: `give 'xs'[pos i]` is refused at the source, and a `for … in give
/// 'xs'` consumes the list as a whole.
fn moves_base(pl: &Place) -> bool {
    !pl.proj.iter().any(|p| matches!(p, Proj::Index(_) | Proj::BundleElem(_)))
}

fn mark_gone(env: &mut Env, pl: &Place, span: SpanId, given_at: &mut HashMap<LocalId, (SpanId, bool)>) {
    if moves_base(pl) {
        env.insert(pl.local, State::Gone);
        given_at.entry(pl.local).or_insert((span, !pl.proj.is_empty()));
    }
}

#[allow(clippy::too_many_arguments)]
fn use_place(p: &Program, f: usize, env: &Env, pl: &Place, how: Use, span: SpanId, given_at: &HashMap<LocalId, (SpanId, bool)>, reported: &mut Vec<(LocalId, SpanId)>, out: &mut Outcome, sources: &SourceMap) {
    let func = &p.functions[f];
    let state = state_of(env, pl.local);
    if state == State::Here {
        return;
    }
    // The compiler's own temporaries are not the writer's problem: a
    // message about one would name something nobody wrote.
    if func.locals[pl.local.0 as usize].name.is_none() {
        return;
    }
    if reported.iter().any(|(l, s)| *l == pl.local && *s == span) {
        return;
    }
    reported.push((pl.local, span));
    // What is gone is the whole value, even when a field is what is being
    // read: giving a field away consumes the variable (`structs.md`).
    let whole = format!("'{}'", func.locals[pl.local.0 as usize].name.clone().unwrap_or_default());
    let name = func.place_name(pl, &p.items.structs);
    let what = match (how, state) {
        (Use::Give, State::Gone) => format!("{} was already given away, so there is nothing left to give", whole),
        (Use::Give, _) => format!("{} may already have been given away, so there may be nothing left to give", whole),
        (_, State::Gone) => format!("{} was given away and no longer owns anything", whole),
        (_, _) => format!("{} may already have been given away", whole),
    };
    let mut d = Diagnostic::error(what).at(func.span(span));
    if func.name != "MAIN" {
        d = d.place(format!("in `{}`", func.name));
    }
    if let Some((at, via_field)) = given_at.get(&pl.local) {
        // Not when it is the same line the message already points at.
        if *at != span {
            d = d.why_at(func.span(*at), "it was given away here");
        }
        if *via_field {
            d = d.why("giving a field away gives away the whole value: there is no half-empty struct");
        }
    }
    if name != whole {
        d = d.why(format!("{} is part of it", name));
    }
    if state == State::Maybe {
        let looped = given_at.get(&pl.local).map(|(at, _)| *at == span).unwrap_or(false);
        if looped {
            d = d.why("the first pass gives it away, and there is nothing left for the next one");
        } else {
            d = d.why("one way through this code gives it away and another does not, so by here it depends on which was taken");
        }
    }
    let target = if name == whole { name } else { whole };
    d = d
        .fix(None, format!("copy {}   # a second, independent value", target))
        .fix(None, format!("lend {}   # a look at it, without taking it", target));
    out.problems.push(d.render(sources));
}

fn join(a: &Env, b: &Env) -> Env {
    let mut out = a.clone();
    for (k, v) in b {
        let merged = match a.get(k) {
            None => State::Here.join(*v),
            Some(old) => old.join(*v),
        };
        out.insert(*k, merged);
    }
    for (k, v) in a {
        if !b.contains_key(k) {
            out.insert(*k, v.join(State::Here));
        }
    }
    out
}

/// Every place an rvalue reads, and how.
fn rvalue_uses(r: &Rvalue) -> Vec<(Place, Use)> {
    let read = |o: &Operand| -> Vec<(Place, Use)> {
        match o {
            Operand::Read(p) => vec![(p.clone(), Use::Read)],
            Operand::Move(p) => vec![(p.clone(), Use::Give)],
            Operand::Const(_) => Vec::new(),
        }
    };
    match r {
        Rvalue::Use(o) | Rvalue::Unary { operand: o, .. } | Rvalue::Convert { operand: o, .. } => read(o),
        Rvalue::Binary { lhs, rhs, .. } => {
            let mut v = read(lhs);
            v.extend(read(rhs));
            v
        }
        Rvalue::Duplicate(p) | Rvalue::Lend { place: p, .. } => vec![(p.clone(), Use::Read)],
        Rvalue::MakeList(v) => v.iter().flat_map(read).collect(),
        Rvalue::MakeMap(v) => v.iter().flat_map(|(k, x)| read(k).into_iter().chain(read(x))).collect(),
        Rvalue::MakeStruct { fields, .. } => fields.iter().flat_map(read).collect(),
        Rvalue::MakeEnum { payload, .. } => payload.iter().flat_map(read).collect(),
        Rvalue::Closure { captures, .. } => captures
            .iter()
            .map(|(v, p)| (p.clone(), if *v == crate::ast::Verb::Give { Use::Give } else { Use::Read }))
            .collect(),
        Rvalue::MakeChannel { .. } | Rvalue::FnRef(_) | Rvalue::ExternRead(_) => Vec::new(),
    }
}

/// Every place an rvalue takes away.
fn rvalue_moves(r: &Rvalue) -> Vec<Place> {
    rvalue_uses(r).into_iter().filter(|(_, h)| *h == Use::Give).map(|(p, _)| p).collect()
}
