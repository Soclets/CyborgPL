# No bin or brain16 result is ever -0 (owner, 2026-10-07: "a"); each `.ieee`
# form, and `neg.ieee`, keeps IEEE's ("yea"). Each zero's text, "0" or "-0",
# tells its sign: `plus` (+0) adds 1 and `ieee` (-0) adds 2 when
# right, 0 when wrong. The oracle gives 56. `mod` and `pow` too
# (owner, 2026-10-07: "yea.").
# Values come from calls so nothing is folded, then once more as literals.
extern.C fn exit ['code': int32];
fn zero [] -> bin64 { return 0.0; }
fn two [] -> bin64 { return -2.0; }
fn three [] -> bin64 { return 3.0; }
fn tiny [] -> bin64 { return 0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001; }
fn sign ['q': bin64] -> int32 {
    let 't' := to.useful.str.utf8['q'];
    if str.bytes[lend 't'] equal_to 1 { return 1; }
    return 0;
}
fn plus ['q': bin64] -> int32 { return sign['q']; }
fn ieee ['q': bin64] -> int32 { return 2 times (1 minus sign['q']); }
MAIN {
    let 'z' := zero[];
    let 'n' := two[];
    let 't' := tiny[];
    let 'm': bin64 = negate.ieee 'z';
    let 'a' := plus['z' times 'n'] plus ieee['z' times.ieee 'n'];
    let 'b' := plus[-'z'] plus ieee[negate.ieee 'z'];
    let 'c' := plus['t' times (-'t')] plus ieee['t' times.ieee (-'t')];
    let 'd' := plus['z' div.near 'n'] plus ieee['z' div.near.ieee 'n'] plus plus['z' div.ceil 'n'] plus ieee['z' div.ceil.ieee 'n'];
    let 'e' := plus['m' plus 'm'] plus ieee['m' plus.ieee 'm'] plus plus['m' minus 'z'] plus ieee['m' minus.ieee 'z'];
    let 'f' := plus[3.0 mod.floor (-3.0)] plus plus['n' mod.floor 'n'] plus ieee['m'];
    let 'bz': brain16 = 0.0;
    let 'bc': brain16 = -3.0;
    let 'g' := plus[to.bin64['bz' times 'bc']] plus ieee[to.bin64['bz' times.ieee 'bc']] plus plus[to.bin64[-'bz']] plus ieee[to.bin64[negate.ieee 'bz']];
    let 'w': bin32 = to.bin32['t' times (-1.0)];
    let 'k': bin32 = to.bin32['m'];
    let 'h' := plus[to.bin64['w']] plus ieee[to.bin64['k']] plus plus[to.bin64[to.brain16['t' times (-1.0)]]];
    let 'vz': simd[bin64, count 2] = simd.fill[value 'z'];
    let 'vm': simd[bin64, count 2] = simd.fill[value 'm'];
    let 'vn': simd[bin64, count 2] = simd.fill[value 'n'];
    let 's' := plus[simd.max['vm']] plus plus[simd.min['vm']] plus plus[simd.sum['vm']] plus plus[simd.max['vz' times 'vn']] plus plus[simd.max['vz' times.ieee 'vn']];
    let 'r' := three[];
    let 'x' := plus[(-'r') mod.trunc 'r'] plus ieee[(-'r') mod.trunc.ieee 'r'] plus ieee['r' mod.floor.ieee (-'r')] plus plus[(-'t') pow 3] plus ieee[(-'t') pow.ieee 3] plus ieee[(-'t') std::pow.ieee 3];
    let 'lit' := plus[0.0 times (-2.0)] plus ieee[0.0 times.ieee (-2.0)] plus plus[-0.0] plus ieee[negate.ieee 0.0];
    no_check my.exit[to.int32['a' plus 'b' plus 'c' plus 'd' plus 'e' plus 'f' plus 'g' plus 'h' plus 's' plus 'x' plus 'lit']];
}
