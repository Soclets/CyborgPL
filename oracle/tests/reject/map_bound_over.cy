MAIN {
    let mut 'small': map[str.utf8[min: 1, max: 3 bytes], i32[min: 0, max: 10]][min: 0, max: 2] = map.empty;
    map.set[lend.mut 'small', key: "ab", value: 5];
    map.set[lend.mut 'small', key: "cd", value: 7];
    let 'n' := map.length[lend 'small'];
    let 'tiny': i8 = 'n';                          # proven by the bound: at most 2
    std::print.useful['tiny', newln];
    map.set[lend.mut 'small', key: "ef", value: 1];              # a third: does not fit [0, 2]
}
