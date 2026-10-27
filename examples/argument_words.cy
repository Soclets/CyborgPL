# Every argument after the first says its role (owner, 2026-10-14): the first, the thing worked on, is bare;
# each other starts with a word. The list is `spellings.cyr`'s `calls`.
function add [$a: integer.signed.32_bit, $b: integer.signed.32_bit] -> integer.signed.32_bit:
    return $a plus $b;
end of function add

!MAIN:
    variable $xs: std::list[integer.signed.32_bit] = [];
    std::list.append(lend.mutable $xs, value: 3);
    std::list.append(lend.mutable $xs, value: 4);
    std::print.shortest(std::list.length(lend $xs), newline);

    variable $phones: std::map[string.utf8, integer.signed.32_bit] = std::map.empty;
    std::map.set(lend.mutable $phones, key: "ann", value: 5551);
    std::map.set(lend.mutable $phones, key: "bob", value: 5552);
    let $n := std::map.get(lend $phones, key: "ann") otherwise 0;
    std::print.shortest($n, " ", std::map.has(lend $phones, key: "bob"), newline);
    let $gone := std::map.remove(lend.mutable $phones, key: "bob") otherwise 0;
    std::print.shortest($gone, " ", std::map.has(lend $phones, key: "bob"), newline);

    let $s := "red,green,blue";
    std::print.shortest(std::string.contains(lend $s, part: "green"), " ", std::string.starts_with(lend $s, part: "red"), " ", std::string.ends_with(lend $s, part: "blue"), newline);
    let $parts := std::string.split(lend $s, at: ",");
    let $joined := std::string.join(lend $parts, between: " and ");
    std::print.shortest($joined, newline);
    std::print.shortest(std::string.replace(lend $s, from: ",", to: "; "), newline);
!end of MAIN
