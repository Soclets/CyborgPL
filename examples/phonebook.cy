# map[K, V] — a language collection beside list and array (decided 2026-09-22).
# A missing key is a failure, not a check site: `otherwise` says what happens.
MAIN {
    let mut 'phones': map[str.utf8, i32] = map["ann" = 5551, "bob" = 5552];   # a literal: key = value
    map.set[lend.mut 'phones', key "ann", value 5553];                          # same key: replaced
    std::print.useful[map.length[lend 'phones'], " entries", newln];
    let 'a' := map.get[lend 'phones', key "ann"] otherwise 0;
    let 'c' := map.get[lend 'phones', key "cat"] otherwise 0;             # missing → 0
    std::print.useful["ann ", 'a', ", cat ", 'c', newln];
    if map.has[lend 'phones', key "bob"] { std::print.useful["bob is in", newln]; }
    let 'gone' := map.remove[lend.mut 'phones', key "bob"] otherwise 0;   # hands the value back
    std::print.useful["removed ", 'gone', ", ", map.length[lend 'phones'], " left", newln];
    map.remove[lend.mut 'phones', key "nobody"] otherwise { missing { std::print.useful["nobody to remove", newln]; } };
    map.set[lend.mut 'phones', key "cat", value 5554];
    for 'k' in map.keys[lend 'phones'] { std::print.useful['k', " "]; }        # insertion order: ann cat
    std::print.stored[newln];
    for 'e' in map.pairs[lend 'phones'] { std::print.useful['e'.key, "=", 'e'.value, " "]; }
    std::print.stored[newln];
    let mut 'fast': map.unordered[str.utf8, i32] = map.empty;                     # chosen: never walked
    map.set[lend.mut 'fast', key "x", value 1];
    std::print.useful[map.has[lend 'fast', key "x"], newln];
    let 'who' := map.get[lend 'phones', key "zed"] otherwise { missing { std::print.useful["no zed", newln]; std::exit[0]; } };
}
