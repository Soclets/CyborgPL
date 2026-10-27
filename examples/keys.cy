# Map keys (decided 2026-09-22): plain values, text, chars, structs and enums by structure —
# or a type's own `op equal_to ` and `fn hash` (it satisfies `Key`), with "equal keys hash the same"
# a check site the compiler must prove.
interface Key <T>:
    op eq [$a: <T>, $b: <T>] -> bool;  fn hash [lent $v: <T>] -> int.unsign.64;
end of interface Key

rec Point:
    x: int.sign.32;
    y: int.sign.32;
end of rec Point                      # structural: fields decide

rec User:
    name: str.utf8;
end of rec User                           # its own rule: names compare lowercased
op eq [$a: User, $b: User] -> bool:
    ret std::str.lower.full(lend $a.name) eq std::str.lower.full(lend $b.name);
end of op eq
fn hash [lent $v: User] -> int.unsign.64:
    let $low := std::str.lower.full(lend $v.name); ret std::str.bytes(lend $low);
end of fn hash
User satisfies Key;

!MAIN:
    var $grid: std::map[Point, str.utf8] = std::map.empty;
    std::map.set(lend.mut $grid, key: Point[x = 1, y = 2], value: "tree");
    let $at := std::map.get(lend $grid, key: Point[x = 1, y = 2]) otherwise "nothing";
    std::print.shortest($at, newln);
    var $users: std::map[User, int.sign.32] = std::map.empty;
    std::map.set(lend.mut $users, key: User[name = "Ann"], value: 1);
    let $n := std::map.get(lend $users, key: User[name = "ANN"]) otherwise 0;     # same key by User's own equal_to 
    std::print.shortest($n, newln);
!end of MAIN
