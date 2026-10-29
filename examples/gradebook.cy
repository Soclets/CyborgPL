# A small gradebook: structs, a failure for no scores, and a proven division.
pub const $passing := 60;

rec Student:
    var name: str.utf8;
    var scores: std::ls[i32];
;rec Student

fn average [lent $s: Student] -> i32 or fail[no_scores]:
    if std::ls.len(lend $s.scores) == 0:
        fail no_scores;
    ;if
    var $total = 0;
    for $x in lend $s.scores:
        change $total to $total + $x;
    ;for
    check ret $total div.floor std::ls.len(lend $s.scores);     # the if above proves the divisor is not 0
;fn average

proc add_score [lent.mut $s: Student, $score: i32] -> fail[out_of_range: i32]:
    if $score < 0 or $score > 100:
        fail out_of_range[$score];
    ;if
    std::ls.append(lend.mut $s.scores, value: $score);
;proc add_score

fn grade [lent $s: Student] -> str.utf8:
    let $avg := average(lend $s) otherwise ret to.stored.str.utf8($passing) concat "? no scores";
    if $avg >= $passing:
        ret "pass (" concat to.stored.str.utf8($avg) concat ")";
    ;if
    ret "fail (" concat to.stored.str.utf8($avg) concat ")";
;fn grade

MAIN:
    var $a = Student[name = "Ann", scores = std::ls.empty];
    var $b = Student[name = "Bo", scores = [40, 55]];
    let $marks := [72, 130, 88];

    for $m in lend $marks:
        add_score(lend.mut $a, $m)-
        otherwise:
            case out_of_range $bad:
                std::print("skipping ", $bad, newln);
            ;case out_of_range
        ;otherwise;
    ;for
    add_score(lend.mut $b, 65) ?? std::exit(code: 1);

    let $students := [give $a, give $b, Student[name = "Cy", scores = std::ls.empty]];
    var $passed = 0;
    for $s in lend $students:
        std::print.stored($s.name, ": ", grade(lend $s), newln);
        let $avg := average(lend $s) ?? 0;
        if $avg >= $passing:
            change $passed to $passed + 1;
        ;if
    ;for
    std::print($passed, " of ", std::ls.len(lend $students), " passed", newln);
;MAIN
