//! HIR to MIR: the flow graph, with every check site, drop, lend and
//! failure edge written out as an instruction.

use std::collections::HashMap;

use crate::ast::{BinOp, Tier, UnOp, Verb};
use crate::check::{Instance, Program as Checked};
use crate::hir::{self, Builtin, FnInst};
use crate::source::Span;
use crate::types::{Bound, IntW, ParamKind, Ty};

use super::domain::Domain;
use super::*;

pub fn build(checked: Checked) -> Program {
    let mut b = Arena { functions: Vec::new(), inst_idx: HashMap::new(), instances: checked.instances, items: checked.items };
    // Every instance with a body gets its index first, so a call can name
    // one that has not been lowered yet.
    let mut order = Vec::new();
    for i in 0..b.instances.len() {
        if b.instances[i].body.is_some() {
            let idx = FnIdx(b.functions.len() as u32);
            b.inst_idx.insert(FnInst(i as u32), idx);
            // The slot is taken now: a task body lowered while this one is
            // being built appends after it, and every index stays what a
            // call already wrote down.
            let body = b.instances[i].body.clone().unwrap();
            b.functions.push(placeholder(&body));
            order.push((FnInst(i as u32), idx));
        }
    }
    for (inst, idx) in &order {
        let f = lower_instance(&mut b, *inst);
        b.functions[idx.0 as usize] = f;
    }
    let main = checked.main.and_then(|m| b.inst_idx.get(&m).copied());
    Program { functions: b.functions, main, items: b.items, instances: b.instances }
}

pub struct Arena {
    pub functions: Vec<Function>,
    inst_idx: HashMap<FnInst, FnIdx>,
    pub instances: Vec<Instance>,
    pub items: crate::resolve::Items,
}

fn lower_instance(arena: &mut Arena, inst: FnInst) -> Function {
    let body = arena.instances[inst.0 as usize].body.clone().unwrap();
    let mut fb = FnBuilder::new(arena, Some(inst), &body, false);
    fb.lower_body(&body);
    fb.finish(&body)
}

/// One function being built.
struct FnBuilder<'a> {
    arena: &'a mut Arena,
    inst: Option<FnInst>,
    locals: Vec<LocalDecl>,
    blocks: Vec<BasicBlock>,
    sites: Vec<Site>,
    spans: Vec<Span>,
    span_index: HashMap<(u32, u32, u32), SpanId>,
    cur: BlockId,
    /// Scopes of owned locals, innermost last; a scope's end drops them.
    scopes: Vec<Vec<LocalId>>,
    /// `break` and `continue` targets, with the scope depth to drop to.
    loops: Vec<LoopTargets>,
    /// The tier covering the statement being lowered.
    tier: Option<Tier>,
    /// The blocks a `concur` started in this scope; joined at its end.
    tasks_in_scope: Vec<usize>,
    is_task: bool,
    /// A task body's captures, as locals.
    captures: Vec<LocalId>,
    /// The checker's local table: a task's body shares its parent's.
    hir_locals: Vec<hir::Local>,
    /// Per scope: where a task started in it lands when it fails.
    task_fails: Vec<Vec<FailArm>>,
    /// Per scope: the block each task handler falls out of. It cannot know
    /// where to go until the wait it belongs to exists, so it is patched
    /// when the block ends.
    task_after: Vec<Vec<BlockId>>,
    /// The slot a `return` fills.
    ret_local: Option<LocalId>,
}

#[derive(Clone, Copy)]
struct LoopTargets {
    brk: BlockId,
    cont: BlockId,
    depth: usize,
}

impl<'a> FnBuilder<'a> {
    fn new(arena: &'a mut Arena, inst: Option<FnInst>, body: &hir::Body, is_task: bool) -> FnBuilder<'a> {
        let mut fb = FnBuilder {
            arena,
            inst,
            locals: Vec::new(),
            blocks: vec![BasicBlock { stmts: Vec::new(), term: Terminator::Unreachable }],
            sites: Vec::new(),
            spans: Vec::new(),
            span_index: HashMap::new(),
            cur: BlockId(0),
            scopes: vec![Vec::new()],
            loops: Vec::new(),
            tier: None,
            tasks_in_scope: vec![0],
            is_task,
            captures: Vec::new(),
            hir_locals: body.locals.clone(),
            task_fails: vec![Vec::new()],
            task_after: vec![Vec::new()],
            ret_local: None,
        };
        // One MIR local per HIR local, in the same order, so indices line up.
        for l in &body.locals {
            let span = fb.span_id(l.span);
            let kind = match l.kind {
                hir::LocalKind::Lent => LocalKind::Lent { mutable: false },
                hir::LocalKind::LentMut => LocalKind::Lent { mutable: true },
                _ if l.ty.is_owned() => LocalKind::Owned,
                _ => LocalKind::Plain,
            };
            fb.locals.push(LocalDecl { name: Some(l.name.clone()), ty: l.ty.clone(), domain: Domain::of_ty(&l.ty), kind, span });
        }
        if body.ret != Ty::Nothing {
            let span = fb.span_id(body.span);
            let id = LocalId(fb.locals.len() as u32);
            fb.locals.push(LocalDecl { name: Some("the result".into()), ty: body.ret.clone(), domain: Domain::of_ty(&body.ret), kind: LocalKind::Return, span });
            fb.ret_local = Some(id);
        }
        fb
    }

    // ------------------------------------------------------------ plumbing

    fn span_id(&mut self, span: Span) -> SpanId {
        let key = (span.file.0, span.start, span.end);
        if let Some(id) = self.span_index.get(&key) {
            return *id;
        }
        let id = SpanId(self.spans.len() as u32);
        self.spans.push(span);
        self.span_index.insert(key, id);
        id
    }

    fn new_block(&mut self) -> BlockId {
        let id = BlockId(self.blocks.len() as u32);
        self.blocks.push(BasicBlock { stmts: Vec::new(), term: Terminator::Unreachable });
        id
    }

    fn push(&mut self, kind: StmtKind, span: SpanId) {
        self.blocks[self.cur.0 as usize].stmts.push(Statement { kind, span });
    }

    fn terminate(&mut self, term: Terminator) {
        let b = &mut self.blocks[self.cur.0 as usize];
        if matches!(b.term, Terminator::Unreachable) {
            b.term = term;
        }
    }

    /// Has the current block already left?
    fn closed(&self) -> bool {
        !matches!(self.blocks[self.cur.0 as usize].term, Terminator::Unreachable)
    }

    fn temp(&mut self, ty: &Ty, span: SpanId) -> LocalId {
        let kind = if ty.is_owned() { LocalKind::Owned } else { LocalKind::Temp };
        let id = LocalId(self.locals.len() as u32);
        self.locals.push(LocalDecl { name: None, ty: ty.clone(), domain: Domain::of_ty(ty), kind, span });
        // A temporary holding an owned value is dropped with its scope.
        if kind == LocalKind::Owned {
            self.scopes.last_mut().unwrap().push(id);
        }
        id
    }

    fn site(&mut self, kind: SiteKind, operands: Vec<Operand>, span: SpanId) -> SiteId {
        let id = SiteId(self.sites.len() as u32);
        self.sites.push(Site { kind, tier: self.tier, span, operands, verdict: Verdict::Unproven });
        self.push(StmtKind::Site(id), span);
        id
    }

    fn assign(&mut self, place: Place, rvalue: Rvalue, span: SpanId) {
        self.push(StmtKind::Assign { place, rvalue }, span);
    }

    fn into_temp(&mut self, rvalue: Rvalue, ty: &Ty, span: SpanId) -> Operand {
        let t = self.temp(ty, span);
        // A temporary holding a borrow is a view, not a value to free: its
        // scope ends the lend instead of dropping it.
        if let Rvalue::Lend { mutable, .. } = &rvalue {
            self.locals[t.0 as usize].kind = LocalKind::Lent { mutable: *mutable };
        }
        self.assign(Place::local(t), rvalue, span);
        Operand::Read(Place::local(t))
    }

    fn push_scope(&mut self) {
        self.scopes.push(Vec::new());
        self.tasks_in_scope.push(0);
        self.task_fails.push(Vec::new());
        self.task_after.push(Vec::new());
    }

    /// Drop this scope's owned values, newest first, and wait for its tasks.
    fn pop_scope(&mut self, span: SpanId) {
        let tasks = self.tasks_in_scope.pop().unwrap_or(0);
        let fails = self.task_fails.pop().unwrap_or_default();
        let after = self.task_after.pop().unwrap_or_default();
        let locals = self.scopes.pop().unwrap_or_default();
        if !self.closed() {
            if tasks > 0 {
                // The block waits for what it started; a task's failure is an
                // edge out of the wait, and its handler rejoins here.
                let ok = self.new_block();
                self.terminate(Terminator::Join { ok, fail: fails });
                for b in after {
                    self.blocks[b.0 as usize].term = Terminator::Goto(ok);
                }
                self.cur = ok;
            }
            self.drop_locals(&locals, span);
        }
    }

    fn drop_locals(&mut self, locals: &[LocalId], span: SpanId) {
        for l in locals.iter().rev() {
            match self.locals[l.0 as usize].kind {
                LocalKind::Owned => self.push(StmtKind::Drop(Place::local(*l)), span),
                LocalKind::Lent { .. } => self.push(StmtKind::EndLend(Place::local(*l)), span),
                _ => {}
            }
        }
    }

    /// Leaving through `return` / `fail` / `break` / `continue`: every scope
    /// down to `depth` ends here too.
    fn unwind_to(&mut self, depth: usize, span: SpanId) {
        let mut i = self.scopes.len();
        while i > depth {
            i -= 1;
            if self.tasks_in_scope.get(i).copied().unwrap_or(0) > 0 {
                let ok = self.new_block();
                let fails = self.task_fails.get(i).cloned().unwrap_or_default();
                self.terminate(Terminator::Join { ok, fail: fails });
                for b in self.task_after.get(i).cloned().unwrap_or_default() {
                    self.blocks[b.0 as usize].term = Terminator::Goto(ok);
                }
                self.cur = ok;
            }
            let locals = self.scopes[i].clone();
            self.drop_locals(&locals, span);
        }
    }

    fn finish(mut self, body: &hir::Body) -> Function {
        let span = self.span_id(body.span);
        if !self.closed() {
            self.unwind_to(0, span);
            self.terminate(Terminator::Return);
        }
        // Any block left without a terminator fell out of a construct that
        // always leaves; nothing reaches it.
        for b in &mut self.blocks {
            if matches!(b.term, Terminator::Unreachable) && b.stmts.is_empty() {
                b.term = Terminator::Unreachable;
            }
        }
        let params = body.params.iter().map(|p| LocalId(p.0)).collect();
        Function {
            name: body.name.clone(),
            inst: self.inst,
            params,
            locals: self.locals,
            blocks: self.blocks,
            ret: body.ret.clone(),
            ret_local: self.ret_local,
            fails: body.fails.clone(),
            sites: self.sites,
            spans: self.spans,
            is_main: body.is_main,
            is_task: self.is_task,
            captures: self.captures,
        }
    }

    // ------------------------------------------------------------ statements

    fn lower_body(&mut self, body: &hir::Body) {
        self.block(&body.block);
    }

    fn block(&mut self, b: &hir::Block) {
        self.push_scope();
        for s in &b.stmts {
            if self.closed() {
                break;
            }
            self.stmt(s);
        }
        let span = self.span_id(b.span);
        self.pop_scope(span);
    }

    fn stmt(&mut self, s: &hir::Stmt) {
        let span = self.span_id(s.span);
        let saved_tier = self.tier;
        if s.tier.is_some() {
            self.tier = s.tier;
        }
        self.stmt_kind(s, span);
        self.tier = saved_tier;
    }

    fn stmt_kind(&mut self, s: &hir::Stmt, span: SpanId) {
        match &s.kind {
            hir::StmtKind::Let { local, value, typed } => {
                let place = Place::local(LocalId(local.0));
                let ty = self.locals[local.0 as usize].ty.clone();
                self.store(place.clone(), value, &ty, *typed, span);
                // An owned local is this scope's to drop.
                match self.locals[local.0 as usize].kind {
                    LocalKind::Owned | LocalKind::Lent { .. } => self.scopes.last_mut().unwrap().push(LocalId(local.0)),
                    _ => {}
                }
            }
            hir::StmtKind::Assign { place, value } => {
                let p = self.place(place, span);
                let ty = place.ty.clone();
                self.store(p, value, &ty, true, span);
            }
            hir::StmtKind::Expr(e) => {
                let _ = self.eval(e, span);
            }
            hir::StmtKind::If { cond, then, else_ } => {
                let c = self.eval(cond, span);
                let then_b = self.new_block();
                let else_b = self.new_block();
                let join = self.new_block();
                self.terminate(Terminator::If { cond: c, then: then_b, else_: else_b });
                self.cur = then_b;
                self.block(then);
                self.terminate(Terminator::Goto(join));
                self.cur = else_b;
                match else_ {
                    Some(e) => self.block(e),
                    None => {}
                }
                self.terminate(Terminator::Goto(join));
                self.cur = join;
            }
            hir::StmtKind::Loop(body) => {
                let head = self.new_block();
                let exit = self.new_block();
                self.terminate(Terminator::Goto(head));
                self.cur = head;
                self.loops.push(LoopTargets { brk: exit, cont: head, depth: self.scopes.len() });
                self.block(body);
                self.terminate(Terminator::Goto(head));
                self.loops.pop();
                self.cur = exit;
            }
            hir::StmtKind::While { cond, body } => {
                let head = self.new_block();
                let body_b = self.new_block();
                let exit = self.new_block();
                self.terminate(Terminator::Goto(head));
                self.cur = head;
                let c = self.eval(cond, span);
                self.terminate(Terminator::If { cond: c, then: body_b, else_: exit });
                self.cur = body_b;
                self.loops.push(LoopTargets { brk: exit, cont: head, depth: self.scopes.len() });
                self.block(body);
                self.terminate(Terminator::Goto(head));
                self.loops.pop();
                self.cur = exit;
            }
            hir::StmtKind::ForRange { counter, start, stop, step, body, perm, seq } => {
                self.for_range(*counter, start, stop, step.as_ref(), body, *perm, *seq, span);
            }
            hir::StmtKind::ForEach { elem, iter, body, perm, seq } => {
                self.for_each(*elem, iter, body, *perm, *seq, span);
            }
            hir::StmtKind::Return(v) => {
                match v {
                    Some(e) => {
                        let ret_ty = self.ret_ty();
                        let ret = self.ret_local.unwrap_or_else(|| self.temp(&ret_ty, span));
                        self.store(Place::local(ret), e, &ret_ty, true, span);
                        self.unwind_to(0, span);
                        self.terminate(Terminator::Return);
                    }
                    None => {
                        self.unwind_to(0, span);
                        self.terminate(Terminator::Return);
                    }
                }
            }
            hir::StmtKind::Fail { index, payload, .. } => {
                let p = payload.as_ref().map(|e| self.eval(e, span));
                self.unwind_to(0, span);
                self.terminate(Terminator::Fail { index: *index, payload: p });
            }
            hir::StmtKind::Break => {
                if let Some(t) = self.loops.last().copied() {
                    self.unwind_to(t.depth, span);
                    self.terminate(Terminator::Goto(t.brk));
                }
            }
            hir::StmtKind::Continue => {
                if let Some(t) = self.loops.last().copied() {
                    self.unwind_to(t.depth, span);
                    self.terminate(Terminator::Goto(t.cont));
                }
            }
            hir::StmtKind::Block(b) => self.block(b),
            hir::StmtKind::Match { value, arms } => {
                let v = self.eval(value, span);
                let join = self.new_block();
                let mut cases = Vec::new();
                for arm in arms {
                    let b = self.new_block();
                    cases.push((arm.case, b));
                }
                self.terminate(Terminator::Switch { value: v.clone(), cases: cases.clone() });
                for (arm, (_, b)) in arms.iter().zip(&cases) {
                    self.cur = *b;
                    self.push_scope();
                    if let (Some(bind), Some(place)) = (arm.bind, v.place()) {
                        let ty = self.locals[bind.0 as usize].ty.clone();
                        let mut src = place.clone();
                        src.proj.push(Proj::Field(arm.case));
                        let rv = match self.locals[bind.0 as usize].kind {
                            LocalKind::Lent { mutable } => Rvalue::Lend { place: src, mutable },
                            _ => Rvalue::Use(Operand::Read(src)),
                        };
                        self.assign(Place::local(LocalId(bind.0)), rv, span);
                        let _ = ty;
                        self.scopes.last_mut().unwrap().push(LocalId(bind.0));
                    }
                    for st in &arm.body.stmts {
                        if self.closed() {
                            break;
                        }
                        self.stmt(st);
                    }
                    self.pop_scope(span);
                    self.terminate(Terminator::Goto(join));
                }
                self.cur = join;
            }
            hir::StmtKind::Concur { detached, captures, body, otherwise, fails } => {
                self.concur(*detached, captures, body, otherwise.as_ref(), fails, span);
            }
            hir::StmtKind::Whichever { arms, otherwise } => {
                self.whichever(arms, otherwise.as_ref(), span);
            }
            hir::StmtKind::InlineAsm { text, ins, outs, clobbers } => {
                let ins = ins.iter().map(|(l, r)| (Place::local(LocalId(l.0)), r.clone())).collect();
                let outs = outs.iter().map(|(l, r)| (Place::local(LocalId(l.0)), r.clone())).collect();
                self.push(StmtKind::Asm { text: text.clone(), ins, outs, clobbers: clobbers.clone() }, span);
            }
        }
    }

    fn ret_ty(&self) -> Ty {
        match self.inst {
            Some(i) => self.arena.instances[i.0 as usize].sig.ret.clone(),
            None => Ty::Nothing,
        }
    }

    /// `for 'i' in range[a, b, s]` — the counter moves by the step and the
    /// body cannot assign to it, so the header alone says the trip count.
    #[allow(clippy::too_many_arguments)]
    fn for_range(&mut self, counter: hir::LocalId, start: &hir::Expr, stop: &hir::Expr, step: Option<&hir::Expr>, body: &hir::Block, perm: bool, _seq: bool, span: SpanId) {
        let c = LocalId(counter.0);
        let ty = self.locals[c.0 as usize].ty.clone();
        let s = self.eval(start, span);
        self.assign(Place::local(c), Rvalue::Use(s), span);
        let stop_v = self.eval(stop, span);
        let stop_t = self.temp(&ty, span);
        self.assign(Place::local(stop_t), Rvalue::Use(stop_v), span);
        let step_v = match step {
            Some(e) => self.eval(e, span),
            None => Operand::Const(Const::Int(1)),
        };
        let step_t = self.temp(&ty, span);
        self.assign(Place::local(step_t), Rvalue::Use(step_v.clone()), span);

        let head = self.new_block();
        let body_b = self.new_block();
        let next = self.new_block();
        let exit = self.new_block();
        self.terminate(Terminator::Goto(head));

        // The stop is inclusive, and a negative step counts down.
        self.cur = head;
        let going_up = self.into_temp(
            Rvalue::Binary { op: BinOp::Ge, lhs: Operand::Read(Place::local(step_t)), rhs: Operand::Const(Const::Int(0)) },
            &Ty::Bool,
            span,
        );
        let up_b = self.new_block();
        let down_b = self.new_block();
        self.terminate(Terminator::If { cond: going_up, then: up_b, else_: down_b });
        self.cur = up_b;
        let le = self.into_temp(
            Rvalue::Binary { op: BinOp::Le, lhs: Operand::Read(Place::local(c)), rhs: Operand::Read(Place::local(stop_t)) },
            &Ty::Bool,
            span,
        );
        self.terminate(Terminator::If { cond: le, then: body_b, else_: exit });
        self.cur = down_b;
        let ge = self.into_temp(
            Rvalue::Binary { op: BinOp::Ge, lhs: Operand::Read(Place::local(c)), rhs: Operand::Read(Place::local(stop_t)) },
            &Ty::Bool,
            span,
        );
        self.terminate(Terminator::If { cond: ge, then: body_b, else_: exit });

        self.cur = body_b;
        self.loops.push(LoopTargets { brk: exit, cont: next, depth: self.scopes.len() });
        self.block(body);
        self.terminate(Terminator::Goto(next));
        self.loops.pop();

        // The step is taken only when there is another trip to make. The
        // counter must hold every value it takes, and the *stop* is one of
        // them: `for 'b': int8 in range[126, 127]` ends at 127 without ever
        // computing 128, so an inclusive loop over a type's last value is
        // not an overflow.
        self.cur = next;
        let up2 = self.into_temp(
            Rvalue::Binary { op: BinOp::Ge, lhs: Operand::Read(Place::local(step_t)), rhs: Operand::Const(Const::Int(0)) },
            &Ty::Bool,
            span,
        );
        let at_end_up = self.new_block();
        let at_end_down = self.new_block();
        let step_b = self.new_block();
        self.terminate(Terminator::If { cond: up2, then: at_end_up, else_: at_end_down });
        self.cur = at_end_up;
        let done_up = self.into_temp(
            Rvalue::Binary { op: BinOp::Ge, lhs: Operand::Read(Place::local(c)), rhs: Operand::Read(Place::local(stop_t)) },
            &Ty::Bool,
            span,
        );
        self.terminate(Terminator::If { cond: done_up, then: exit, else_: step_b });
        self.cur = at_end_down;
        let done_down = self.into_temp(
            Rvalue::Binary { op: BinOp::Le, lhs: Operand::Read(Place::local(c)), rhs: Operand::Read(Place::local(stop_t)) },
            &Ty::Bool,
            span,
        );
        self.terminate(Terminator::If { cond: done_down, then: exit, else_: step_b });
        self.cur = step_b;
        // A step that lands past the type's own end is still a site: with a
        // step of 3 an `int8` counter can reach 129, which is nobody's fault
        // but the program's.
        self.site(SiteKind::Overflow(BinOp::Add), vec![Operand::Read(Place::local(c)), Operand::Read(Place::local(step_t))], span);
        self.assign(
            Place::local(c),
            Rvalue::Binary { op: BinOp::Add, lhs: Operand::Read(Place::local(c)), rhs: Operand::Read(Place::local(step_t)) },
            span,
        );
        self.terminate(Terminator::Goto(head));

        self.cur = exit;
        if perm {
            // The counter outlives the loop, holding what the loop last gave it.
            self.scopes.last_mut().unwrap().push(c);
        }
    }

    /// A `for` over a list, a text walk or a map walk: by position, so the
    /// element's own bounds check is the one the loop already proves.
    fn for_each(&mut self, elem: hir::LocalId, iter: &hir::Expr, body: &hir::Block, perm: bool, _seq: bool, span: SpanId) {
        let e = LocalId(elem.0);
        let iter_op = self.eval(iter, span);
        let iter_ty = self.resolve_ty(&iter.ty);
        let seq = self.temp(&iter_ty, span);
        self.locals[seq.0 as usize].kind = match &iter_op {
            Operand::Move(_) => LocalKind::Owned,
            _ => LocalKind::Lent { mutable: false },
        };
        self.assign(Place::local(seq), Rvalue::Use(iter_op), span);

        let u64_ty = Ty::u64();
        let len = self.temp(&u64_ty, span);
        let length_op = match iter_ty.base() {
            Ty::Seq { kind: crate::types::SeqKind::Array, .. } => Op::ArrayLength,
            _ => Op::ListLength,
        };
        self.push(StmtKind::Op { dest: Some(Place::local(len)), op: length_op, args: vec![Operand::Read(Place::local(seq))] }, span);

        let i = self.temp(&u64_ty, span);
        self.assign(Place::local(i), Rvalue::Use(Operand::Const(Const::Int(1))), span);

        let head = self.new_block();
        let body_b = self.new_block();
        let next = self.new_block();
        let exit = self.new_block();
        self.terminate(Terminator::Goto(head));

        self.cur = head;
        let cmp = self.into_temp(
            Rvalue::Binary { op: BinOp::Le, lhs: Operand::Read(Place::local(i)), rhs: Operand::Read(Place::local(len)) },
            &Ty::Bool,
            span,
        );
        self.terminate(Terminator::If { cond: cmp, then: body_b, else_: exit });

        self.cur = body_b;
        // The index is the loop's own, between 1 and the length: the site is
        // here so Precog has something to prove rather than nothing to see.
        self.site(SiteKind::Bounds, vec![Operand::Read(Place::local(i)), Operand::Read(Place::local(len))], span);
        let mut src = Place::local(seq);
        src.proj.push(Proj::Index(Operand::Read(Place::local(i))));
        let rv = match self.locals[e.0 as usize].kind {
            LocalKind::Lent { mutable } => Rvalue::Lend { place: src, mutable },
            LocalKind::Owned => Rvalue::Use(Operand::Move(src)),
            _ => Rvalue::Use(Operand::Read(src)),
        };
        self.assign(Place::local(e), rv, span);
        self.loops.push(LoopTargets { brk: exit, cont: next, depth: self.scopes.len() });
        self.block(body);
        self.terminate(Terminator::Goto(next));
        self.loops.pop();

        self.cur = next;
        self.site(SiteKind::Overflow(BinOp::Add), vec![Operand::Read(Place::local(i)), Operand::Const(Const::Int(1))], span);
        self.assign(
            Place::local(i),
            Rvalue::Binary { op: BinOp::Add, lhs: Operand::Read(Place::local(i)), rhs: Operand::Const(Const::Int(1)) },
            span,
        );
        self.terminate(Terminator::Goto(head));

        self.cur = exit;
        if perm {
            self.scopes.last_mut().unwrap().push(e);
        }
    }

    fn concur(&mut self, detached: bool, captures: &[hir::CaptureBinding], body: &hir::Block, otherwise: Option<&hir::Otherwise>, fails: &[(String, Option<Ty>)], span: SpanId) {
        // The task's body becomes its own MIR function; its captures are its
        // parameters, taken with the verbs the writer wrote. The checker put
        // the task's locals in the enclosing table, so the numbering is shared.
        let task_body = hir::Body {
            locals: self.hir_locals.clone(),
            params: captures.iter().map(|c| c.inner).collect(),
            ret: Ty::Nothing,
            fails: fails.to_vec(),
            block: body.clone(),
            span: self.spans[span.0 as usize],
            name: format!("a task in {}", self.fn_name()),
            is_main: false,
        };
        let idx = FnIdx(self.arena.functions.len() as u32);
        self.arena.functions.push(placeholder(&task_body));
        let mut fb = FnBuilder::new(self.arena, None, &task_body, true);
        fb.captures = captures.iter().map(|c| LocalId(c.inner.0)).collect();
        fb.lower_body(&task_body);
        let f = fb.finish(&task_body);
        self.arena.functions[idx.0 as usize] = f;

        let caps: Vec<(Verb, Place)> = captures.iter().map(|c| (c.verb, Place::local(LocalId(c.outer.0)))).collect();
        // A lend into a task must end before the lender's scope does: the
        // proof structured tasks make possible (`concurrency.md`).
        for (verb, place) in &caps {
            if matches!(verb, Verb::Lend | Verb::LendMut) {
                self.site(SiteKind::TaskLendOutlives, vec![Operand::Read(place.clone())], span);
            }
        }
        self.push(StmtKind::Task { detached, body: idx, captures: caps }, span);
        if !detached {
            *self.tasks_in_scope.last_mut().unwrap() += 1;
        }
        if let Some(o) = otherwise {
            // The handler runs where the block waits, not here.
            let (arms, after) = self.fail_blocks(o, fails, None, span);
            self.task_fails.last_mut().unwrap().extend(arms);
            self.task_after.last_mut().unwrap().push(after);
        }
    }

    fn whichever(&mut self, arms: &[hir::WhicheverArm], otherwise: Option<&hir::Otherwise>, span: SpanId) {
        let join = self.new_block();
        let mut marms = Vec::new();
        for arm in arms {
            let ch = self.eval(&arm.channel, span);
            let b = self.new_block();
            marms.push(WhicheverArm { channel: ch, dest: Place::local(LocalId(arm.value.0)), block: b });
        }
        let closed_b = otherwise.map(|_| self.new_block());
        self.terminate(Terminator::Whichever { arms: marms.clone(), closed: closed_b });
        for (arm, marm) in arms.iter().zip(&marms) {
            self.cur = marm.block;
            self.push_scope();
            self.scopes.last_mut().unwrap().push(Place::local(LocalId(arm.value.0)).local);
            for st in &arm.body.stmts {
                if self.closed() {
                    break;
                }
                self.stmt(st);
            }
            self.pop_scope(span);
            self.terminate(Terminator::Goto(join));
        }
        if let (Some(b), Some(o)) = (closed_b, otherwise) {
            self.cur = b;
            match o {
                hir::Otherwise::Arms(arms) => {
                    for arm in arms {
                        self.push_scope();
                        for st in &arm.body.stmts {
                            if self.closed() {
                                break;
                            }
                            self.stmt(st);
                        }
                        self.pop_scope(span);
                    }
                    self.terminate(Terminator::Goto(join));
                }
                _ => self.otherwise(o, None, join, span),
            }
        }
        self.cur = join;
    }

    // ------------------------------------------------------------ stores

    /// Put a value into a place, with the check site a typed store needs.
    fn store(&mut self, place: Place, value: &hir::Expr, target: &Ty, typed: bool, span: SpanId) {
        // A call fills its destination directly, so a failing call has one
        // edge rather than a value to test afterwards.
        if let hir::ExprKind::Call { .. } | hir::ExprKind::Conv { .. } = &value.kind {
            self.call_into(Some(place.clone()), value, span);
            self.store_site(&place, target, typed, span);
            return;
        }
        let rv = self.rvalue(value, span);
        if typed {
            if let Some(op) = rvalue_operand(&rv) {
                self.narrow_site(&op, target, &value.ty, span);
            }
        }
        self.assign(place.clone(), rv, span);
        self.store_site(&place, target, typed, span);
    }

    /// After a store into a bounded collection, its bound is a check site.
    fn store_site(&mut self, place: &Place, target: &Ty, typed: bool, span: SpanId) {
        if !typed {
            return;
        }
        if let Ty::Bounded { bound: Bound::Len { .. }, .. } = self.resolve_ty(target) {
            self.site(SiteKind::LengthBound { to: target.clone() }, vec![Operand::Read(place.clone())], span);
        }
    }

    /// A value going into a narrower or bounded type.
    fn narrow_site(&mut self, op: &Operand, target: &Ty, from: &Ty, span: SpanId) {
        let target = self.resolve_ty(target);
        let from = self.resolve_ty(from);
        if needs_narrow_check(&from, &target) {
            self.site(SiteKind::Narrow { to: target }, vec![op.clone()], span);
        }
    }

    fn resolve_ty(&self, t: &Ty) -> Ty {
        t.clone()
    }

    // ------------------------------------------------------------ expressions

    /// A value, as an operand. Calls split the block.
    fn eval(&mut self, e: &hir::Expr, span: SpanId) -> Operand {
        let span = self.span_id(e.span).max_of(span);
        match &e.kind {
            hir::ExprKind::Int(v) => Operand::Const(Const::Int(*v)),
            hir::ExprKind::Bin(t) => Operand::Const(Const::Bin(t.clone())),
            hir::ExprKind::Str(t) => Operand::Const(Const::Str(t.clone())),
            hir::ExprKind::Char(c) => Operand::Const(Const::Char(c.clone())),
            hir::ExprKind::Bool(b) => Operand::Const(Const::Bool(*b)),
            hir::ExprKind::Duration(n) => Operand::Const(Const::Duration(*n)),
            hir::ExprKind::Const(c) => Operand::Const(Const::Const(*c)),
            hir::ExprKind::SizeOf { align, ty } => Operand::Const(Const::SizeOf { align: *align, ty: ty.clone() }),
            hir::ExprKind::Nothing => Operand::Const(Const::Nothing),
            hir::ExprKind::Local(l) => Operand::Read(Place::local(LocalId(l.0))),
            hir::ExprKind::Read(p) => {
                let place = self.place(p, span);
                Operand::Read(place)
            }
            hir::ExprKind::Give(p) => {
                let place = self.place(p, span);
                Operand::Move(place)
            }
            hir::ExprKind::Call { .. } | hir::ExprKind::Conv { .. } => {
                let ty = e.ty.clone();
                if ty == Ty::Nothing {
                    self.call_into(None, e, span);
                    Operand::Const(Const::Nothing)
                } else {
                    let t = self.temp(&ty, span);
                    self.call_into(Some(Place::local(t)), e, span);
                    Operand::Read(Place::local(t))
                }
            }
            _ => {
                let rv = self.rvalue(e, span);
                let ty = e.ty.clone();
                self.into_temp(rv, &ty, span)
            }
        }
    }

    fn rvalue(&mut self, e: &hir::Expr, span: SpanId) -> Rvalue {
        let span = self.span_id(e.span).max_of(span);
        match &e.kind {
            hir::ExprKind::Copy(p) => {
                let place = self.place(p, span);
                Rvalue::Duplicate(place)
            }
            hir::ExprKind::Lend { place, mutable } => {
                let p = self.place(place, span);
                Rvalue::Lend { place: p, mutable: *mutable }
            }
            hir::ExprKind::ExternRead(id) => Rvalue::ExternRead(*id),
            hir::ExprKind::Unary { op, expr } => {
                let operand = self.eval(expr, span);
                if *op == UnOp::Neg && self.resolve_ty(&expr.ty).is_int() {
                    self.site(SiteKind::Overflow(BinOp::Sub), vec![operand.clone()], span);
                }
                Rvalue::Unary { op: *op, operand }
            }
            hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                let l = self.eval(lhs, span);
                let r = self.eval(rhs, span);
                self.binary_sites(op, &l, &r, &lhs.ty, span);
                Rvalue::Binary { op: op.clone(), lhs: l, rhs: r }
            }
            hir::ExprKind::UserOp { inst, lhs, rhs } => {
                // An operator is a call under another spelling.
                let args = vec![self.arg(lhs, span), self.arg(rhs, span)];
                let ty = e.ty.clone();
                let dest = self.temp(&ty, span);
                let idx = self.fn_idx(*inst);
                let ok = self.new_block();
                self.terminate(Terminator::Call { callee: Callee::Fn(idx), args, dest: Some(Place::local(dest)), ok, fail: Vec::new() });
                self.cur = ok;
                Rvalue::Use(Operand::Read(Place::local(dest)))
            }
            hir::ExprKind::List(items) => {
                let ops = items.iter().map(|a| self.arg_operand(a, span)).collect();
                Rvalue::MakeList(ops)
            }
            hir::ExprKind::MapLit(pairs) => {
                let ps = pairs.iter().map(|(k, v)| (self.arg_operand(k, span), self.arg_operand(v, span))).collect();
                Rvalue::MakeMap(ps)
            }
            hir::ExprKind::ListEmpty => Rvalue::MakeList(Vec::new()),
            hir::ExprKind::MapEmpty => Rvalue::MakeMap(Vec::new()),
            hir::ExprKind::Construct { id, fields } => {
                let ops = fields.iter().map(|a| self.arg_operand(a, span)).collect();
                Rvalue::MakeStruct { id: *id, fields: ops }
            }
            hir::ExprKind::EnumCase { id, case, payload } => {
                let p = payload.as_ref().map(|a| self.arg_operand(a, span));
                Rvalue::MakeEnum { id: *id, case: *case, payload: p }
            }
            hir::ExprKind::Channel { elem, capacity } => Rvalue::MakeChannel { elem: elem.clone(), capacity: *capacity },
            hir::ExprKind::FnRef(inst) => {
                let idx = self.fn_idx(*inst);
                Rvalue::FnRef(idx)
            }
            hir::ExprKind::FnLit { inst, captures } => {
                let idx = self.fn_idx(*inst);
                let caps = captures.iter().map(|c| (c.verb, Place::local(LocalId(c.outer.0)))).collect();
                Rvalue::Closure { body: idx, captures: caps }
            }
            hir::ExprKind::BundleElem { place, index } => {
                let mut p = self.place(place, span);
                p.proj.push(Proj::BundleElem(*index));
                Rvalue::Use(Operand::Read(p))
            }
            _ => Rvalue::Use(self.eval(e, span)),
        }
    }

    /// The check sites a binary operation carries (`precog.md`).
    fn binary_sites(&mut self, op: &BinOp, l: &Operand, r: &Operand, lhs_ty: &Ty, span: SpanId) {
        let ty = self.resolve_ty(lhs_ty);
        let int = ty.is_int();
        match op {
            // Wrapping arithmetic is always defined: never a site.
            BinOp::WrapAdd | BinOp::WrapSub | BinOp::WrapMul | BinOp::WrapPow => {}
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow if int => {
                self.site(SiteKind::Overflow(op.clone()), vec![l.clone(), r.clone()], span);
            }
            BinOp::Div if int => {
                self.site(SiteKind::DivZero, vec![r.clone()], span);
                // The one overflowing division: the lowest value by -1.
                self.site(SiteKind::Overflow(BinOp::Div), vec![l.clone(), r.clone()], span);
            }
            BinOp::Mod if int => {
                self.site(SiteKind::ModZero, vec![r.clone()], span);
            }
            BinOp::Shl => {
                self.site(SiteKind::ShiftWidth, vec![l.clone(), r.clone()], span);
            }
            _ => {}
        }
    }

    fn place(&mut self, p: &hir::Place, span: SpanId) -> Place {
        let mut out = Place::local(LocalId(p.base.0));
        for pr in &p.proj {
            match pr {
                hir::Proj::Field(i) => out.proj.push(Proj::Field(*i)),
                hir::Proj::Index(e) => {
                    let idx = self.eval(e, span);
                    // Every index is a bounds site: 1-based, within the length.
                    let len_ty = Ty::u64();
                    let len = self.temp(&len_ty, span);
                    let base = Place { local: out.local, proj: out.proj.clone() };
                    self.push(StmtKind::Op { dest: Some(Place::local(len)), op: Op::ListLength, args: vec![Operand::Read(base)] }, span);
                    self.site(SiteKind::Bounds, vec![idx.clone(), Operand::Read(Place::local(len))], span);
                    out.proj.push(Proj::Index(idx));
                }
            }
        }
        out
    }

    fn fn_idx(&mut self, inst: FnInst) -> FnIdx {
        match self.arena.inst_idx.get(&inst) {
            Some(i) => *i,
            None => {
                // A function with no body (never happens for a call the
                // checker accepted); keep the graph honest rather than panic.
                FnIdx(u32::MAX)
            }
        }
    }

    fn arg(&mut self, a: &hir::Arg, span: SpanId) -> Arg {
        match a {
            hir::Arg::Value(e) => Arg::Value(self.eval(e, span)),
            hir::Arg::Lend { place, mutable } => {
                let p = self.place(place, span);
                Arg::Lend { place: p, mutable: *mutable }
            }
            hir::Arg::Give(place) => {
                let p = self.place(place, span);
                Arg::Give(p)
            }
            hir::Arg::Copy(place) => {
                let p = self.place(place, span);
                Arg::Copy(p)
            }
            hir::Arg::Spread { expr, count } => {
                let (place, verb) = match &**expr {
                    hir::Arg::Lend { place, .. } => (self.place(place, span), None),
                    hir::Arg::Give(place) => (self.place(place, span), Some(Verb::Give)),
                    hir::Arg::Copy(place) => (self.place(place, span), Some(Verb::Copy)),
                    other => {
                        let op = self.arg_operand(other, span);
                        match op.place() {
                            Some(p) => (p.clone(), None),
                            None => (Place::local(LocalId(0)), None),
                        }
                    }
                };
                Arg::Spread { place, count: *count, verb }
            }
        }
    }

    /// An argument that is being stored rather than passed: the operand it is.
    fn arg_operand(&mut self, a: &hir::Arg, span: SpanId) -> Operand {
        match a {
            hir::Arg::Value(e) => self.eval(e, span),
            hir::Arg::Give(place) => {
                let p = self.place(place, span);
                Operand::Move(p)
            }
            hir::Arg::Copy(place) => {
                let p = self.place(place, span);
                let ty = place.ty.clone();
                self.into_temp(Rvalue::Duplicate(p), &ty, span)
            }
            hir::Arg::Lend { place, mutable } => {
                let p = self.place(place, span);
                let ty = place.ty.clone();
                self.into_temp(Rvalue::Lend { place: p, mutable: *mutable }, &ty, span)
            }
            hir::Arg::Spread { expr, .. } => self.arg_operand(expr, span),
        }
    }

    /// A call or a conversion, with its failure edge.
    fn call_into(&mut self, dest: Option<Place>, e: &hir::Expr, span: SpanId) {
        let span = self.span_id(e.span).max_of(span);
        let (callee, args, otherwise, fails) = match &e.kind {
            hir::ExprKind::Call { callee, args, otherwise, fails } => {
                let c = match callee {
                    hir::Callee::Fn(inst) => {
                        let idx = self.fn_idx(*inst);
                        Callee::Fn(idx)
                    }
                    hir::Callee::Extern(id) => Callee::Extern(*id),
                    hir::Callee::Value(v) => {
                        let op = self.eval(v, span);
                        Callee::Value(op)
                    }
                    hir::Callee::Builtin(b) => Callee::Op(builtin_op(b)),
                };
                let margs: Vec<Arg> = args.iter().map(|a| self.arg(a, span)).collect();
                (c, margs, otherwise.as_deref(), fails.clone())
            }
            hir::ExprKind::Conv { target, expr, otherwise, fails } => {
                let op = self.eval(expr, span);
                let f: Vec<(String, Option<Ty>)> = fails.iter().map(|n| (n.clone(), None)).collect();
                if f.is_empty() {
                    // A conversion that cannot fail is an ordinary instruction.
                    let rv = Rvalue::Convert { to: target.clone(), operand: op };
                    match dest {
                        Some(p) => self.assign(p, rv, span),
                        None => {
                            let ty = target.clone();
                            let _ = self.into_temp(rv, &ty, span);
                        }
                    }
                    return;
                }
                (Callee::Convert(target.clone()), vec![Arg::Value(op)], otherwise.as_deref(), f)
            }
            _ => unreachable!("call_into on {:?}", e.kind),
        };
        // A language operation that cannot fail is a statement, not an edge.
        if fails.is_empty() {
            if let Callee::Op(op) = &callee {
                let op = op.clone();
                let ops: Vec<Operand> = args.iter().map(arg_as_operand).collect();
                self.push(StmtKind::Op { dest: dest.clone(), op: op.clone(), args: ops }, span);
                self.after_op_sites(&op, &args, span);
                return;
            }
        }
        let ok = self.new_block();
        let fail_arms = self.fail_blocks_for_call(otherwise, &fails, dest.clone(), ok, span);
        self.terminate(Terminator::Call { callee, args, dest, ok, fail: fail_arms });
        self.cur = ok;
    }

    /// A change to a bounded collection is checked after it happens.
    fn after_op_sites(&mut self, op: &Op, args: &[Arg], span: SpanId) {
        let changes = matches!(op, Op::ListAppend | Op::ListRemove | Op::MapSet | Op::MapRemove);
        if !changes {
            return;
        }
        if let Some(Arg::Lend { place, .. }) = args.first() {
            let place = place.clone();
            let base = self.locals[place.local.0 as usize].ty.clone();
            let ty = projected_ty(&base, &place.proj, &self.arena.items);
            if let Ty::Bounded { bound: Bound::Len { .. }, .. } = &ty {
                self.site(SiteKind::LengthBound { to: ty }, vec![Operand::Read(place)], span);
            }
        }
    }

    /// The blocks a failing call's failure lands in, one arm per reason.
    fn fail_blocks_for_call(&mut self, otherwise: Option<&hir::Otherwise>, fails: &[(String, Option<Ty>)], dest: Option<Place>, ok: BlockId, span: SpanId) -> Vec<FailArm> {
        let Some(o) = otherwise else { return Vec::new() };
        self.fail_blocks(o, fails, Some((dest, ok)), span).0
    }

    /// Lower an `otherwise` into blocks: one arm per reason the callee lists.
    /// `cont` is where the success path carries on, when there is one (a task's
    /// handler has none — the block it belongs to has already ended).
    fn fail_blocks(&mut self, o: &hir::Otherwise, fails: &[(String, Option<Ty>)], cont: Option<(Option<Place>, BlockId)>, span: SpanId) -> (Vec<FailArm>, BlockId) {
        let saved = self.cur;
        let after = match cont {
            Some((_, ok)) => ok,
            None => self.new_block(),
        };
        let dest = cont.and_then(|(d, _)| d);
        let out = match o {
            hir::Otherwise::Arms(arms) => {
                let mut out = Vec::new();
                for arm in arms {
                    let b = self.new_block();
                    let payload = arm.bind.map(|l| Place::local(LocalId(l.0)));
                    out.push((arm, FailArm { index: arm.index, payload, block: b }));
                }
                for (arm, m) in &out {
                    self.cur = m.block;
                    self.push_scope();
                    if let Some(l) = arm.bind {
                        self.scopes.last_mut().unwrap().push(LocalId(l.0));
                    }
                    for st in &arm.body.stmts {
                        if self.closed() {
                            break;
                        }
                        self.stmt(st);
                    }
                    self.pop_scope(span);
                    self.terminate(Terminator::Goto(after));
                }
                out.into_iter().map(|(_, m)| m).collect()
            }
            hir::Otherwise::PassUp => {
                // The failure climbs unchanged, under *this* function's number
                // for the same reason — the caller's list has the name.
                let mine = self.my_fails();
                let mut out = Vec::new();
                for (i, (name, payload_ty)) in fails.iter().enumerate() {
                    let b = self.new_block();
                    self.cur = b;
                    let index = mine.iter().position(|(n, _)| n == name).unwrap_or(i);
                    let payload = payload_ty.as_ref().map(|t| {
                        let p = self.temp(t, span);
                        Place::local(p)
                    });
                    self.unwind_to(0, span);
                    self.terminate(Terminator::Fail { index, payload: payload.clone().map(Operand::Move) });
                    out.push(FailArm { index: i, payload, block: b });
                }
                out
            }
            _ => {
                // One block for every reason: a value, or a leaving statement.
                let b = self.new_block();
                self.cur = b;
                self.otherwise(o, dest, after, span);
                fails.iter().enumerate().map(|(i, _)| FailArm { index: i, payload: None, block: b }).collect()
            }
        };
        self.cur = saved;
        (out, after)
    }

    fn fn_name(&self) -> String {
        match self.inst {
            Some(i) => self.arena.instances[i.0 as usize].body.as_ref().map(|b| b.name.clone()).unwrap_or_else(|| "a function".into()),
            None => "a task".into(),
        }
    }

    /// The failures this function itself lists.
    fn my_fails(&self) -> Vec<(String, Option<Ty>)> {
        match self.inst {
            Some(i) => self.arena.instances[i.0 as usize].sig.fails.clone(),
            None => Vec::new(),
        }
    }

    /// What one non-arm `otherwise` does, lowered into the current block.
    fn otherwise(&mut self, o: &hir::Otherwise, dest: Option<Place>, ok: BlockId, span: SpanId) {
        match o {
            hir::Otherwise::Value(e) => {
                let v = self.eval(e, span);
                if let Some(p) = dest {
                    self.assign(p, Rvalue::Use(v), span);
                }
                self.terminate(Terminator::Goto(ok));
            }
            hir::Otherwise::Leave(s) => {
                self.stmt(s);
                if !self.closed() {
                    self.terminate(Terminator::Goto(ok));
                }
            }
            hir::Otherwise::PassUp | hir::Otherwise::Arms(_) => unreachable!("handled in fail_blocks"),
        }
    }
}

fn placeholder(body: &hir::Body) -> Function {
    Function {
        name: body.name.clone(),
        inst: None,
        params: Vec::new(),
        locals: Vec::new(),
        blocks: vec![BasicBlock { stmts: Vec::new(), term: Terminator::Unreachable }],
        ret: Ty::Nothing,
        ret_local: None,
        fails: Vec::new(),
        sites: Vec::new(),
        spans: Vec::new(),
        is_main: body.is_main,
        is_task: false,
        captures: Vec::new(),
    }
}

fn arg_as_operand(a: &Arg) -> Operand {
    match a {
        Arg::Value(o) => o.clone(),
        Arg::Lend { place, .. } | Arg::Copy(place) => Operand::Read(place.clone()),
        Arg::Give(place) => Operand::Move(place.clone()),
        Arg::Spread { place, .. } => Operand::Read(place.clone()),
    }
}

fn rvalue_operand(rv: &Rvalue) -> Option<Operand> {
    match rv {
        Rvalue::Use(o) => Some(o.clone()),
        Rvalue::Binary { .. } | Rvalue::Unary { .. } | Rvalue::Convert { .. } => None,
        _ => None,
    }
}

/// Does storing a `from` into a `to` need a check?
pub fn needs_narrow_check(from: &Ty, to: &Ty) -> bool {
    // Into a bound: always, unless the source's bound is inside it.
    if let Ty::Bounded { bound: to_b, inner } = to {
        if let Bound::Int { lo, hi } = to_b {
            if let Ty::Bounded { bound: Bound::Int { lo: flo, hi: fhi }, .. } = from {
                return !(flo >= lo && fhi <= hi);
            }
            let _ = inner;
            return true;
        }
        if let Bound::Len { lo, hi, unit } = to_b {
            if let Ty::Bounded { bound: Bound::Len { lo: flo, hi: fhi, unit: funit }, .. } = from {
                return !(funit == unit && flo >= lo && fhi <= hi);
            }
            return true;
        }
        return true;
    }
    match (from.base(), to.base()) {
        (Ty::Int(a), Ty::Int(b)) => match (a, b) {
            (IntW::Fixed { signed: s1, bits: b1 }, IntW::Fixed { signed: s2, bits: b2 }) => {
                let (f_lo, f_hi) = crate::resolve::int_range(*s1, *b1);
                let (t_lo, t_hi) = crate::resolve::int_range(*s2, *b2);
                f_lo < t_lo || f_hi > t_hi
            }
            // A width Precog has not chosen: the value decides, and the store
            // is where it has to fit.
            _ => true,
        },
        (Ty::Bin(_), Ty::Bin(_)) => true,
        _ => false,
    }
}

fn builtin_op(b: &Builtin) -> Op {
    match b {
        Builtin::ListLength => Op::ListLength,
        Builtin::ListAppend => Op::ListAppend,
        Builtin::ListRemove => Op::ListRemove,
        Builtin::ArrayLength => Op::ArrayLength,
        Builtin::MapLength => Op::MapLength,
        Builtin::MapHas => Op::MapHas,
        Builtin::MapGet => Op::MapGet,
        Builtin::MapSet => Op::MapSet,
        Builtin::MapRemove => Op::MapRemove,
        Builtin::MapKeys => Op::MapKeys,
        Builtin::MapValues => Op::MapValues,
        Builtin::MapPairs => Op::MapPairs,
        Builtin::StrBytes => Op::StrBytes,
        Builtin::StrPoints => Op::StrPoints,
        Builtin::StrGraphemes => Op::StrGraphemes,
        Builtin::StrEach(u) => Op::StrEach(*u),
        Builtin::CharCode => Op::CharCode,
        Builtin::CharFromCode(u) => Op::CharFromCode(*u),
        Builtin::PtrRead => Op::PtrRead,
        Builtin::PtrWrite => Op::PtrWrite,
        Builtin::PtrAt => Op::PtrAt,
        Builtin::PtrIsNull => Op::PtrIsNull,
        Builtin::ChannelSend => Op::ChannelSend,
        Builtin::ChannelTake => Op::ChannelTake,
        Builtin::Std(n) => Op::Std(n.clone()),
    }
}

/// The type a projection reaches.
pub fn projected_ty(base: &Ty, proj: &[Proj], items: &crate::resolve::Items) -> Ty {
    let mut t = base.clone();
    for p in proj {
        t = match (p, t.base().clone()) {
            (Proj::Field(i), Ty::Struct { id, args }) => items.structs[id.0 as usize].fields[*i].ty.subst(&args),
            (Proj::Index(_), Ty::Seq { elem, .. }) => (*elem).clone(),
            (Proj::BundleElem(i), Ty::Bundle(ts)) => ts.get(*i).cloned().unwrap_or(Ty::Nothing),
            (Proj::Deref, Ty::Ptr { elem, .. }) => (*elem).clone(),
            _ => Ty::Nothing,
        };
    }
    t
}

impl SpanId {
    /// The more specific of two spans: an expression's own, when it has one.
    fn max_of(self, _other: SpanId) -> SpanId {
        self
    }
}

impl ParamKind {
    pub fn is_lent(self) -> bool {
        matches!(self, ParamKind::Lent | ParamKind::LentMut)
    }
}
