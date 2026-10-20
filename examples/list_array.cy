# A list and an array are different types; nothing converts by itself (2026-09-19).
# `to.array[…]` freezes a list's length, `to.list[…]` lets an array grow (owner, 2026-09-23).
# The element type stays, so it is not written. `give` hands the elements over; `copy` keeps the original.
MAIN {
    let mut 'l': list[i32] = [1, 2];
    list.append[lend.mut 'l', value 3];
    let 'a' := to.array[give 'l'];                 # [1, 2, 3], can no longer grow
    let mut 'm' := to.list[copy 'a'];              # 'a' is still here
    list.append[lend.mut 'm', value 4];
    let mut 's': list[i32] = array.slice[lend 'a', from pos 2, to pos 3];   # a slice is a list, an array's too
    list.append[lend.mut 's', value 9];
    std::print.useful['a', " ", 'm', " ", 's', newln];
}
