# A function that can fail, its failures named, and `otherwise` handling them.
rec Locker:
    var owner: str.utf8;
    var items: std::ls[str.utf8];
end of rec Locker

proc stash [lent.mut $l: Locker, let $item: str.utf8] -> bool or fail[full, bad_item: str.utf8]:
    if std::ls.len(lend $l.items) at_least 2:
        fail full;
    end of if
    if $item eq str.empty:
        fail bad_item["(empty)"];
    end of if
    std::ls.append(lend.mut $l.items, value: give $item);
    ret true;
end of proc stash

fn parse_digit [let $t: str.utf8] -> int.sign.32 or fail[not_a_digit]:
    if $t eq "7":
        ret 7;
    end of if
    fail not_a_digit;
end of fn parse_digit

fn double_digit [let $t: str.utf8] -> int.sign.32 or fail[not_a_digit]:
    let $d := parse_digit(copy $t) otherwise fail;      # pass it up
    ret $d times 2;
end of fn double_digit

fn save [lent $l: Locker] -> fail[disk_full]:
    if std::ls.len(lend $l.items) greater_than 1:
        fail disk_full;
    end of if
end of fn save

!MAIN:
    var $mine = Locker[owner = "Tankun", items = std::ls.empty];
    let $names := ["rope", str.empty, "helmet", "torch"];
    for $n in lend $names:
        let $ok := stash(lend.mut $mine, copy $n)-
        otherwise:
            !case full:
                std::print.stored("full, stopping", newln); break;
            !end of case full
            !case bad_item $why:
                std::print.stored("bad item ", $why, newln); continue;
            !end of case bad_item
        end of otherwise;
        std::print.stored("stored ", $n, newln);
    end of for
    let $a := double_digit("7") otherwise 0;
    let $b := double_digit("x") otherwise 0;
    std::print.shortest($a, " ", $b, newln);
    save(lend $mine)-
    otherwise:
        case disk_full:
            std::print.stored("could not save", newln);
        end of case disk_full
    end of otherwise;
    let $c := parse_digit("x") otherwise std::exit(code: 3);
    std::print.stored("unreachable", newln);
!end of MAIN
