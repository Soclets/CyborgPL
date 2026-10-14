# `list.slice` and `array.slice` (owner, 2026-09-23): 1-based, inclusive,
# a new list owning copies — so changing the slice leaves the original be.
# `pos n, n - 1` is empty. Elements that own memory are copied, not
# shared: the leak check frees each list's texts once.
extern.C fn exit ['code': int32];
MAIN {
    let 'xs': list[int64] = [10, 20, 30, 40, 50];
    let mut 'mid' := list.slice[lend 'xs', from pos 2, to pos 4];
    list.append[lend.mut 'mid', value 7];
    let 'none' := list.slice[lend 'xs', from pos 3, to pos 2];
    let 'fixed': array[int32] = [1, 2, 3];
    let 'head' := array.slice[lend 'fixed', from pos 1, to pos 2];
    let 'words': list[str.utf8] = ["a", "bb", "ccc"];
    let 'tail' := list.slice[lend 'words', from pos 2, to pos 3];
    # 20 + 30 + 40 + 7, four elements, the original's five, none, two, two
    let 'sum': int64 = 'mid'[pos 1] plus 'mid'[pos 2] plus 'mid'[pos 3] plus 'mid'[pos 4];
    let 'n': uint64 = list.length[lend 'mid'] times 10 plus list.length[lend 'xs'] plus list.length[lend 'none'] plus list.length[lend 'head'] plus list.length[lend 'tail'];
    no_check my.exit[to.int32['sum' plus to.int64['n']]];
}
