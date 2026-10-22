# `list.slice` and `array.slice` (owner, 2026-09-23): 1-based, inclusive,
# a new list owning copies — so changing the slice leaves the original be.
# `pos n, n - 1` is empty. Elements that own memory are copied, not
# shared: the leak check frees each list's texts once.
extern.C fn exit ['code': i32];
MAIN {
    let 'xs': std::list[i64] = [10, 20, 30, 40, 50];
    let mut 'mid' := std::list.slice[lend 'xs', from: pos 2, to: pos 4];
    std::list.append[lend.mut 'mid', value: 7];
    let 'none' := std::list.slice[lend 'xs', from: pos 3, to: pos 2];
    let 'fixed': array[i32] = [1, 2, 3];
    let 'head' := array.slice[lend 'fixed', from: pos 1, to: pos 2];
    let 'words': std::list[str.utf8] = ["a", "bb", "ccc"];
    let 'tail' := std::list.slice[lend 'words', from: pos 2, to: pos 3];
    # 20 + 30 + 40 + 7, four elements, the original's five, none, two, two
    let 'sum': i64 = 'mid'[pos: 1] plus 'mid'[pos: 2] plus 'mid'[pos: 3] plus 'mid'[pos: 4];
    let 'n': u64 = std::list.length[lend 'mid'] times 10 plus std::list.length[lend 'xs'] plus std::list.length[lend 'none'] plus std::list.length[lend 'head'] plus std::list.length[lend 'tail'];
    no_check my.exit[to.i32['sum' plus to.i64['n']]];
}
