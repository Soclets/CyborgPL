# A quoted name may hold any printable character (owner, 2026-10-16):
# symbols, a `.`, single spaces between words and a `'` written `\'`, in a
# const, parameters and locals, through LLVM and the debug info as they
# are. 'v1.2' is not 'v1' and a field. 1 + 2 + 5 + 12 + 3 is 23.
extern.C fn exit ['code': int32];
const 'rate%' := 5;
fn add ['a+b': int32, 'c?': int32] -> int32 { return 'a+b' + 'c?'; }
MAIN {
    let 'v1.2': int32 = 12;
    let 'it\'s up': int32 = 3;
    let 'x"y \\ z': int32 = add[1, 2] + 'rate%' + 'v1.2' + 'it\'s up';
    no_check my.exit['x"y \\ z'];
}
