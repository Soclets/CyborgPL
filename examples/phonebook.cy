# map[K, V] — a language collection beside list and array (decided 2026-09-22).
# A missing key is a failure, not a check site: `otherwise` says what happens.
MAIN:
    var $phones: std::map[str.utf8, i32] = std::map["ann" = 5551, "bob" = 5552];   # a literal: key = value
    std::map.set(lend.mut $phones, key: "ann", value: 5553);                          # same key: replaced
    std::println(std::map.count(lend $phones), " entries");
    let $a := std::map.get(lend $phones, key: "ann") ?? 0;
    let $c := std::map.get(lend $phones, key: "cat") ?? 0;             # missing → 0
    std::println("ann ", $a, ", cat ", $c);
    if std::map.has(lend $phones, key: "bob"):
        std::println("bob is in");
    ;if
    let $gone := std::map.remove(lend.mut $phones, key: "bob") ?? 0;   # hands the value back
    std::println("removed ", $gone, ", ", std::map.count(lend $phones), " left");
    std::map.remove(lend.mut $phones, key: "nobody")-
    otherwise:
        case missing:
            std::println("nobody to remove");
        ;case missing
    ;otherwise;
    std::map.set(lend.mut $phones, key: "cat", value: 5554);
    for $k in std::map.keys(lend $phones):
        std::print($k, " ");
    ;for        # insertion order: ann cat
    std::println.stored();
    for $e in std::map.pairs(lend $phones):
        std::print($e.key, "=", $e.value, " ");
    ;for
    std::println.stored();
    var $fast: std::map.unordered[str.utf8, i32] = std::map.empty;                     # chosen: never walked
    std::map.set(lend.mut $fast, key: "x", value: 1);
    std::println(std::map.has(lend $fast, key: "x"));
    let $who := std::map.get(lend $phones, key: "zed")-
    otherwise:
        case missing:
            std::println("no zed"); std::exit(code: 0);
        ;case missing
    ;otherwise;
;MAIN
