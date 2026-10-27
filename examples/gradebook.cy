# A small gradebook: structs, a failure for no scores, and a proven division.
pub const $passing := 60;

rec Student:
    name: str.utf8;
    scores: std::ls[int.sign.32];
end of rec Student

fn average [lent $s: Student] -> int.sign.32 or fail[no_scores]:
    if std::ls.len(lend $s.scores) eq 0:
        fail no_scores;
    end of if
    var $total := 0;
    for $x in lend $s.scores:
        change $total = $total plus $x;
    end of for
    check ret $total div.floor std::ls.len(lend $s.scores);     # the if above proves the divisor is not 0
end of fn average

proc add_score [lent.mut $s: Student, $score: int.sign.32] -> fail[out_of_range: int.sign.32]:
    if $score less_than 0 or $score greater_than 100:
        fail out_of_range[$score];
    end of if
    std::ls.append(lend.mut $s.scores, value: $score);
end of proc add_score

fn grade [lent $s: Student] -> str.utf8:
    let $avg := average(lend $s) otherwise ret to.stored.str.utf8($passing) concat "? no scores";
    if $avg at_least $passing:
        ret "pass (" concat to.stored.str.utf8($avg) concat ")";
    end of if
    ret "fail (" concat to.stored.str.utf8($avg) concat ")";
end of fn grade

!MAIN:
    var $a := Student[name = "Ann", scores = std::ls.empty];
    var $b := Student[name = "Bo", scores = [40, 55]];
    let $marks := [72, 130, 88];

    for $m in lend $marks:
        add_score(lend.mut $a, $m)-
        otherwise:
            case out_of_range $bad:
                std::print.shortest("skipping ", $bad, newln);
            end of case out_of_range
        end of otherwise;
    end of for
    add_score(lend.mut $b, 65) otherwise std::exit(code: 1);

    let $students := [give $a, give $b, Student[name = "Cy", scores = std::ls.empty]];
    var $passed := 0;
    for $s in lend $students:
        std::print.stored($s.name, ": ", grade(lend $s), newln);
        let $avg := average(lend $s) otherwise 0;
        if $avg at_least $passing:
            change $passed = $passed plus 1;
        end of if
    end of for
    std::print.shortest($passed, " of ", std::ls.len(lend $students), " passed", newln);
!end of MAIN
