# Map keys (decided 2026-09-22): plain values, text, chars, structs and enums by structure —
# or a type's own `op equal_to ` and `fn hash` (it satisfies `Key`), with "equal keys hash the same"
# a check site the compiler must prove.
interface Key <T> { op equal_to ['a': <T>, 'b': <T>] -> bool;  fn hash [lent 'v': <T>] -> u64; }

struct Point { x: i32, y: i32 }                      # structural: fields decide

struct User { name: str.utf8 }                           # its own rule: names compare lowercased
op equal_to ['a': User, 'b': User] -> bool { return std::str.lower.full[lend 'a'.name] equal_to std::str.lower.full[lend 'b'.name]; }
fn hash [lent 'v': User] -> u64 { let 'low' := std::str.lower.full[lend 'v'.name]; return std::str.bytes[lend 'low']; }
User satisfies Key;

MAIN {
    let mut 'grid': std::map[Point, str.utf8] = std::map.empty;
    std::map.set[lend.mut 'grid', key: Point[x = 1, y = 2], value: "tree"];
    let 'at' := std::map.get[lend 'grid', key: Point[x = 1, y = 2]] otherwise "nothing";
    std::print.useful['at', newln];
    let mut 'users': std::map[User, i32] = std::map.empty;
    std::map.set[lend.mut 'users', key: User[name = "Ann"], value: 1];
    let 'n' := std::map.get[lend 'users', key: User[name = "ANN"]] otherwise 0;     # same key by User's own equal_to 
    std::print.useful['n', newln];
}
