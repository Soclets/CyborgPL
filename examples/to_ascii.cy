# Text into ASCII fails `not_ascii` unless every character is ASCII (owner, 2026-09-30: "`otherwise`
# exists"). Into utf16 or utf32 it never fails, and from ASCII text there is nothing to fail.
!MAIN:
    let $plain: str.utf8 = "hello";
    let $accented: str.utf8 = "héllo";
    let $a := to.str.ascii($plain) otherwise "?";
    let $b := to.str.ascii($accented) otherwise "not ascii";
    std::print.stored($a, newln, $b, newln);
    let $s: str.ascii = "abc";
    let $c := to.str.ascii($s);
    let $w := to.str.%utf16($accented);
    std::print.stored($c, " ", $w, newln);
!end of MAIN
