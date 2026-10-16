interface Printable <T> { fn show [lent 'x': <T>] -> str.utf8; }
struct V { x: int32 }
V satisfies Printable;
MAIN {
}
