# A name may be an old operator word: `lt` is Lithuanian's case here, as in std's casing. Only where an operator stands is
# `lt` refused, naming `<` (compiler claude, 2026-10-29).
union.tag Lang:
    en;
    lt;
;union.tag Lang
MAIN:
    let $l := Lang.lt;
    match $l:
        case en:
            std::print("en", newln);
        ;case en
        case lt:
            std::print("lt", newln);
        ;case lt
    ;match
;MAIN
