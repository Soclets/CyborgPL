# A typed `d` const is folded in at each use (design claude's night_trains, 2026-10-25: "the const 'near_rate'
# is not folded into the generated code yet"). 0.12 x 100 is exactly 12: exit 12.
const $rate: d64 = 0.12;
MAIN {
    let $hundred: d64 = 100.0;
    let $r := to.i32($rate x $hundred) otherwise 0;
    std::exit(code: $r);
}
