# `coll.handler` (owner, 2026-10-13): named `otherwise` blocks. Each member holds the arms written
# after `otherwise` today, one per way the call can fail, and `otherwise half_failed.as_zero` reads them
# as if they were written at the call: an arm may `return`, `break`, `continue` or `fail` from where it
# is used, and a member that misses a way the call can fail is refused there.
fn half ['n': i32] -> i32 or fail[odd, negative: i32] {
    if 'n' < 0 { fail negative['n']; }
    if ('n' div.floor 2) x 2 not_equal 'n' { fail odd; }
    return 'n' div.floor 2;
}

coll.handler half_failed {
    half_failed.as_zero {
        odd { return 0; }
        negative 'v' { std::print.useful["negative: ", 'v', newln]; return 0; }
    }
    half_failed.skip {
        odd { continue; }
        negative 'v' { continue; }
    }
    half_failed.stop {
        odd { break; }
        negative 'v' { break; }
    }
}

fn half_or_zero ['n': i32] -> i32 {
    let 'h' := half['n'] otherwise half_failed.as_zero;
    return 'h';
}

fn sum_of_halves [lent 'xs': list[i32], 'until_bad': bool] -> i32 {
    let 'sum' := 0;
    for 'x' in lend 'xs' {
        if 'until_bad' {
            let 'h' := half['x'] otherwise half_failed.stop;
            change 'sum' = 'sum' + 'h';
        } else {
            let 'h' := half['x'] otherwise half_failed.skip;
            change 'sum' = 'sum' + 'h';
        }
    }
    return 'sum';
}

MAIN {
    std::print.useful[half_or_zero[10], " ", half_or_zero[7], newln];
    std::print.useful[half_or_zero[-4], newln];
    let 'xs': list[i32] = [2, 4, 5, 8, -2, 10];
    std::print.useful[sum_of_halves[lend 'xs', false], " ", sum_of_halves[lend 'xs', true], newln];
}
