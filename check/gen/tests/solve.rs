//! The solver translation finds a failing input where one exists, and none
//! where none does.

use check_gen::ast::*;
use check_gen::solve::{encode, parse_answers, Answer};
use std::process::Command;

fn program(main: Vec<Stmt>) -> Program {
    Program {
        seed: 0,
        printer: String::new(),
        inputs: vec![Input { name: "in1".into(), var: "CHK_IN1".into(), domain: None }],
        funcs: vec![],
        main,
        loose: false,
    }
}

fn ask(p: &Program) -> Vec<Answer> {
    let q = encode(p).expect("small program");
    let dir = std::env::temp_dir().join(format!("check-solve-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    // one file per question: the tests run in parallel
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let f = dir.join(format!("q{}.smt2", N.fetch_add(1, std::sync::atomic::Ordering::SeqCst)));
    std::fs::write(&f, &q.script).unwrap();
    let out = Command::new("z3").arg("-smt2").arg(&f).output().expect("z3 on PATH");
    parse_answers(&String::from_utf8_lossy(&out.stdout), q.sites.len(), q.inputs.len())
}

fn var(n: &str) -> Box<Expr> {
    Box::new(Expr::Var(n.into()))
}

#[test]
fn narrowing_fails_past_255() {
    // let 'x': uint8 = to.uint8['in1'];
    let p = program(vec![Stmt::Let { tier: Tier::Default, name: "x".into(), mutability: Mutability::Plain, ty: Ty::U8.into(), value: Expr::To(Ty::U8, var("in1")) }]);
    let a = ask(&p);
    let fails: Vec<&Answer> = a.iter().filter(|x| matches!(x, Answer::Fails(_))).collect();
    assert!(!fails.is_empty(), "expected a failing input, got {a:?}");
    for f in fails {
        if let Answer::Fails(v) = f {
            assert!(v[0] >= 256, "a failing input must be at least 256, got {v:?}");
        }
    }
}

#[test]
fn guarded_narrowing_never_fails() {
    // let 'x': uint8 = to.uint8['in1' mod 256];
    let p = program(vec![Stmt::Let {
        tier: Tier::Default,
        name: "x".into(),
        mutability: Mutability::Plain,
        ty: Ty::U8.into(),
        value: Expr::To(Ty::U8, Box::new(Expr::Bin(BinOp::Mod, var("in1"), Box::new(Expr::Lit(256))))),
    }]);
    let a = ask(&p);
    assert!(a.iter().all(|x| *x == Answer::Never), "expected every site safe, got {a:?}");
}

#[test]
fn a_failure_behind_an_if_needs_its_condition() {
    // if 'in1' > 1000 { let 'x': int8 = to.int8['in1' mod 1000] ... }  ->  never (value < 1000 but int8 max 127: fails)
    // Here: if 'in1' == 7 { let 'y': uint8 = to.uint8['in1' x 100]; }  fails only at in1 = 7 (700 > 255).
    let cond = Expr::Cmp(CmpOp::Eq, var("in1"), Box::new(Expr::Lit(7)));
    let body = vec![Stmt::Let {
        tier: Tier::Default,
        name: "y".into(),
        mutability: Mutability::Plain,
        ty: Ty::U8.into(),
        value: Expr::To(Ty::U8, Box::new(Expr::Bin(BinOp::Mul, var("in1"), Box::new(Expr::Lit(100))))),
    }];
    let p = program(vec![Stmt::If { arms: vec![(cond, body)], otherwise: None }]);
    let a = ask(&p);
    assert!(a.contains(&Answer::Fails(vec![7])), "expected in1 = 7, got {a:?}");
}
