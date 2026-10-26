# Finding 2026-09-29 (check run, seed 6313, shrunk): 'v16' is 831993207373 ((-256867307) x f3[1, 2, 7],
# f3 giving -3239), so to.i16['v16'] at line 77, column 755 must stop the program. CCA's Precog claims
# that narrowing proven (layer 3, solver) while its own operand range is about ±2.4e27, drops the
# check, and the binary runs on. The oracle stops there, as it should (design claude traced it).
# check claude generator, seed 6313
%extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': std::list[u8] = [];
    if 'neg' { std::list.append(lend.mut 'buf', value: 45); }
    let mut 'v': u64 = 'm';
    let mut 'ds': std::list[u8] = [];
    loop {
        std::list.append(lend.mut 'ds', value: to.u8(48 plus ('v' mod.floor 10)));
        change 'v' = 'v' div.floor 10;
        if 'v' equal_to 0 { break; }
    }
    for 'k': u64 [from: std::list.length(lend 'ds'), to: 1, by: -1] {
        std::list.append(lend.mut 'buf', value: 'ds'[pos: 'k']);
    }
    std::list.append(lend.mut 'buf', value: 10);
    NO_FUCKING_CHECKS write(1, lend 'buf', std::list.length(lend 'buf'));
}

fn put_u ['n': u64] {
    emit('n', false);
}

fn put_i ['n': i64] {
    if 'n' < 0 {
        emit(to.u64(0 minus ('n' plus 1)) plus 1, true);
    } else {
        emit(to.u64('n'), false);
    }
}

enum E1 {
    c1: i32,
    c2: i32,
    c3
}

fn b1 [@x, @y] [lent @x 'a': std::list[u64], lent @y 'b': std::list[u64]] -> lent @x std::list[u64] {
    return lend 'a';
}

fn b2 [@x] [lent @x 'a': std::list[u64]] -> lent @x std::list[u64] {
    if std::list.length(lend 'a') at_least std::list.length(lend 'a') {
        return b1(lend 'a', lend 'a');
    }
    return lend 'a';
}

fn f1 ['p1': i32, 'p2': i8, 'p3': i16] -> i32 {
    return 0;
}

fn f2 ['p1': bool, 'p2': i8] -> i8 {
    return 0;
}

fn f3 ['pr': i64[min: 0, max: 10], 'p1': u16, 'p2': i16] -> i64 {
    if 'pr' at_least 10 {
        return 'pr';
    }
    let 'r10': i64 = f3('pr' plus 1, 7 shr to.u16(0 pow 3), (('p2' mod.floor 363) minus 181) times (('p2' mod.floor 363) minus 181));
    return 'r10' minus 361;
}

MAIN {
    let 'v16': i64 = (-256867307) times f3(1, 2, 7);
    let 'w17': i32 = 8;
    while 'w17' > 0 {
        change 'w17' = 'w17' minus 1;
        for 'i19': i8 [from: -13, to: -36, by: -2] {
        }
    }
    let 'e25': E1 = E1.c1(f1(((to.i32(((-2147483648) plus 1073741823) div.floor (((f1((((((19 plus (-995984740)) mod.floor 92681) minus 46340) times (((to.i32(('v16' mod.floor 4294967296) minus 2147483648) minus (f1((((1495962669 minus ((f1(0 times (-2147483648), ((to.i8(('v16' mod.floor 256) minus 128) mod.floor ((to.i8(('v16' mod.floor 256) minus 128) mod.floor 6) plus 1)) mod.floor ((to.i8(('v16' mod.floor 256) minus 128) mod.floor 8) plus 1)) mod.floor (f2(not (((-0) div.floor ((((('v16' mod.floor 6074000999) minus 3037000499) times (('v16' mod.floor 6074000999) minus 3037000499)) mod.floor 10) plus 1)) at_least f3((14 mod.floor (('v16' mod.floor 6581727740982457365) minus 6581727740982457365)) mod.floor 11, to.u16('v16' mod.floor 65536) minus (((to.u16('v16' mod.floor 65536) mod.floor ((to.u16('v16' mod.floor 65536) mod.floor 3) plus 1)) mod.floor (((37339 mod.floor ((to.u16('v16' mod.floor 65536) mod.floor 62597) plus 1)) mod.floor 7) plus 1)) mod.floor 1), (-32768) plus (((((to.i16('v16') plus 0) mod.floor 363) minus 181) times (((to.i16(('v16' mod.floor 65536) minus 32768) shl 0) mod.floor 363) minus 181)) mod.floor 32767))), (((-1) pow 3) mod.floor 28) plus 100) minus (f2((('v16' div.floor (-1058134406755732930)) < ((('v16' mod.floor 6074000999) minus 3037000499) times (('v16' mod.floor 6074000999) minus 3037000499))) and (to.u8('v16' mod.floor 256) not_equal 18), (to.i8(('v16' mod.floor 256) minus 128) mod.floor 28) plus 100) minus (-44))), ((((-24941) plus (-10)) plus ((((-1) times 32766) mod.floor 32767) minus 4474)) mod.floor 215) minus 854) mod.floor 2147483647) minus 631999445)) div.floor ((((-1073741824) plus ((f1(((to.i32(('v16' mod.floor 4294967296) minus 2147483648) shl 0) mod.floor 506) minus 357, ((to.i8(('v16' mod.floor 256) minus 128) plus 0) mod.floor 103) minus 84, ((((((-7) times (((10 minus ((to.i16(('v16' mod.floor 65536) minus 32768) mod.floor 32767) minus 25945)) mod.floor 363) minus 181)) mod.floor 363) minus 181) times ((to.i16(('v16' mod.floor 65536) minus 32768) mod.floor 363) minus 181)) mod.floor 215) minus 854) mod.floor 2147483647) minus 426085934)) mod.floor 961562281) plus 1)) mod.floor 506) minus 357, (((-7) times ((to.i8(('v16' mod.floor 256) minus 128) mod.floor 23) minus 11)) mod.floor 103) minus 84, ((12895 plus ((to.i16('v16') mod.floor 32767) minus 29982)) mod.floor 215) minus 854) mod.floor 1)) mod.floor 92681) minus 46340)) mod.floor 506) minus 357, 8, (((to.i16(('v16' mod.floor 65536) minus 32768) plus ((to.i16(('v16' mod.floor 65536) minus 32768) mod.floor (-9)) mod.floor 1)) minus 0) mod.floor 215) minus 854) minus 0) mod.floor 67998013) plus 1)) and.bit to.i32(f1(((381626155 plus 614013067) mod.floor 506) minus 357, 11 shr (to.i8(('v16' mod.floor 256) minus 128) mod.floor 8), -687) plus ((1073741823 minus ((to.i32(('v16' mod.floor 4294967296) minus 2147483648) mod.floor 2147483647) minus 577224165)) mod.floor 1))) mod.floor 506) minus 357, ((f2((((14 times 5) minus ((-1) plus f2(not (to.i16(('v16' mod.floor 65536) minus 32768) not_equal (-32768)), ((to.i8(('v16' mod.floor 256) minus 128) plus 0) mod.floor 28) plus 100))) at_most ((((f2(to.u32('v16' mod.floor 4294967296) equal_to (0 times 4231346141), (((to.i8(to.i8(('v16' mod.floor 256) minus 128) shr 4) shr (to.i8(((to.i8(('v16' mod.floor 256) minus 128) mod.floor 7) minus 3) pow 2) mod.floor 8)) div.floor ((((((to.i8(('v16' mod.floor 256) minus 128) mod.floor 23) minus 11) times (-8)) plus (-1)) mod.floor 67) plus 1)) mod.floor 28) plus 100) minus ((to.i8(('v16' mod.floor 256) minus 128) mod.floor 127) minus 40)) mod.floor 23) minus 11) times (((to.i8(('v16' mod.floor 256) minus 128) minus 0) mod.floor 23) minus 11))) and ((to.u64(29137074 times 3294738485) xor to.u64(1 times 18446744073709551615)) equal_to (0 mod.floor (((5 or.bit to.u64('v16' mod.floor 9223372036854775807)) mod.floor 4) plus 1))), 111) minus f2(((to.i16(('v16' mod.floor 65536) minus 32768) shr 2) div.floor ((-76) times 2)) not_equal (-(to.i16(('v16' mod.floor 65536) minus 32768) div.floor 6708)), ((63 div.floor ((to.i8(('v16' mod.floor 256) minus 128) mod.floor 10) plus 1)) mod.floor 28) plus 100)) div.floor (-15)) plus (-33), ((14026 minus (3 div.floor 16383)) mod.floor 215) minus 854));
}
