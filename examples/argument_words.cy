# Every argument after the first says its role (owner, 2026-10-14): the first, the thing worked on, is bare;
# each other starts with a word. The list is `spellings.cyr`'s `calls`.
fn add [let $a: int.sign.32, let $b: int.sign.32] -> int.sign.32:
    ret $a plus $b;
end of fn add

!MAIN:
    var $xs: std::ls[int.sign.32] = [];
    std::ls.append(lend.mut $xs, value: 3);
    std::ls.append(lend.mut $xs, value: 4);
    std::print.shortest(std::ls.len(lend $xs), newln);

    var $phones: std::map[str.utf8, int.sign.32] = std::map.empty;
    std::map.set(lend.mut $phones, key: "ann", value: 5551);
    std::map.set(lend.mut $phones, key: "bob", value: 5552);
    let $n := std::map.get(lend $phones, key: "ann") otherwise 0;
    std::print.shortest($n, " ", std::map.has(lend $phones, key: "bob"), newln);
    let $gone := std::map.remove(lend.mut $phones, key: "bob") otherwise 0;
    std::print.shortest($gone, " ", std::map.has(lend $phones, key: "bob"), newln);

    let $s := "red,green,blue";
    std::print.shortest(std::str.contains(lend $s, part: "green"), " ", std::str.starts_with(lend $s, part: "red"), " ", std::str.ends_with(lend $s, part: "blue"), newln);
    let $parts := std::str.split(lend $s, at: ",");
    let $joined := std::str.join(lend $parts, between: " and ");
    std::print.shortest($joined, newln);
    std::print.shortest(std::str.replace(lend $s, from: ",", to: "; "), newln);
!end of MAIN
