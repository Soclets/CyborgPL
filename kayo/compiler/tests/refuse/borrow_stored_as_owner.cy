# refuse: here it would be a second owner of what it borrows
# A borrow stored into a variable that owns its value.
fn f [lent $a: std::ls[i8]] -> lent std::ls[i8]:
    ret lend $a;
;fn f
MAIN:
    var $s: std::ls[i8] = [1, 2];
    var $o: std::ls[i8] = [3,];
    change $o to f(lend $s);
;MAIN
