# Score handling: small functions, `if` / `else if`, and clamping a value into a range.
fn is_even [$n: i32] -> bool {
    return ($n mod.floor 2) equal_to 0;
}

fn clamp [$n: i32, $low: i32, $high: i32] -> i32 {
    if $n < $low {
        return $low;
    } else if $n > $high {
        return $high;
    } else {
        return $n;
    }
}

MAIN {
    let $scores := [72, 95, 38, 100, 61, 87];

    let $count := 0;
    let $total := 0;
    let $evens := 0;
    let $best := 0;

    for $score in lend $scores {
        change $count = $count + 1;
        change $total = $total + $score;
        if is_even($score) {
            change $evens = $evens + 1;
        }
        if $score > $best {
            change $best = $score;
        }
    }

    let $average := $total div.floor $count;
    let $curved := clamp($average + 10, 0, 100);

    std::print.useful("scored ", $count, " tests", newln);
    std::print.useful("total ", $total, ", best ", $best, newln);
    std::print.useful("even scores: ", $evens, newln);
    std::print.useful("average ", $average, ", curved ", $curved, newln);

    if $average at_least 90 {
        std::print.useful("grade A", newln);
    } else if $average at_least 75 {
        std::print.useful("grade B", newln);
    } else {
        std::print.useful("keep going", newln);
    }
}
