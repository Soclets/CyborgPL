extern.C fn exit ['code': i32];
enum Shape {
    dot,
    line: i32,
    box: i32
}
fn measure ['s': Shape] -> i32 {
    match 's' {
        dot        { return 1; }
        line 'n'   { return 'n' plus 10; }
        box 'n'    { return 'n' plus 100; }
    }
}
MAIN {
    let 'a' := Shape.dot;
    let 'b' := Shape.line[5];
    let 'c' := Shape.box[7];
    no_check my.exit[measure[give 'a'] plus measure[give 'b'] plus measure[give 'c']];
}
