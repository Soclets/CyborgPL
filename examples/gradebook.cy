# A small gradebook: structs, a failure for no scores, and a proven division.
pub const $passing := 60;

struct Student {
    name: str.utf8,
    scores: std::list[i32]
}

fn average [lent $s: Student] -> i32 or fail[no_scores] {
    if std::list.length(lend $s.scores) equal_to 0 { fail no_scores; }
    let $total := 0;
    for $x in lend $s.scores { change $total = $total + $x; }
    check return $total div.floor std::list.length(lend $s.scores);     # the if above proves the divisor is not 0
}

fn record [lent.mut $s: Student, $score: i32] -> fail[out_of_range: i32] {
    if $score < 0 or $score > 100 { fail out_of_range[$score]; }
    std::list.append(lend.mut $s.scores, value: $score);
}

fn grade [lent $s: Student] -> str.utf8 {
    let $avg := average(lend $s) otherwise return to.stored.str.utf8($passing) concat "? no scores";
    if $avg at_least $passing { return "pass (" concat to.stored.str.utf8($avg) concat ")"; }
    return "fail (" concat to.stored.str.utf8($avg) concat ")";
}

MAIN {
    let $a := Student[name = "Ann", scores = std::list.empty];
    let $b := Student[name = "Bo", scores = [40, 55]];
    let $marks := [72, 130, 88];

    for $m in lend $marks {
        record(lend.mut $a, $m) otherwise {
            out_of_range $bad { std::print.useful("skipping ", $bad, newln); }
        };
    }
    record(lend.mut $b, 65) otherwise std::exit(1);

    let $students := [give $a, give $b, Student[name = "Cy", scores = std::list.empty]];
    let $passed := 0;
    for $s in lend $students {
        std::print.stored($s.name, ": ", grade(lend $s), newln);
        let $avg := average(lend $s) otherwise 0;
        if $avg at_least $passing { change $passed = $passed + 1; }
    }
    std::print.useful($passed, " of ", std::list.length(lend $students), " passed", newln);
}
