# A const's whole number into a deci is its value (check claude, seed
# 48186): the conversion took the constant's raw bits as a deci, so 1234
# came out as 0.000…0001234 and back as nothing whole. 1234 mod 256 is 210.
extern.C fn exit ['code': int32];
const 'top': int64 = 1000;
MAIN {
    const 'c': int64 = 234;
    let 'd' := to.deci128['top'] plus to.deci128['c'];
    let 'n': int64 = to.int64['d' times 1] otherwise -1;
    no_check my.exit[to.int32['n' mod.floor 256]];
}
