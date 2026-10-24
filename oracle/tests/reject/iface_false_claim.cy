interface Ordered <T> { op < [$a: <T>, $b: <T>] -> bool; }
struct V { x: i32 }
V satisfies Ordered;
MAIN {
}
