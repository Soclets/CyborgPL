# a slice is a list, an array's too (owner, 2026-09-23)
MAIN {
    let 'f': array[int32] = [10, 20, 30];
    let 'b': array[int32] = array.slice[lend 'f', from pos 1, to pos 2];
}
