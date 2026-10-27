# A variable is written `$name`, or quoted, `'total due'`: the same variable either way (owner, 2026-10-22: "Why `$`?
# Even though, it requires a shift, it's easier to write"). Good practice: `$`, and quotes only for a name that needs
# them. A call takes `( )`, a definition keeps `[ ]` (owner, 2026-10-22: "`( )` runs something; `[ ]` describes or
# holds something").
fn twice [let $n: int.sign.64] -> int.sign.64:
    ret $n times 2;
end of fn twice

!MAIN:
    var $total := 1;
    change $total = $total plus 1;
    let $'total due' := twice($total) times 10;
    let $xs: std::ls[int.sign.64] = [$total, $'total due'];
    std::print.shortest($total, " ", $'total due', " ", $xs[pos: 2], newln);
!end of MAIN
