extern.C fn p_one = C.printf ['format': ptr[uint8], ...'n': int32] -> int32;

MAIN {
    let 'f' := C.str.make["%d %d"];
    no_check p_one[lend 'f', 1, 2];
}
