# Map keys (decided 2026-09-22): plain values, text, chars, structs and enums by structure —
# or a type's own `op equal_to ` and `fn hash` (it satisfies `Key`), with "equal keys hash the same"
# a check site the compiler must prove.
sig Key <T>:
    op == [lent $a: <T>, lent $b: <T>] -> bool;  fn hash [lent $v: <T>] -> u64;
;sig Key

rec Point:
    var x: i32;
    var y: i32;
;rec Point                      # structural: fields decide

rec User:
    var name: str.utf8;
;rec User                           # its own rule: names compare lowercased
op == [lent $a: User, lent $b: User] -> bool:
    ret std::str.lower.full(lend $a.name) == std::str.lower.full(lend $b.name);
;op ==
fn hash [lent $v: User] -> u64:
    let $low := std::str.lower.full(lend $v.name); ret std::str.bytes(lend $low);
;fn hash
User satisfies Key;

MAIN:
    var $grid: std::map[Point, str.utf8] = std::map.empty;
    std::map.set(lend.mut $grid, key: Point[x = 1, y = 2], value: "tree");
    let $at := std::map.get(lend $grid, key: Point[x = 1, y = 2]) ?? "nothing";
    std::println($at);
    var $users: std::map[User, i32] = std::map.empty;
    std::map.set(lend.mut $users, key: User[name = "Ann"], value: 1);
    let $n := std::map.get(lend $users, key: User[name = "ANN"]) ?? 0;     # same key by User's own equal_to 
    std::println($n);
;MAIN
