# map[K, V] — a language collection beside list and array (decided 2026-09-22).
# A missing key is a failure, not a check site: `otherwise` says what happens.
MAIN:
    var $phones: std::map[str.utf8, i32] = std::map["ann" = 5551, "bob" = 5552];   # a literal: key = value
    std::map.set(lend.mut $phones, key: "ann", value: 5553);                          # same key: replaced
    std::print(std::map.count(lend $phones), " entries", newln);
    let $a := std::map.get(lend $phones, key: "ann") ?? 0;
    let $c := std::map.get(lend $phones, key: "cat") ?? 0;             # missing → 0
    std::print("ann ", $a, ", cat ", $c, newln);
    if std::map.has(lend $phones, key: "bob"):
        std::print("bob is in", newln);
    ;if
    let $gone := std::map.remove(lend.mut $phones, key: "bob") ?? 0;   # hands the value back
    std::print("removed ", $gone, ", ", std::map.count(lend $phones), " left", newln);
    std::map.remove(lend.mut $phones, key: "nobody")-
    otherwise:
        case missing:
            std::print("nobody to remove", newln);
        ;case missing
    ;otherwise;
    std::map.set(lend.mut $phones, key: "cat", value: 5554);
    for $k in std::map.keys(lend $phones):
        std::print($k, " ");
    ;for        # insertion order: ann cat
    std::print.stored(newln);
    for $e in std::map.pairs(lend $phones):
        std::print($e.key, "=", $e.value, " ");
    ;for
    std::print.stored(newln);
    var $fast: std::map.unordered[str.utf8, i32] = std::map.empty;                     # chosen: never walked
    std::map.set(lend.mut $fast, key: "x", value: 1);
    std::print(std::map.has(lend $fast, key: "x"), newln);
    let $who := std::map.get(lend $phones, key: "zed")-
    otherwise:
        case missing:
            std::print("no zed", newln); std::exit(code: 0);
        ;case missing
    ;otherwise;
;MAIN
