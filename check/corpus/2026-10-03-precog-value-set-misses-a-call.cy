# Finding 2026-10-03 (check claims, seed 17021 with --avoid exact-bits,lists,borrows,simd): a wrong
# layer-2 value claim. Precog says the value of to.u8['p3' mod.floor 256] in f1's `return` (line
# 47 here, line 40 in the seed) is only {4}; f1 reaches it twice, with 6 and with 4 — the line before
# it prints 999006 then 999004 in both engines, and check's exact evaluator sees [6, 4]. Harmless here
# (6 fits u8), but a value set missing a reachable value is a wrong proof. The engines agree on the
# output, so `check corpus` passes; see `cca precog --json` on this file, line 47 (start 47:17).
# check claude generator, seed 17021
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': std::list[u8] = [];
    if 'neg' { std::list.append[lend.mut 'buf', value: 45]; }
    let mut 'v': u64 = 'm';
    let mut 'ds': std::list[u8] = [];
    loop {
        std::list.append[lend.mut 'ds', value: to.u8[48 plus ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
        if 'v' equal_to 0 { break; }
    }
    for 'k': u64 [from: std::list.length[lend 'ds'], to: 1, by: -1] {
        std::list.append[lend.mut 'buf', value: 'ds'[pos: 'k']];
    }
    std::list.append[lend.mut 'buf', value: 10];
    no_check write[1, lend 'buf', std::list.length[lend 'buf']];
}

fn put_u ['n': u64] {
    emit['n', false];
}

fn put_i ['n': i64] {
    if 'n' < 0 {
        emit[to.u64[0 minus ('n' plus 1)] plus 1, true];
    } else {
        emit[to.u64['n'], false];
    }
}

enum E1 {
    c1,
    c2: u8
}

fn f1 ['pr': i32[min: 0, max: 8], 'p1': bool, 'p2': u16, 'p3': u64] -> u8 {
    if 'pr' at_least 8 {
        put_u[999000 plus ('p3' mod.floor 256)];
        return (to.u8['p3' mod.floor 256] shl 0) minus ((to.u8['pr'] plus 127) mod.floor 1);
    }
    let 'r1': u8 = f1['pr' plus 1, 'p1' or (((117 div.floor 15) div.near (((12 mod.floor ((to.i8['pr'] mod.floor 62) minus 62)) mod.floor 62) minus 62)) not_equal ((-128) plus (((to.i8['p3' mod.floor 128] or.bit (-11)) mod.floor 127) plus 1))), 32768 plus ((('p2' times 1) mod.floor (((to.u16[('p2' minus ('p2' mod.floor 1)) minus (('p2' plus 0) mod.floor 1)] times.wrap to.u16['p3' mod.floor 65536]) mod.floor 43388) plus 1)) mod.floor 32768), ((to.u64['p2'] plus 9223372036854775807) mod.floor ((to.u64['p2'] mod.floor 10) plus 1)) mod.floor 15663594281552694135];
    let 'v2': u32[min: 0, max: 198] = 2 pow 4;
    {
        let mut 'w3': u8 = 3;
        loop {
            if 'w3' equal_to 0 {
                break;
            }
            change 'w3' = 'w3' minus 1;
            put_i[(to.i64['v2'] div.near (-1150194949118367529)) div.floor (((to.i64['r1'] shl 0) mod.floor 5) plus 1)];
            put_i['pr' div.floor ((('pr' minus 'pr') mod.floor 1168629669) plus 1)];
        }
    }
    return to.u8[0 times 'r1'] and.bit to.u8['r1' plus (to.u8['p2' mod.floor 256] mod.floor 1)];
}

fn f2 ['p1': u16] -> u16 {
    if ((49 div.floor 19) > to.u8['p1' mod.floor 256]) and true {
        let 'e4': E1 = E1.c1;
        let 'v5': i16[max: 100] = (to.i16['p1' mod.floor 32768] mod.floor 32767) minus 32758;
    }
    let 'v7': i32 = match E1.c1 {
        c1 { is (to.i32['p1'] mod.floor 523) minus 592; }
        c2 'd6' { is -141; }
    };
    let mut 'v8': i64 = ((to.i64[(to.i64['v7'] minus 15) minus (9223372036854775806 div.floor 4611686018427387904)] shl to.i64[(to.i64['p1'] or.bit to.i64['v7']) times (to.i64['v7'] div.trunc (-4611686018427387904))]) mod.floor 868) minus 324;
    no_cheat for 'i9': i64 [from: 9223372036854775786, to: 9223372036854775807, by: 2] {
        change 'v8' = (((to.i64[(-5054614212144933731) plus 'i9'] minus.wrap 'v8') plus (('i9' minus 'i9') mod.floor 1)) mod.floor 868) minus 324;
        let 'v10': i16 = ((to.i16[('i9' mod.floor 65536) minus 32768] mod.trunc ((((to.i16['v8'] shr 13) plus (((-16384) minus to.i16['v7']) mod.floor 1)) mod.floor 3) plus 1)) mod.floor 45) plus 32723;
    }
    return ((('p1' plus 0) plus (('p1' div.trunc (('p1' mod.floor 7) plus 1)) mod.floor 1)) mod.floor 60) plus 65476;
}

MAIN {
    let 'v11': bool = true;
    let 'v12': u16 = ((47702 plus 10915) plus ((20 plus 44125) mod.floor 6919)) mod.trunc ((65535 div.floor f2[f2[65534 mod.trunc 44603] minus 0]) mod.floor 11);
    if true {
        if ('v12' equal_to 28) or.else ((12 mod.floor ('v12' minus 28)) < (-1)) {
            let 'e13': E1 = E1.c2[(f1[7, (to.i32['v12'] not_equal ((-320966724) mod.ceil 594269508)) or ((to.u8['v12'] shl to.u8['v12']) equal_to f1[to.i32['v12'] times to.i32['v12'], ((-64) equal_to (to.i8['v12'] times (-69))) and (('v12' equal_to 0) or.else ((20 mod.floor 'v12') < 0)), 'v12', to.u64['v12'] plus 0]), (f2[to.u16[f2[0 plus f2[0 plus 'v12']]] and.bit to.u16[to.u16['v12' plus 'v12'] xor to.u16['v12' div.trunc (('v12' mod.floor 8460) plus 1)]]] div.floor 3) plus ('v12' plus ('v12' or.bit 'v12')), 5 times (to.u64['v12'] div.floor 9223372036854775807)] minus 0) mod.floor 53];
            let immut 'v14': u16[min: 111] = ('v12' mod.floor 65425) plus 111;
            put_i[to.i16['v14' mod.floor 32768]];
            let 'v16': i16[max: -110] = match 'e13' {
                c1 { is ((1 times ((((-8590) plus to.i16['v12']) mod.floor 363) minus 181)) mod.floor 32658) minus 32767; }
                c2 'd15' { is ((to.i16[to.i16['d15'] minus to.i16['v14' mod.floor 32768]] shr to.i16[to.i16[to.i16['d15'] minus 17535] mod.floor 16]) mod.floor 32658) minus 32767; }
            };
        } else if (to.i8[(-9) minus 26] and.bit 2) equal_to (-9) {
            NO_FUCKING_CHECKS let mut 'v17': i64 = (((-15) times to.i64['v12']) mod.floor 954) minus 988;
            change 'v17' = ((3485829636007882345 mod.floor 'v17') mod.floor 954) minus 988;
        } else {
            let 'v18': i16[max: 100] = 1;
            put_u[1151851384 plus to.u32['v12']];
        }
        put_u[15 or.bit to.u64[(to.u64['v12'] minus 0) minus to.u64['v12']]];
        match E1.c1 {
            c1 {
                let 'v19': bool = ((to.i8['v12'] div.ceil (-4)) > (to.i8['v12'] shl to.i8['v12'])) or.else 'v11';
                let 'v20': bool = (to.i16['v12'] mod.floor ((to.i16['v12'] mod.floor 9) plus 1)) < (to.i16['v12'] and.bit to.i16['v12']);
            }
            c2 'd21' {
                put_i[(-7) times (((to.i8['v12'] plus (-40)) mod.floor 23) minus 11)];
                put_i[to.i16['v12'] times ((3 pow to.i16['v12']) mod.trunc (((to.i16['d21'] mod.floor (-32767)) mod.floor 4) plus 1))];
                let 'e22': E1 = E1.c1;
            }
        }
        let 'v24': u64 = match E1.c1 {
            c1 { is ((to.u64[0 div.floor (13170323352803615965 div.ceil 8773915497413217533)] or.bit to.u64[(10497958590430486209 mod.floor ((to.u64['v12'] mod.floor 8) plus 1)) mod.floor (((to.u64['v12'] div.floor 8) mod.floor 4) plus 1)]) mod.floor 328) plus 57; }
            c2 'd23' { is ((to.u64[(2 div.trunc ((to.u64['v12'] mod.floor 8202440049947476054) plus 1)) div.trunc ((to.u64['d23'] mod.floor 9612966461674267916) plus 1)] shr 15) mod.floor 328) plus 57; }
        };
    } else {
        let 'v25': u8 = ((to.u8[f1[1 and.bit to.i32[405190320 minus to.i32['v12']], (('v12' not_equal 2) and.then ((47 div.floor ('v12' minus 2)) < 0)) and (to.i8['v12'] at_least (-16)), 16, to.u64['v12']] times to.u8['v12']] shl to.u8[to.u8['v12'] plus to.u8['v12']]) mod.floor 56) plus 200;
        let 'v26': i32 = (11 div.floor 1472794052) plus to.i32['v12'];
        put_u[f1[6, to.i16['v26'] equal_to (-31677), ((0 mod.floor f2[((5 times 'v12') plus (65535 and.bit 'v12')) pow ((2 div.floor (('v12' mod.floor 39297) plus 1)) plus ('v12' times 2))]) times ('v12' minus 0)) times (to.u16['v12' minus 'v12'] or.bit to.u16['v12' mod.floor (('v12' mod.floor 3) plus 1)]), to.u64['v25'] minus ((9223372036854775807 xor to.u64[5 times 9]) mod.floor 201)] div.floor 69];
        if 'v11' {
            change 'v25' = ((('v25' mod.floor 'v25') minus ('v25' mod.floor 1)) mod.floor 56) plus 200;
            change 'v25' = 'v25' plus (f1[0 times (-'v26'), not (to.i16['v26'] at_most to.i16['v26']), 'v12', to.u64[0 plus to.u64['v25']] or.bit 6429869190640371940] mod.floor 1);
            change 'v25' = ((('v25' mod.floor 16) times ('v25' mod.floor 16)) mod.floor 56) plus 200;
        } else if 'v11' and.then ((to.i64[3] shr to.i64['v12']) at_least 9223372036854775806) {
            put_i[(-25260) plus to.i16['v26']];
            change 'v25' = (to.u8['v26'] mod.floor 56) plus 200;
        } else if 'v11' {
            let 'v27': i64 = ((19 div.ceil 2) mod.floor 476) plus 378;
            put_i[((-5) mod.ceil (-52)) plus to.i8['v12']];
        } else {
            change 'v25' = (((('v25' mod.trunc ((to.u8['v12'] mod.floor 109) plus 1)) plus ('v25' mod.floor 148)) minus ((('v25' minus.wrap 242) plus (('v25' plus to.u8['v12']) mod.floor 1)) mod.floor 1)) mod.floor 56) plus 200;
            let immut 'v28': bool = not (((-116) mod.trunc (((to.i8['v26'] plus to.i8['v26']) mod.floor 3) plus 1)) equal_to ((to.i8['v26'] times (-100)) pow 3));
        }
    }
    put_u['v12'];
    put_i[((to.i16['v12'] minus to.i16['v12']) times 16384) mod.ceil ((((1865 plus (-15)) mod.floor ((-7092) minus 2)) mod.floor 6) plus 1)];
    match E1.c2[to.u8['v12']] {
        c1 {
            for perm 'i29': i64 [from: 9223372036854775787, to: 9223372036854775807, by: 2] {
                put_u[32768 plus 'v12'];
                let 'v30': u32 = to.u32['v12'] div.floor ((to.u32['v12'] mod.floor 1984127403) plus 1);
                let immut 'v31': bool = (('v12' not_equal 21) and.then ((6 div.floor ('v12' minus 21)) < 1)) and.then (((to.i32['v12'] times 5) minus (to.i32['v30'] div.trunc ((to.i32['v12'] mod.floor 977265213) plus 1))) equal_to (-to.i32['v12']));
            }
            if false {
                let mut 'e32': E1 = E1.c1;
                change 'e32' = E1.c2[to.u8['v12'] minus 0];
            }
        }
        c2 'd33' {
            if 2 not_equal ((10922990468106277506 mod.floor ((to.u64['d33'] mod.floor 4) plus 1)) minus (to.u64['v12'] minus 0)) {
                let immut 'v34': bool = (to.u64['v12'] at_most to.u64['v12']) or 'v11';
                let immut 'v36': i16[min: 1] = match E1.c2['d33'] {
                    c1 { is (((1 div.near (((to.i16['d33'] and.bit (-19)) mod.floor 5) plus 1)) mod.ceil ((-16383) div.ceil (((0 mod.trunc 9) mod.floor 10) plus 1))) mod.floor 32767) plus 1; }
                    c2 'd35' { is ((1 mod.floor ((-32768) plus 31630)) mod.floor 32767) plus 1; }
                };
            } else if ((2040893491 div.near ((((-752746485) mod.floor 11) mod.floor 5) plus 1)) at_most ((1494455099 plus to.i32['v12']) mod.floor 6)) and 'v11' {
                let 'v37': u8 = ((('d33' minus.wrap to.u8['d33' or.bit to.u8['v12']]) mod.floor (('d33' mod.floor 6) plus 1)) mod.floor 51) plus 100;
                let 'v38': i16 = 16383 plus to.i16['d33'];
            } else {
                let 'v39': u32 = 2314499214 plus to.u32['v12'];
                let 'v40': u64 = (((((to.u64['v12'] plus to.u64['d33']) mod.floor 4) pow 27) div.floor ((to.u64['d33'] plus 9223372036854775807) plus (9223372036854775808 div.trunc to.u64['v39']))) mod.floor 966) plus 13;
                let 'v41': bool = (((to.i8['v40'] minus 73) at_least (((to.i8['v39' mod.floor 128] mod.floor 23) minus 11) times ((to.i8['d33'] mod.floor 23) minus 11))) or.else (((to.i8['v39' mod.floor 128] div.near ((to.i8['v40' mod.floor 128] mod.floor 2) plus 1)) not_equal (to.i8['d33'] div.floor (-17))) and (0 < to.i64['d33']))) and (not (not 'v11'));
            }
            put_i[to.i64['v12'] or.bit 4611686018427387904];
        }
    }
}
