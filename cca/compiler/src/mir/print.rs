//! MIR, written out. For reading the graph and for the report.

use std::fmt::Write as _;

use super::*;
use crate::resolve::Items;
use crate::types::TySpelling;

pub fn function(f: &Function, items: &Items) -> String {
    let spell = |t: &Ty| TySpelling { ty: t, structs: &items.structs, enums: &items.enums }.to_string();
    let mut o = String::new();
    let _ = write!(o, "fn {}", f.name);
    let _ = writeln!(o, " -> {}{}", spell(&f.ret), if f.fails.is_empty() { String::new() } else { format!(" or fail[{}]", f.fails.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>().join(", ")) });
    for (i, l) in f.locals.iter().enumerate() {
        let name = l.name.clone().unwrap_or_else(|| format!("t{}", i));
        let _ = writeln!(o, "    let _{} '{}': {}   [{}]", i, name, spell(&l.ty), l.domain.describe());
    }
    for (i, b) in f.blocks.iter().enumerate() {
        let _ = writeln!(o, "  b{}:", i);
        for s in &b.stmts {
            let _ = writeln!(o, "    {}", stmt(&s.kind, f, &spell));
        }
        let _ = writeln!(o, "    {}", term(&b.term, f, &spell));
    }
    o
}

fn stmt(k: &StmtKind, f: &Function, spell: &dyn Fn(&Ty) -> String) -> String {
    match k {
        StmtKind::Assign { place, rvalue } => format!("{} = {}", place_s(place, f), rvalue_s(rvalue, f, spell)),
        StmtKind::Op { dest, op, args } => {
            let a = args.iter().map(|x| operand_s(x, f)).collect::<Vec<_>>().join(", ");
            match dest {
                Some(d) => format!("{} = {:?}[{}]", place_s(d, f), op, a),
                None => format!("{:?}[{}]", op, a),
            }
        }
        StmtKind::Site(id) => {
            let s = f.site(*id);
            let tier = match s.tier {
                Some(crate::ast::Tier::Check) => " check",
                Some(crate::ast::Tier::Nocheck) => " nocheck",
                Some(crate::ast::Tier::NoChecks) => " NO_FUCKING_CHECKS",
                None => "",
            };
            format!("site s{}{}: {}", id.0, tier, s.sentence(spell))
        }
        StmtKind::Drop(p) => format!("drop {}", place_s(p, f)),
        StmtKind::EndLend(p) => format!("end lend {}", place_s(p, f)),
        StmtKind::Task { detached, body, captures, .. } => format!(
            "task{} f{} with [{}]",
            if *detached { ".detached" } else { "" },
            body.0,
            captures.iter().map(|(v, p)| format!("{} {}", v.spelled(), place_s(p, f))).collect::<Vec<_>>().join(", ")
        ),
        StmtKind::Asm { text, .. } => format!("asm {{ {} }}", text.trim().lines().next().unwrap_or("")),
        StmtKind::Nop => "nop".into(),
    }
}

fn term(t: &Terminator, f: &Function, spell: &dyn Fn(&Ty) -> String) -> String {
    match t {
        Terminator::Goto(b) => format!("goto b{}", b.0),
        Terminator::If { cond, then, else_ } => format!("if {} then b{} else b{}", operand_s(cond, f), then.0, else_.0),
        Terminator::Switch { value, cases } => format!("switch {} [{}]", operand_s(value, f), cases.iter().map(|(c, b)| format!("case {} -> b{}", c, b.0)).collect::<Vec<_>>().join(", ")),
        Terminator::Call { callee, args, dest, ok, fail } => {
            let a = args.iter().map(|x| arg_s(x, f)).collect::<Vec<_>>().join(", ");
            let d = dest.as_ref().map(|d| format!("{} = ", place_s(d, f))).unwrap_or_default();
            let fl = if fail.is_empty() { String::new() } else { format!(" fail [{}]", fail.iter().map(|x| format!("{} -> b{}", x.index, x.block.0)).collect::<Vec<_>>().join(", ")) };
            format!("{}{}({}) -> b{}{}", d, callee_s(callee, spell), a, ok.0, fl)
        }
        Terminator::Whichever { arms, closed } => format!(
            "whichever [{}]{}",
            arms.iter().map(|a| format!("{} from {} -> b{}", place_s(&a.dest, f), operand_s(&a.channel, f), a.block.0)).collect::<Vec<_>>().join(", "),
            closed.map(|c| format!(" closed -> b{}", c.0)).unwrap_or_default()
        ),
        Terminator::Join { group, ok } => format!("join group {} -> b{}", group, ok.0),
        Terminator::TaskOutcome { id, ok, fail, .. } => format!("outcome of task {} -> b{} fail [{}]", id, ok.0, fail.iter().map(|x| format!("{} -> b{}", x.index, x.block.0)).collect::<Vec<_>>().join(", ")),
        Terminator::Return => match f.ret_local {
            Some(r) => format!("return {}", place_s(&Place::local(r), f)),
            None => "return".into(),
        },
        Terminator::Fail { index, payload } => format!("fail {}{}", index, payload.as_ref().map(|p| format!("[{}]", operand_s(p, f))).unwrap_or_default()),
        Terminator::Trap(s) => format!("trap s{}", s.0),
        Terminator::Unreachable => "unreachable".into(),
    }
}

fn callee_s(c: &Callee, spell: &dyn Fn(&Ty) -> String) -> String {
    match c {
        Callee::Fn(i) => format!("f{}", i.0),
        Callee::Extern(i) => format!("extern{}", i.0),
        Callee::Value(_) => "a held function".into(),
        Callee::Op(op) => format!("{:?}", op),
        Callee::Convert(t) => format!("to.{}", spell(t)),
    }
}

fn rvalue_s(r: &Rvalue, f: &Function, spell: &dyn Fn(&Ty) -> String) -> String {
    match r {
        Rvalue::Use(o) => operand_s(o, f),
        Rvalue::Duplicate(p) => format!("copy {}", place_s(p, f)),
        Rvalue::Lend { place, mutable } => format!("lend{} {}", if *mutable { ".mut" } else { "" }, place_s(place, f)),
        Rvalue::Unary { op, operand } => format!("{:?} {}", op, operand_s(operand, f)),
        Rvalue::Binary { op, lhs, rhs } => format!("{} {} {}", operand_s(lhs, f), op.spelled(), operand_s(rhs, f)),
        Rvalue::Convert { to, operand } => format!("to.{}[{}]", spell(to), operand_s(operand, f)),
        Rvalue::MakeList(v) => format!("[{}]", v.iter().map(|x| operand_s(x, f)).collect::<Vec<_>>().join(", ")),
        Rvalue::MakeMap(v) => format!("map[{}]", v.iter().map(|(k, x)| format!("{} = {}", operand_s(k, f), operand_s(x, f))).collect::<Vec<_>>().join(", ")),
        Rvalue::MakeStruct { id, fields } => format!("struct{}[{}]", id.0, fields.iter().map(|x| operand_s(x, f)).collect::<Vec<_>>().join(", ")),
        Rvalue::MakeEnum { id, case, payload } => format!("enum{}.{}{}", id.0, case, payload.as_ref().map(|p| format!("[{}]", operand_s(p, f))).unwrap_or_default()),
        Rvalue::MakeChannel { elem, capacity } => format!("channel[{}]{}", spell(elem), capacity.map(|(a, b)| format!("[{}, {}]", a, b)).unwrap_or_default()),
        Rvalue::FnRef(i) => format!("f{}", i.0),
        Rvalue::Closure { body, captures } => format!("closure f{} with [{}]", body.0, captures.iter().map(|(v, p)| format!("{} {}", v.spelled(), place_s(p, f))).collect::<Vec<_>>().join(", ")),
        Rvalue::ExternRead(i) => format!("extern{}", i.0),
    }
}

fn arg_s(a: &Arg, f: &Function) -> String {
    match a {
        Arg::Value(o) => operand_s(o, f),
        Arg::Lend { place, mutable } => format!("lend{} {}", if *mutable { ".mut" } else { "" }, place_s(place, f)),
        Arg::Give(p) => format!("give {}", place_s(p, f)),
        Arg::Copy(p) => format!("copy {}", place_s(p, f)),
        Arg::Spread { place, count, verb } => format!("...{}{} ({} of them)", verb.map(|v| format!("{} ", v.spelled())).unwrap_or_default(), place_s(place, f), count),
    }
}

fn operand_s(o: &Operand, f: &Function) -> String {
    match o {
        Operand::Const(c) => const_s(c),
        Operand::Read(p) => place_s(p, f),
        Operand::Move(p) => format!("give {}", place_s(p, f)),
    }
}

fn const_s(c: &Const) -> String {
    match c {
        Const::Int(v) => v.to_string(),
        Const::Bin(v) => v.clone(),
        Const::Str(v) => format!("{:?}", v),
        Const::Char(v) => format!("`{}`", v),
        Const::Bool(v) => v.to_string(),
        Const::Duration(v) => format!("{} nanos", v),
        Const::Const(i) => format!("const{}", i.0),
        Const::SizeOf { align, ty: _ } => if *align { "align.of[…]".into() } else { "size.of[…]".into() },
        Const::Nothing => "nothing".into(),
    }
}

fn place_s(p: &Place, f: &Function) -> String {
    let mut s = match &f.locals.get(p.local.0 as usize).and_then(|l| l.name.clone()) {
        Some(n) => format!("'{}'", n),
        None => format!("_{}", p.local.0),
    };
    for pr in &p.proj {
        match pr {
            Proj::Field(i) => {
                let _ = write!(s, ".{}", i);
            }
            Proj::Index(o) => {
                let _ = write!(s, "[{}]", operand_s(o, f));
            }
            Proj::BundleElem(i) => {
                let _ = write!(s, "[bundle {}]", i);
            }
            Proj::Deref => s = format!("*{}", s),
        }
    }
    s
}
