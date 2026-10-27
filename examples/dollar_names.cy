# A variable is written `$name`, or quoted, `'total due'`: the same variable either way (owner, 2026-10-22: "Why `$`?
# Even though, it requires a shift, it's easier to write"). Good practice: `$`, and quotes only for a name that needs
# them. A call takes `( )`, a definition keeps `[ ]` (owner, 2026-10-22: "`( )` runs something; `[ ]` describes or
# holds something").
function twice [$n: integer.signed.64_bit] -> integer.signed.64_bit:
    return $n times 2;
end of function twice

!MAIN:
    variable $total := 1;
    change $total = $total plus 1;
    let $'total due' := twice($total) times 10;
    let $xs: std::list[integer.signed.64_bit] = [$total, $'total due'];
    std::print.shortest($total, " ", $'total due', " ", $xs[position: 2], newline);
!end of MAIN
