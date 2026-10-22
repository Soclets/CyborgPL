# A function may pass its own `lent.mut` parameter on with `lend.mut` and use it again after (compiler
# claude, 2026-10-17): the callee reborrows it, as Rust's `&mut *x` does.
fn add1 [lent.mut 'out': std::list[i32]] { std::list.append[lend.mut 'out', value: 1]; }
fn walk [lent.mut 'out': std::list[i32]] { add1[lend.mut 'out']; std::list.append[lend.mut 'out', value: 2]; }
MAIN { let mut 'xs': std::list[i32] = []; walk[lend.mut 'xs']; std::print.useful[std::list.length[lend 'xs'], newln]; }
