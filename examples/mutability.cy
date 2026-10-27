# Plain, `mut` and `immut` variables, and an `immut` parameter.
fn twice [let $n: int.sign.32] -> int.sign.32:
    ret $n times 2;
end of fn twice

!MAIN:
    var $plain := 1;
    change $plain to $plain plus 1;
    var $counter := 0;
    change $counter to $counter plus 1;
    let $limit: int.sign.32 := 10;
    std::print.shortest($plain, " ", $counter, " ", $limit, " ", twice($limit), newln);
!end of MAIN
