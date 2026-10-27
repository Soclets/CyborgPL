# A quoted name may hold any printable character (owner, 2026-10-16: "Yeah, allow it", then "Allow 1, 2, and
# 4"): symbols, a `.`, single spaces between words, and a `'` written `\'` (`\\` for a `\`). Good naming
# depends on what you would use (style.md 17).
!MAIN:
    let $'80%' := 80;
    let $'ready?' := true;
    let $'v1.2' := 12;
    let $'total due' := $'v1.2' plus 3;
    let $'it\'s' := "yes";
    std::print.shortest($'80%', " ", $'ready?', " ", $'v1.2', " ", $'total due', " ", $'it\'s', newln);
!end of MAIN
