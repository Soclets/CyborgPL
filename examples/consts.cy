# Constants, worked out while building, `pub` and not, used in MAIN.
bare.kit_prefix.never [fn, struct];

pub const 'max' := 8;
const 'pi': f64 = 3.141_592_653_589_793;
const 'twice_max' := 'max' x 2;
const 'greeting' := "hello " concat "world";
pub struct P { x: i32 }
pub fn area ['r': f64] -> f64 { return 'pi' x 'r' x 'r'; }
MAIN {
    std::print.useful['max', " ", 'twice_max', " ", 'greeting', " ", area[2.0], newln];
    let 's' := 'greeting';
    let 'p' := P[x = 'max'];
    for 'i' [from: 1, to: 'max', by: 4] { std::print.useful['i', " "]; }
    std::print.stored[newln];
}
