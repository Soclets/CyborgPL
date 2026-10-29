# Into ASCII fails `not_ascii` unless every character is ASCII (owner,
# 2026-09-30: "`otherwise` exists"); a `char.byte` never fails. "hello"
# stays (5 bytes), "héllo" becomes "no" (2), `é` becomes "x" (1), and `a`
# stays (1): 5 x 16 + 2 x 4 + 1 + 1 = 90.
%extern.C proc exit [let $code: i32];
MAIN:
    let $plain: str.utf8 := "hello";
    let $accented: str.utf8 := "héllo";
    let $e: char.%point := `é`;
    let $k: char.byte := `a`;
    let $a := to.str.ascii(lend $plain) ?? "?";
    let $b := to.str.ascii(lend $accented) ?? "no";
    let $c := to.str.ascii($e) ?? "x";
    let $d := to.str.ascii($k);
    let $n: u64 := std::str.bytes(lend $a) x 16 + std::str.bytes(lend $b) x 4 + std::str.bytes(lend $c) + std::str.bytes(lend $d);
    no_check my.exit(to.i32($n));
;MAIN
