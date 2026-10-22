# `to.array[…]` freezes a list's length, `to.list[…]` lets an array grow
# (owner, 2026-09-23). `give` hands the elements over without a copy;
# `copy` keeps the original. Texts inside are copied, not shared: the
# leak check frees each once.
extern.C fn exit ['code': i32];
MAIN {
    let mut 'l': std::list[i32] = [1, 2];
    std::list.append[lend.mut 'l', value: 3];
    let 'a' := to.array[give 'l'];
    let mut 'm' := to.list[copy 'a'];
    std::list.append[lend.mut 'm', value: 4];
    let mut 's': std::list[i32] = array.slice[lend 'a', from: pos 2, to: pos 3];
    std::list.append[lend.mut 's', value: 9];
    let mut 'w': std::list[str.utf8] = ["x", "yy"];
    std::list.append[lend.mut 'w', value: "zzz"];
    let 'fw' := to.array[copy 'w'];
    let 'back' := to.list[give 'fw'];
    # 3 + 4 x 10 + 3 x 100 + 3 + (1 + 4 + 9)
    let 'n': u64 = array.length[lend 'a'] plus std::list.length[lend 'm'] times 10 plus std::list.length[lend 's'] times 100 plus std::list.length[lend 'back'];
    let 'v': i32 = 'a'[pos: 1] plus 'm'[pos: 4] plus 's'[pos: 3];
    no_check my.exit[to.i32['n'] plus 'v'];
}
