# An append changes the list (check claude, 2026-09-29): while 'r', a borrow
# of 's' that `first` handed back, is still used, 1000 appends would move
# 's''s storage from under it — CCA's binary read freed memory. Refused.
fn first [lent 'a': std::list[i32]] -> lent std::list[i32] { return lend 'a'; }
MAIN {
    let mut 's': std::list[i32] = [1, 2, 3];
    let 'r' := first[lend 's'];
    for 'k': i32 [from: 1, to: 1000, by: 1] { std::list.append[lend.mut 's', value: 'k']; }
    let 'n' := std::list.length[lend 'r'];
}
