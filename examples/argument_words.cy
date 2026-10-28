# Every argument after the first says its role (owner, 2026-10-14): the first, the thing worked on, is bare;
# each other starts with a word. The list is `spellings.cyr`'s `calls`.
fn add [let $a: i32, let $b: i32] -> i32:
    ret $a + $b;
;fn add

MAIN:
    var $xs: std::ls[i32] = [];
    std::ls.append(lend.mut $xs, value: 3);
    std::ls.append(lend.mut $xs, value: 4);
    std::print(std::ls.len(lend $xs), newln);

    var $phones: std::map[str.utf8, i32] = std::map.empty;
    std::map.set(lend.mut $phones, key: "ann", value: 5551);
    std::map.set(lend.mut $phones, key: "bob", value: 5552);
    let $n := std::map.get(lend $phones, key: "ann") ?? 0;
    std::print($n, " ", std::map.has(lend $phones, key: "bob"), newln);
    let $gone := std::map.remove(lend.mut $phones, key: "bob") ?? 0;
    std::print($gone, " ", std::map.has(lend $phones, key: "bob"), newln);

    let $s := "red,green,blue";
    std::print(std::str.contains(lend $s, part: "green"), " ", std::str.starts_with(lend $s, part: "red"), " ", std::str.ends_with(lend $s, part: "blue"), newln);
    let $parts := std::str.split(lend $s, at: ",");
    let $joined := std::str.join(lend $parts, between: " and ");
    std::print($joined, newln);
    std::print(std::str.replace(lend $s, from: ",", to: "; "), newln);
;MAIN
