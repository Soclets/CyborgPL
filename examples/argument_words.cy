# Every argument after the first says its role (owner, 2026-10-14): the first, the thing worked on, is bare;
# each other starts with a word. The list is `spellings.cyr`'s `calls`.
fn add [$a: i32, $b: i32] -> i32 { return $a + $b; }

MAIN {
    let $xs: std::list[i32] = [];
    std::list.append(lend.mut $xs, value: 3);
    std::list.append(lend.mut $xs, value: 4);
    std::print.useful(std::list.length(lend $xs), newln);

    let $phones: std::map[str.utf8, i32] = std::map.empty;
    std::map.set(lend.mut $phones, key: "ann", value: 5551);
    std::map.set(lend.mut $phones, key: "bob", value: 5552);
    let $n := std::map.get(lend $phones, key: "ann") otherwise 0;
    std::print.useful($n, " ", std::map.has(lend $phones, key: "bob"), newln);
    let $gone := std::map.remove(lend.mut $phones, key: "bob") otherwise 0;
    std::print.useful($gone, " ", std::map.has(lend $phones, key: "bob"), newln);

    let $s := "red,green,blue";
    std::print.useful(std::str.contains(lend $s, part: "green"), " ", std::str.starts_with(lend $s, part: "red"), " ", std::str.ends_with(lend $s, part: "blue"), newln);
    let $parts := std::str.split(lend $s, at: ",");
    let $joined := std::str.join(lend $parts, between: " and ");
    std::print.useful($joined, newln);
    std::print.useful(std::str.replace(lend $s, from: ",", to: "; "), newln);
}
