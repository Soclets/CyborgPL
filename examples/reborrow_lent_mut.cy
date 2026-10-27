# A function may pass its own `lent.mut` parameter on with `lend.mut` and use it again after (compiler
# claude, 2026-10-17): the callee reborrows it, as Rust's `&mut *x` does.
proc add1 [lent.mut $out: std::ls[int.sign.32]]:
    std::ls.append(lend.mut $out, value: 1);
end of proc add1
proc walk [lent.mut $out: std::ls[int.sign.32]]:
    add1(lend.mut $out); std::ls.append(lend.mut $out, value: 2);
end of proc walk
!MAIN:
    var $xs: std::ls[int.sign.32] = []; walk(lend.mut $xs); std::print.shortest(std::ls.len(lend $xs), newln);
!end of MAIN
