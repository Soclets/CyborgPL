# A kit's `pub coll.handler` (owner, 2026-10-23: "Should `pub` handlers be a thing? (the suggestor AI) No shit"),
# its member taking arguments (owner: "Yes"): another kit uses it through its path,
# `otherwise halves::half_failed.or_default(default: 0, label: "…")`.
pub fn half [$n: int.sign.32] -> int.sign.32 or fail[odd, negative: int.sign.32]:
    if $n less_than 0:
        fail negative[$n];
    end of if
    if ($n div.floor 2) times 2 not_eq $n:
        fail odd;
    end of if
    ret $n div.floor 2;
end of fn half

pub coll.handler half_failed:
    member half_failed.or_default [$default: int.sign.32, $label: str.utf8]:
        !case odd:
            std::print.shortest($label, ": odd, using ", $default, newln); ret $default;
        !end of case odd
        !case negative $v:
            std::print.shortest($label, ": ", $v, " is negative, using ", $default, newln); ret $default;
        !end of case negative
    end of member half_failed.or_default
end of coll.handler half_failed
