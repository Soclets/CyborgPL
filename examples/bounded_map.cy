# A map's count bound (decided 2026-09-22), and bounds on the things inside — types composing.
MAIN {
    let mut 'small': std::map[str.utf8[min: 1, max: 3 bytes], i32[min: 0, max: 10]][min: 0, max: 2] = std::map.empty;
    std::map.set[lend.mut 'small', key: "ab", value: 5];
    std::map.set[lend.mut 'small', key: "cd", value: 7];
    let 'n' := std::map.length[lend 'small'];         # u64[min: 0, max: 2]
    let 'tiny': i8 = 'n';                      # proven by the bound
    std::print.useful['tiny', " entries", newln];
    std::map.remove[lend.mut 'small', key: "ab"] otherwise { missing { } };
    std::map.set[lend.mut 'small', key: "ef", value: 10];         # back to two: fits
    std::print.useful[std::map.length[lend 'small'], newln];
}
