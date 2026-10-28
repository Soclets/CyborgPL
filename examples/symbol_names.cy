# A quoted name may hold any printable character (owner, 2026-10-16: "Yeah, allow it", then "Allow 1, 2, and
# 4"): symbols, a `.`, single spaces between words, and a `'` written `\'` (`\\` for a `\`). Good naming
# depends on what you would use (style.md 17).
MAIN:
    let $'80%' := 80;
    let $'ready?' := true;
    let $'v1.2' := 12;
    let $'total due' := $'v1.2' + 3;
    let $'it\'s' := "yes";
    std::println($'80%', " ", $'ready?', " ", $'v1.2', " ", $'total due', " ", $'it\'s');
;MAIN
