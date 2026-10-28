# A function that can fail, its failures named, and `otherwise` handling them.
rec Locker:
    var owner: str.utf8;
    var items: std::ls[str.utf8];
;rec Locker

proc stash [lent.mut $l: Locker, let $item: str.utf8] -> bool or fail[full, bad_item: str.utf8]:
    if std::ls.len(lend $l.items) >= 2:
        fail full;
    ;if
    if $item == str.empty:
        fail bad_item["(empty)"];
    ;if
    std::ls.append(lend.mut $l.items, value: give $item);
    ret true;
;proc stash

fn parse_digit [let $t: str.utf8] -> i32 or fail[not_a_digit]:
    if $t == "7":
        ret 7;
    ;if
    fail not_a_digit;
;fn parse_digit

fn double_digit [let $t: str.utf8] -> i32 or fail[not_a_digit]:
    let $d := parse_digit(copy $t)?;      # pass it up
    ret $d x 2;
;fn double_digit

fn save [lent $l: Locker] -> fail[disk_full]:
    if std::ls.len(lend $l.items) > 1:
        fail disk_full;
    ;if
;fn save

MAIN:
    var $mine = Locker[owner = "Tankun", items = std::ls.empty];
    let $names := ["rope", str.empty, "helmet", "torch"];
    for $n in lend $names:
        let $ok := stash(lend.mut $mine, copy $n)-
        otherwise:
            case full:
                std::print.stored("full, stopping", newln); break;
            ;case full
            case bad_item $why:
                std::print.stored("bad item ", $why, newln); continue;
            ;case bad_item
        ;otherwise;
        std::print.stored("stored ", $n, newln);
    ;for
    let $a := double_digit("7") ?? 0;
    let $b := double_digit("x") ?? 0;
    std::print($a, " ", $b, newln);
    save(lend $mine)-
    otherwise:
        case disk_full:
            std::print.stored("could not save", newln);
        ;case disk_full
    ;otherwise;
    let $c := parse_digit("x") ?? std::exit(code: 3);
    std::print.stored("unreachable", newln);
;MAIN
