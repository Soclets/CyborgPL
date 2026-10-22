# Consts, worked out once at compile time to the literal each is: integers,
# text, f64 and powers, used in a struct, a call, a loop's bound.


extern.C fn exit ['code': i32];
extern.C fn strlen ['s': ptr[u8]] -> u64;
pub const 'max' := 8;
const 'pi': f64 = 3.141_592_653_589_793;
const 'twice_max' := 'max' times 2;
const 'greeting' := "hello " concat "world";
const 'big': i64 = 'max' pow 20;
pub struct P { x: i32 }
pub fn area ['r': f64] -> f64 { return 'pi' times 'r' times 'r'; }
MAIN {
    let mut 'got': i32 = 0;
    if 'twice_max' equal_to 16 { change 'got' = 'got' plus 1; }
    let 's' := 'greeting';
    no_check if strlen[lend 's'] equal_to 11 { change 'got' = 'got' plus 2; }
    let 'p' := P[x = 'max'];
    if 'p'.x equal_to 8 { change 'got' = 'got' plus 4; }
    let 'a' := area[2.0];
    if 'a' > 12.56 and 'a' < 12.57 { change 'got' = 'got' plus 8; }
    let mut 'n': i32 = 0;
    for 'i' [from: 1, to: 'max', by: 4] { change 'n' = 'n' plus 1; }
    if 'n' equal_to 2 { change 'got' = 'got' plus 16; }
    if 'big' equal_to 1152921504606846976 { change 'got' = 'got' plus 32; }
    no_check my.exit['got'];
}
