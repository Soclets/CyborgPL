# A kit's `pub coll.handler` (owner, 2026-10-23: "Should `pub` handlers be a thing? (the suggestor AI) No shit"),
# its member taking arguments (owner: "Yes"): another kit uses it through its path,
# `otherwise halves::half_failed.or_default(default: 0, label: "…")`.
pub fn half [$n: i32] -> i32 or fail[odd, negative: i32] {
    if $n < 0 { fail negative[$n]; }
    if ($n div.floor 2) x 2 not_equal $n { fail odd; }
    return $n div.floor 2;
}

pub coll.handler half_failed {
    half_failed.or_default [$default: i32, $label: str.utf8] {
        odd { std::print.useful($label, ": odd, using ", $default, newln); return $default; }
        negative $v { std::print.useful($label, ": ", $v, " is negative, using ", $default, newln); return $default; }
    }
}
