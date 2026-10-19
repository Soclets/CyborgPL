# Finding 2026-10-07 (check run --risk 20, seed 32250 before the call cap c720eda4+1), not cut down:
# CCA refuses an xor at line 178 (column 174 in the program below): "`xor` on an exact answer works at
# the width of its typed operands, and these have different sizes". Its right side holds int32 names
# beside simd.sum['s37'] (a plain number since the 2026-10-07 ruling). The oracle runs the program.
# Small versions agree, so it needs more of this context.
# check claude generator, seed 32250
extern.C fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;

fn emit ['m': uint64, 'neg': bool] {
    let mut 'buf': list[uint8] = [];
    if 'neg' { list.append[lend.mut 'buf', value 45]; }
    let mut 'v': uint64 = 'm';
    let mut 'ds': list[uint8] = [];
    list.append[lend.mut 'ds', value to.uint8[48 plus ('v' mod.floor 10)]];
    change 'v' = 'v' div.floor 10;
    while 'v' > 0 {
        list.append[lend.mut 'ds', value to.uint8[48 plus ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
    }
    let mut 'k': uint64 = list.length[lend 'ds'];
    while 'k' at_least 1 {
        list.append[lend.mut 'buf', value 'ds'[pos 'k']];
        change 'k' = 'k' minus 1;
    }
    list.append[lend.mut 'buf', value 10];
    NO_FUCKING_CHECKS write[1, lend 'buf', list.length[lend 'buf']];
}

fn put_u ['n': uint64] {
    emit['n', false];
}

fn put_i ['n': int64] {
    if 'n' at_least 0 {
        emit[to.uint64['n'], false];
    } else {
        let 'up': int64 = -('n' plus 1);
        emit[to.uint64['up'] plus 1, true];
    }
}

enum E1 {
    c1,
    c2: int64
}

fn f1 ['p1': uint16] -> int32[min -181, max 64] {
    {
        let mut 'w1': int32 = 7;
        loop {
            if 'w1' at_most 0 {
                break;
            }
            change 'w1' = 'w1' minus 1;
            let 'v2': uint32 = (to.uint32['p1'] mod.floor 12) plus 4294967284;
            let 's3': simd[int64, count 2] = simd.fill[value ((18 mod.floor to.int64['p1']) plus (to.int64['p1'] plus to.int64['w1'])) mod.near ((((-to.int64['w1']) times (((7638060362594655029 minus to.int64['v2']) mod.floor 6074000999) minus 3037000499)) mod.floor 7) plus 1)];
            check change 'v2' = (((to.uint32['w1'] plus ('v2' mod.floor 4294967289)) plus.wrap 'v2') mod.floor 12) plus 4294967284;
        }
    }
    {
        let 'w4': brain16 = -1.75;
        let 'g5': bin64 = 12.75;
        let 'w6' := to.brain16['g5'];
        let 'w7': brain16 = ('w4' mod.ceil.ieee (0.5));
        if 'w6' at_least 2.5 {
            put_u[1];
        } else {
            put_u[0];
        }
        if 'w7' < 0.1 {
            put_u[1];
        } else {
            put_u[0];
        }
    }
    {
        let mut 'w8': uint64 = 6;
        loop {
            if 'w8' at_most 0 {
                break;
            }
            change 'w8' = 'w8' minus 1;
            {
                let 'w9': brain16 = 0.5;
                let 'w10': brain16 = 'w9' div.near 0.001;
                let 'w11': brain16 = 'w10' times 'w9';
                let 'w12': brain16 = ('w10' minus.ieee 'w11');
                let 'g13': bin64 = -1.75;
                let 'w14' := to.brain16['g13'];
                let 'd15' := to.stored.deci64['w11'];
                let 'z16' := to.int64['d15' times 1000000] otherwise -1;
                put_i['z16'];
                let 't17' := to.stored.str.utf8['w14'];
                put_u[str.bytes[lend 't17']];
            }
            let 'v18': int64 = 18 minus (to.int64['p1'] minus to.int64['w8'] minus to.int64['w8'] times to.int64['p1']);
        }
    }
    return ((to.int32['p1'] mod.floor (-380335291)) mod.near (((710220129 div.floor 1081046253) mod.floor 1) plus 1)) minus 14;
}

fn f2 ['pr': uint8[min 0, max 3], 'p1': uint16, 'p2': int16, 'p3': int8] -> int8 {
    if 'pr' equal_to 0 {
        return 'p3';
    }
    let 'r19': int8 = f2['pr' minus 1, 'p1' div.ceil (('p1' mod.floor 25929) plus 1), (('p2' mod.floor 363) minus 181) times (((-(('p2' mod.floor 32767) minus 2638)) mod.floor 363) minus 181), to.int8['pr']];
    for perm 'i20': uint8 from 212 to 255 by 7 {
        put_u[to.int64[-9223372036854775808] mod.floor (-1)];
        {
            let 'd21': deci128 = 6411.0;
            let 'd22': deci128 = 'd21' pow (-3);
            let 'd23': deci128 = 'd22' pow (-1);
            let 'd24' := to.deci32['d21'] otherwise -88.0;
            let 'z25' := to.int64['d24' times 1000] otherwise -1;
            put_i['z25'];
            let 't26' := to.useful.str.utf8['d22'];
            put_u[str.bytes[lend 't26']];
            let 'z27' := to.int64['d23' times 10] otherwise -1;
            put_i['z27'];
            let 't28': str.utf8 = "Z";
            let 't29' := to.str.ascii['t28'] otherwise "?";
            put_u[str.bytes[lend 't29']];
            let 'k30' := to.int16["1e-3"] otherwise -2;
            put_i[to.int64['k30']];
        }
        let 'v31': int64 = ((((((to.int64['p1'] mod.floor 7) minus 3) pow 1) xor (4 mod.floor (-1))) minus (((1 mod.floor 2) mod.ceil (((to.int64['p1'] div.floor ((to.int64['r19'] mod.floor 8) minus 8)) mod.floor 5) plus 1)) mod.floor 1)) mod.floor 1715) minus 876;
    }
    let 'v32': uint8 = 'i20';
    return 'r19' mod.floor (-3);
}

MAIN {
    check let 'v33': uint8 = ((0 div.trunc 1) mod.floor 17) plus 232;
    check let 'v34': uint32 = ((((to.uint32['v33'] mod.floor 4) pow 11) or.bit (278533995 times 0)) mod.floor 311) plus 660;
    let 'v36': uint16 = match E1.c1 {
        c1 { is ((14 minus 9 plus to.uint16['v33']) mod.floor 35) plus 65501; }
        c2 'd35' { is ((((((40959 times 1) mod.floor 256) times (2 minus (to.uint16['v34'] mod.floor 3))) mod.floor 256) times 218) mod.floor 35) plus 65501; }
    };
    let mut 's37': simd[int32, count 4] = simd.fill[value f1[917]];
    change 'v34' = ((((to.uint32['v36'] times ((3626913585 minus 'v34' plus 'v34') mod.floor 65536)) mod.floor 311) plus 660) mod.floor 311) plus 660;
    {
        if false and to.uint64['v36'] minus 60550 equal_to 4044568063511618629 {
            let 'v38': int64[min 1] = ((-1) mod.floor 9223372036854775807) plus 1;
            {
                let 'w39': brain16 = 2.5;
                let 'w40' := to.brain16['v33'];
                let 'w41' := to.brain16[".5"] otherwise 0.1;
                let 'w42': brain16 = ('w40' minus.ieee 'w39');
                if 'w40' < 1000.0 {
                    put_u[1];
                } else {
                    put_u[0];
                }
                let 'd43' := to.useful.deci64['w40'];
                let 'z44' := to.int64['d43' times 100] otherwise -1;
                put_i['z44'];
            }
        } else if ('v36' not_equal 2 and.then (10 mod.floor ('v36' minus 2)) < 3) and (to.int32['v33'] shl 22) equal_to ((simd.sum.wrap['s37'] mod.floor 92681) minus 46340) times ((simd.min['s37'] mod.floor 92681) minus 46340) {
            change 'v36' = (((to.uint16['v33'] xor 'v36') mod.floor ((('v36' and.bit 1) mod.floor 42626) plus 1)) mod.floor 35) plus 65501;
            {
                let 'w45': brain16 = 300.7;
                let 'w46' := to.brain16['v33'];
                let 'g47': bin64 = 2.5;
                let 'w48' := to.brain16['g47'];
                let 'd49': deci64 = 12.75;
                let 'w50' := to.brain16['d49'];
                let 'w51': brain16 = 'w48';
                let 'h52' := to.bin16['w51'];
                let 'd53' := to.useful.deci64['h52'];
                let 'z54' := to.int64['d53'] otherwise -1;
                put_i['z54'];
                if 'w45' at_least 0.001 {
                    put_u[1];
                } else {
                    put_u[0];
                }
                let 't55' := to.stored.str.utf8['w45'];
                put_u[str.bytes[lend 't55']];
            }
        }
        let 's56': simd[uint32, count 4] = simd.fill[value (('v34' minus.wrap 2147483648) or.bit (12 plus 943786366)) mod.floor (((0 div.floor (to.uint32['v33'] plus to.uint32['v33'])) mod.floor 521607920) plus 1)];
        put_i[to.int64['v34'] plus to.int64['v33'] minus 5058769046726752839];
        change 's37' = simd.fill[value (((simd.max['s37'] mod.floor 7) minus 3) pow (simd.max['s37'] mod.floor 20) div.floor (-(-9))) mod.floor ((((to.int32['v36'] mod.ceil (-2)) xor (to.int32['v34'] minus ((simd.sum['s37'] mod.floor 2147483647) minus 351439531))) mod.floor 854227907) plus 1)];
    }
    for 'i57': int32 from 10 to 22 by 2 {
        put_i[to.int64['v36'] minus to.int64['v33'] plus (to.int64['v33'] shl 4) minus (to.int64['v34'] shl 52)];
        {
            let 'w58': brain16 = 0.1;
            let 'w59': brain16 = ('w58' div.ceil.ieee (0.001));
            let 'w60': brain16 = 'w58';
            let 'h61' := to.bin16['w60'];
            let 'd62' := to.useful.deci64['h61'];
            let 'z63' := to.int64['d62'] otherwise -1;
            put_i['z63'];
            if 'w58' equal_to 0.1 {
                put_u[1];
            } else {
                put_u[0];
            }
            let 'z64' := to.int64['w58'] otherwise -1;
            put_i['z64'];
        }
    }
    change 'v36' = 65527;
}
