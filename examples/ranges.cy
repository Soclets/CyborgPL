# Range types (design/input.md, 2026-09-20): a type with bounds, provably within them.
struct Percent { value: uint8[min 0, max 100] }

fn scale ['p': Percent, 'total': int32[min 0, max 1000]] -> int32[min 0, max 1000] {
    return ('total' x to.int32['p'.value]) div.floor 100;
}

MAIN {
    let 'half' := Percent[value = 50];
    std::print.useful[scale[give 'half', 800], newln];
    let 'raw' := 250;
    if 'raw' at_least 0 and 'raw' at_most 100 {
        let 'ok': int32[min 0, max 100] = 'raw';                   # the if is the guardrail; a check site
        std::print.useful['ok', newln];
    } else {
        std::print.stored["250 is not a percentage", newln];
    }
    let 'r' := to.uint8[min 0, max 100]["42"] otherwise 0;          # into a range type: fails, does not check
    let 'bad' := to.uint8[min 0, max 100]["142"] otherwise 0;
    let 'huge' := to.uint8[min 0, max 100]["99999999999"] otherwise 0;   # not even a uint8: still out of range, not a crash
    std::print.useful['r', " ", 'bad', " ", 'huge', newln];
    let 'unit': bin64[min 0.0, max 1.0] = 0.25;
    std::print.useful['unit', newln];
}
