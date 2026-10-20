extern.C fn p_one = C.printf ['format': ptr[u8], ...'n': i32] -> i32;

MAIN {
    let 'f' := C.str.make["%d %d"];
    no_check p_one[lend 'f', 1, 2];
}
