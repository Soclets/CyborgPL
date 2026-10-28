# A kit's `pub coll.handler` (owner, 2026-10-23: "Should `pub` handlers be a thing? (the suggestor AI) No shit"),
# its member taking arguments (owner: "Yes"): another kit uses it through its path,
# `otherwise halves::half_failed.or_default(default: 0, label: "…")`.
pub fn half [let $n: i32] -> i32 or fail[odd, negative: i32]:
    if $n < 0:
        fail negative[$n];
    ;if
    if ($n div.floor 2) x 2 != $n:
        fail odd;
    ;if
    ret $n div.floor 2;
;fn half

pub coll.handler half_failed:
    member half_failed.or_default [let $default: i32, let $label: str.utf8]:
        case odd:
            std::println($label, ": odd, using ", $default); ret $default;
        ;case odd
        case negative $v:
            std::println($label, ": ", $v, " is negative, using ", $default); ret $default;
        ;case negative
    ;member half_failed.or_default
;coll.handler half_failed
