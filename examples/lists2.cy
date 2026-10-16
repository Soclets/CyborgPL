# Reading, appending to and summing a list.
fn sum [lent 'items': list[int32]] -> int32 {
    let 'total' := 0;
    for 'n' in lend 'items' {
        change 'total' = 'total' + 'n';
    }
    return 'total';
}

MAIN {
    let 'xs': list[int32] = [10, 20, 30];
    std::print.useful["first ", 'xs'[pos 1], ", last ", 'xs'[pos 3], newln];
    let 'n' := list.length[lend 'xs'];
    std::print.useful["length ", 'n', newln];
    list.append[lend.mut 'xs', value 40];
    list.remove[lend.mut 'xs', pos 1];
    std::print.useful['xs', " length ", list.length[lend 'xs'], newln];
    std::print.useful["sum ", sum[lend 'xs'], newln];
    for 'i' from 1 to list.length[lend 'xs'] by 1 {
        std::print.useful['xs'[pos 'i'], " "];
    }
    std::print.stored[newln];
    let 'days': array[str.utf8] = ["mon", "tue"];
    std::print.stored['days'[pos 2], " of ", array.length[lend 'days'], newln];
    let 'grid' := [[1, 2], [3, 4]];
    std::print.useful['grid'[pos 2], newln];
    for perm 'i' from 1 to list.length[lend 'xs'] by 1 {
        if 'xs'[pos 'i'] equal_to 30 { break; }
    }
    change 'xs'[pos 'i'] = 33;
    std::print.useful['xs', newln];
}
