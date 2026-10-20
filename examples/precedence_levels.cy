# A kit gives its operators a precedence (owner, 2026-10-03): `precedence.low_first` counts level 1
# as the loosest (good practice), each level says how a chain reads, and a language operator in a
# level pins it where that operator is. std's `pow` binds above a negative, right pair first.
struct Vec { x: f64, y: f64 }

precedence.low_first {
    level 1 { equal_to, ~= } refused
    level 2 { + } left_first
    level 3 { times, dot } left_first
    level 4 { scale } left_first
}

op dot ['a': Vec, 'b': Vec] -> f64 {
    return ('a'.x x 'b'.x) + ('a'.y x 'b'.y);
}

op scale ['a': Vec, 'k': f64] -> Vec {
    return Vec[x = 'a'.x x 'k', y = 'a'.y x 'k'];
}

op ~= ['a': f64, 'b': f64] -> bool {
    return ('a' - 'b') < 0.000001 and ('b' - 'a') < 0.000001;
}

MAIN {
    let 'u' := Vec[x = 1.0, y = 2.0];
    let 'v' := Vec[x = 3.0, y = 4.0];
    let 'd' := 'u' dot 'v' + 1.0;          # ('u' dot 'v') plus 1.0
    let 'w' := 'u' scale 2.0 dot 'v';         # ('u' scale 2.0) dot 'v'
    let 'ok' := 'd' ~= 12.0;
    std::print.useful['d', " ", 'w', " ", 'ok', newln];
    std::print.useful[2 xx 3 + 1, " ", 2 xx 3 xx 2, " ", -2 xx 2, " ", 2 x 3 xx 2, newln];
}
