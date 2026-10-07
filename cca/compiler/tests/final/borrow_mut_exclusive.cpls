# While a `lent.mut` borrow is in use, what it borrows is not read by anyone
# else.
fn pickmut [lent.mut 'a': list[int32]] -> lent.mut list[int32] { return lend.mut 'a'; }
MAIN {
    let mut 'xs': list[int32] = [1, 2];
    let 'm' := pickmut[lend.mut 'xs'];
    let 'y' := 'xs'[pos 1];
    change 'm'[pos 1] = 7;
}
