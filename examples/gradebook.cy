# A small gradebook: structs, a failure for no scores, and a proven division.
public constant $passing := 60;

record Student:
    name: string.utf8;
    scores: std::list[integer.signed.32_bit];
end of record Student

function average [lent $s: Student] -> integer.signed.32_bit or fail[no_scores]:
    if std::list.length(lend $s.scores) equal_to 0:
        fail no_scores;
    end of if
    variable $total := 0;
    for $x in lend $s.scores:
        change $total = $total plus $x;
    end of for
    check return $total divide.floor std::list.length(lend $s.scores);     # the if above proves the divisor is not 0
end of function average

procedure add_score [lent.mutable $s: Student, $score: integer.signed.32_bit] -> fail[out_of_range: integer.signed.32_bit]:
    if $score less_than 0 or $score greater_than 100:
        fail out_of_range[$score];
    end of if
    std::list.append(lend.mutable $s.scores, value: $score);
end of procedure add_score

function grade [lent $s: Student] -> string.utf8:
    let $avg := average(lend $s) otherwise return to.stored.string.utf8($passing) concatenate "? no scores";
    if $avg at_least $passing:
        return "pass (" concatenate to.stored.string.utf8($avg) concatenate ")";
    end of if
    return "fail (" concatenate to.stored.string.utf8($avg) concatenate ")";
end of function grade

!MAIN:
    variable $a := Student[name = "Ann", scores = std::list.empty];
    variable $b := Student[name = "Bo", scores = [40, 55]];
    let $marks := [72, 130, 88];

    for $m in lend $marks:
        add_score(lend.mutable $a, $m) otherwise:
            case out_of_range $bad:
                std::print.shortest("skipping ", $bad, newline);
            end of case out_of_range
        end of otherwise;
    end of for
    add_score(lend.mutable $b, 65) otherwise std::exit(code: 1);

    let $students := [give $a, give $b, Student[name = "Cy", scores = std::list.empty]];
    variable $passed := 0;
    for $s in lend $students:
        std::print.stored($s.name, ": ", grade(lend $s), newline);
        let $avg := average(lend $s) otherwise 0;
        if $avg at_least $passing:
            change $passed = $passed plus 1;
        end of if
    end of for
    std::print.shortest($passed, " of ", std::list.length(lend $students), " passed", newline);
!end of MAIN
