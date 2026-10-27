# Map keys (decided 2026-09-22): plain values, text, chars, structs and enums by structure —
# or a type's own `op equal_to ` and `fn hash` (it satisfies `Key`), with "equal keys hash the same"
# a check site the compiler must prove.
interface Key <T>:
    operator equal_to [$a: <T>, $b: <T>] -> boolean;  function hash [lent $v: <T>] -> integer.unsigned.64_bit;
end of interface Key

record Point:
    x: integer.signed.32_bit;
    y: integer.signed.32_bit;
end of record Point                      # structural: fields decide

record User:
    name: string.utf8;
end of record User                           # its own rule: names compare lowercased
operator equal_to [$a: User, $b: User] -> boolean:
    return std::string.lowercase.full(lend $a.name) equal_to std::string.lowercase.full(lend $b.name);
end of operator equal_to
function hash [lent $v: User] -> integer.unsigned.64_bit:
    let $low := std::string.lowercase.full(lend $v.name); return std::string.bytes(lend $low);
end of function hash
User satisfies Key;

!MAIN:
    variable $grid: std::map[Point, string.utf8] = std::map.empty;
    std::map.set(lend.mutable $grid, key: Point[x = 1, y = 2], value: "tree");
    let $at := std::map.get(lend $grid, key: Point[x = 1, y = 2]) otherwise "nothing";
    std::print.shortest($at, newline);
    variable $users: std::map[User, integer.signed.32_bit] = std::map.empty;
    std::map.set(lend.mutable $users, key: User[name = "Ann"], value: 1);
    let $n := std::map.get(lend $users, key: User[name = "ANN"]) otherwise 0;     # same key by User's own equal_to 
    std::print.shortest($n, newline);
!end of MAIN
