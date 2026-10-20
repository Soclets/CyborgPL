# std marks everything `bare.optional` (owner, 2026-10-06: "optional for all"): each of std's names
# may be written with `std::` or without it.

MAIN {
    let 's' := "a b c";
    let 'parts' := str.split[lend 's', at " "];
    let 'same' := std::str.split[lend 's', at " "];
    print.useful[list.length[lend 'parts'], " ", std::list.length[lend 'same'], newln];
    std::print.useful[round.half_up[2.5: f64], newln];
}
