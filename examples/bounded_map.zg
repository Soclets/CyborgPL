# A map's count bound (decided 2026-09-22), and bounds on the things inside — types composing.
MAIN {
    let mut 'small': map[str.utf8[min 1, max 3 bytes], int32[min 0, max 10]][min 0, max 2] = map.empty;
    map.set[lend.mut 'small', key "ab", value 5];
    map.set[lend.mut 'small', key "cd", value 7];
    let 'n' := map.length[lend 'small'];         # uint64[min 0, max 2]
    let 'tiny': int8 = 'n';                      # proven by the bound
    std::print.useful['tiny', " entries", newln];
    map.remove[lend.mut 'small', key "ab"] otherwise { missing { } };
    map.set[lend.mut 'small', key "ef", value 10];         # back to two: fits
    std::print.useful[map.length[lend 'small'], newln];
}
