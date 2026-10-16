# A quoted name may hold symbols (owner, 2026-10-16): a const, parameters
# and locals named with `%`, `+`, `?` and `"` reach LLVM and the debug
# info as they are. 1 + 2 + 5 is 8.
extern.C fn exit ['code': int32];
const 'rate%' := 5;
fn add ['a+b': int32, 'c?': int32] -> int32 { return 'a+b' + 'c?'; }
MAIN {
    let 'x"y': int32 = add[1, 2] + 'rate%';
    no_check my.exit['x"y'];
}
