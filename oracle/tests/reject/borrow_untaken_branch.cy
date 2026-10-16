# the borrow rule holds on every path, not only the one a run takes (check claude, 2026-09-29)
MAIN {
    let 'n': int32 = 3;
    let mut 't': list[int32] = [5,];
    if 'n' > 5 {
        let 'q' := lend 't';
        list.append[lend.mut 't', value 1];
        std::print.useful[list.length[lend 'q'], newln];
    }
}
