# `div.trunc` and `mod.trunc` worked out at run time, over every int8 and
# five divisors: q x d + r is the dividend, the remainder is below the
# divisor's size and on the dividend's side of zero. The lowest int8 by -1
# is past the type for `div.trunc` and is left out; its `mod.trunc` is 0.
# Lane by lane on a vector too, and C's `fmod` on bins: 4 + 8 + 16 = 28
# when all hold, minus 1 for the first that does not.
extern.C fn exit ['code': int32];
MAIN {
    let 'ds': list[int8] = [-3, -1, 1, 2, 7];
    let mut 'bad': int32 = 0;
    for 'd' in lend 'ds' {
        for 'i' from -128 to 127 by 1 {
            let 'n': int8 = to.int8['i'];
            let 'r': int8 = 'n' mod.trunc 'd';
            if ('n' not_equal -128) or ('d' not_equal -1) {
                let 'q': int8 = 'n' div.trunc 'd';
                if (to.int32['q'] times to.int32['d']) plus to.int32['r'] not_equal 'i' { change 'bad' = 1; }
            }
            if ('r' < 0) and ('n' > 0) { change 'bad' = 1; }
            if ('r' > 0) and ('n' < 0) { change 'bad' = 1; }
        }
    }
    let mut 'code': int32 = 4;
    if 'bad' not_equal 0 { change 'code' = 3; }
    let 'xs': list[int32] = [-7, 7, -8, 9];
    let 'v': simd[int32, 4] = simd.load[lend 'xs', pos 1];
    let 'two': simd[int32, 4] = simd.fill[value -2];
    let 'q' := 'v' div.trunc 'two';
    let 'r' := 'v' mod.trunc 'two';
    if ('q'[pos 1] equal_to 3) and ('q'[pos 2] equal_to -3) and ('q'[pos 3] equal_to 4) and ('q'[pos 4] equal_to -4) and ('r'[pos 1] equal_to -1) and ('r'[pos 2] equal_to 1) and ('r'[pos 3] equal_to 0) and ('r'[pos 4] equal_to 1) {
        change 'code' = 'code' plus 8;
    }
    let 'fs': list[bin64] = [-370.0, 370.0, 7.5];
    let mut 'fsum': bin64 = 0.0;
    for 'f' in lend 'fs' {
        change 'fsum' = 'fsum' plus ('f' mod.trunc 360.0);
    }
    if 'fsum' equal_to 7.5 { change 'code' = 'code' plus 16; }
    no_check my.exit['code'];
}
