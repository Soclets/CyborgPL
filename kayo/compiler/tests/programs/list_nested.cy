extern.C fn exit ['code': i32];
MAIN {
    let mut 'grid': list[list[i32]] = [[1, 2], [3, 4]];
    let 'row': list[i32] = [5, 6, 7];
    list.append[lend.mut 'grid', value give 'row'];
    let 'copy_of': list[list[i32]] = copy 'grid';
    let 'rows': i32 = list.length[lend 'grid'];
    let 'third': i32 = 'grid'[pos 3][pos 3];
    let 'crows': i32 = list.length[lend 'copy_of'];
    no_check my.exit['rows' plus 'third' plus 'crows'];
}
