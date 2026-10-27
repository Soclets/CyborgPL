# Text into ASCII fails `not_ascii` unless every character is ASCII (owner, 2026-09-30: "`otherwise`
# exists"). Into utf16 or utf32 it never fails, and from ASCII text there is nothing to fail.
!MAIN:
    let $plain: string.utf8 = "hello";
    let $accented: string.utf8 = "héllo";
    let $a := to.string.ascii($plain) otherwise "?";
    let $b := to.string.ascii($accented) otherwise "not ascii";
    std::print.stored($a, newline, $b, newline);
    let $s: string.ascii = "abc";
    let $c := to.string.ascii($s);
    let $w := to.string.%utf16($accented);
    std::print.stored($c, " ", $w, newline);
!end of MAIN
