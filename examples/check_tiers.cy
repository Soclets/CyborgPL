# `check`: proving a narrowing safe, with a guard the prover can see.
fn draw ['y': f16] { std::print.useful["drawing ", 'y', newln]; }
MAIN {
    # A guardrail the compiler can see: past the `if`, the value is inside f16's range,
    # so the narrowing below is proven and `check` says so. A name never changes numeric
    # family (`types.md`), so the value is a bin from the start — `to.f16` would be the
    # way to cross from an integer, and it would fail rather than round silently.
    let 'x': f32 = 70000.0;
    if 'x' > 65_504.0 {
        std::print.stored["too big", newln];
    } else {
        check let 'y': f16 = 'x';
        draw['y'];
    }
    let 'xs' := [1, 2, 3];
    check {
        for 'i' [from: 1, to: list.length[lend 'xs'], by: 1] {
            std::print.useful['xs'[pos: 'i'], " "];
        }
    }
    std::print.stored[newln];
    no_check change 'xs'[pos: 2] = 9;
    # crossing families is spelled, and it fails rather than rounds (`types.md`)
    check if 'x' > 0.0 { change 'xs'[pos: 1] = to.i32['x' div.near 2.0] otherwise 0; }
    std::print.useful['xs', newln];
}
