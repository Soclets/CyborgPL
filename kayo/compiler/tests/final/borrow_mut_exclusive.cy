# While a `lent.mut` borrow is in use, what it borrows is not read by anyone
# else.
proc pickmut [lent.mut $a: std::ls[i32]] -> lent.mut std::ls[i32]:
    ret lend.mut $a;
;proc pickmut
MAIN:
    var $xs: std::ls[i32] = [1, 2];
    var $m = pickmut(lend.mut $xs);
    let $y := $xs[pos: 1];
    change $m[pos: 1] to 7;
;MAIN
