# A function that can fail, its failures named, and `otherwise` handling them.
record Locker:
    owner: string.utf8;
    items: std::list[string.utf8];
end of record Locker

procedure stash [lent.mutable $l: Locker, $item: string.utf8] -> boolean or fail[full, bad_item: string.utf8]:
    if std::list.length(lend $l.items) at_least 2:
        fail full;
    end of if
    if $item equal_to string.empty:
        fail bad_item["(empty)"];
    end of if
    std::list.append(lend.mutable $l.items, value: give $item);
    return true;
end of procedure stash

function parse_digit [$t: string.utf8] -> integer.signed.32_bit or fail[not_a_digit]:
    if $t equal_to "7":
        return 7;
    end of if
    fail not_a_digit;
end of function parse_digit

function double_digit [$t: string.utf8] -> integer.signed.32_bit or fail[not_a_digit]:
    let $d := parse_digit(copy $t) otherwise fail;      # pass it up
    return $d times 2;
end of function double_digit

function save [lent $l: Locker] -> fail[disk_full]:
    if std::list.length(lend $l.items) greater_than 1:
        fail disk_full;
    end of if
end of function save

!MAIN:
    variable $mine := Locker[owner = "Tankun", items = std::list.empty];
    let $names := ["rope", string.empty, "helmet", "torch"];
    for $n in lend $names:
        let $ok := stash(lend.mutable $mine, copy $n) otherwise:
            !case full:
                std::print.stored("full, stopping", newline); break;
            !end of case full
            !case bad_item $why:
                std::print.stored("bad item ", $why, newline); continue;
            !end of case bad_item
        end of otherwise;
        std::print.stored("stored ", $n, newline);
    end of for
    let $a := double_digit("7") otherwise 0;
    let $b := double_digit("x") otherwise 0;
    std::print.shortest($a, " ", $b, newline);
    save(lend $mine) otherwise:
        case disk_full:
            std::print.stored("could not save", newline);
        end of case disk_full
    end of otherwise;
    let $c := parse_digit("x") otherwise std::exit(code: 3);
    std::print.stored("unreachable", newline);
!end of MAIN
