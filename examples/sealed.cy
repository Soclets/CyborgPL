# Three separate promises (owner, 2026-09-25):
#   mut   — the value may be changed          immut — it may not
#   no_shadow    — the name may not be shadowed; it means this variable all the way down
# Shadowing (`let shadow $a := …`) makes a new variable under the old name, so it needs no `mut`.
!MAIN:
    let $a := 1;
    let shadow $a := 2;                  # an immut can be shadowed: nothing is changed
    let $b := 3;                    # no word: it may not change either
    var no_shadow $total := 0;        # changeable, and 'total' always means this one
    change $total = $total plus $a plus $b;
    std::print.shortest($a, " ", $b, " ", $total, newln);
!end of MAIN
