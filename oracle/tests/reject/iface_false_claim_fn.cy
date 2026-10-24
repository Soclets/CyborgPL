interface Printable <T> { fn show [lent $x: <T>] -> str.utf8; }
struct V { x: i32 }
V satisfies Printable;
MAIN {
}
