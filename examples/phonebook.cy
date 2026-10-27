# map[K, V] — a language collection beside list and array (decided 2026-09-22).
# A missing key is a failure, not a check site: `otherwise` says what happens.
!MAIN:
    variable $phones: std::map[string.utf8, integer.signed.32_bit] = std::map["ann" = 5551, "bob" = 5552];   # a literal: key = value
    std::map.set(lend.mutable $phones, key: "ann", value: 5553);                          # same key: replaced
    std::print.shortest(std::map.count(lend $phones), " entries", newline);
    let $a := std::map.get(lend $phones, key: "ann") otherwise 0;
    let $c := std::map.get(lend $phones, key: "cat") otherwise 0;             # missing → 0
    std::print.shortest("ann ", $a, ", cat ", $c, newline);
    if std::map.has(lend $phones, key: "bob"):
        std::print.shortest("bob is in", newline);
    end of if
    let $gone := std::map.remove(lend.mutable $phones, key: "bob") otherwise 0;   # hands the value back
    std::print.shortest("removed ", $gone, ", ", std::map.count(lend $phones), " left", newline);
    std::map.remove(lend.mutable $phones, key: "nobody") otherwise:
        case missing:
            std::print.shortest("nobody to remove", newline);
        end of case missing
    end of otherwise;
    std::map.set(lend.mutable $phones, key: "cat", value: 5554);
    for $k in std::map.keys(lend $phones):
        std::print.shortest($k, " ");
    end of for        # insertion order: ann cat
    std::print.stored(newline);
    for $e in std::map.pairs(lend $phones):
        std::print.shortest($e.key, "=", $e.value, " ");
    end of for
    std::print.stored(newline);
    variable $fast: std::map.unordered[string.utf8, integer.signed.32_bit] = std::map.empty;                     # chosen: never walked
    std::map.set(lend.mutable $fast, key: "x", value: 1);
    std::print.shortest(std::map.has(lend $fast, key: "x"), newline);
    let $who := std::map.get(lend $phones, key: "zed") otherwise:
        case missing:
            std::print.shortest("no zed", newline); std::exit(code: 0);
        end of case missing
    end of otherwise;
!end of MAIN
