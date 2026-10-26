# A quoted name may hold any printable character (owner, 2026-10-16):
# symbols, a `.`, single spaces between words and a `'` written `\'`, in a
# const, parameters and locals, through LLVM and the debug info as they
# are. 'v1.2' is not 'v1' and a field. 1 + 2 + 5 + 12 + 3 is 23.
%extern.C fn exit [$code: i32];
const 'rate%' := 5;
fn add ['a+b': i32, 'c?': i32] -> i32 { return 'a+b' + 'c?'; }
MAIN {
    let 'v1.2': i32 = 12;
    let 'it\'s up': i32 = 3;
    let 'x"y \\ z': i32 = add(1, 2) + 'rate%' + 'v1.2' + 'it\'s up';
    no_check my.exit('x"y \\ z');
}
