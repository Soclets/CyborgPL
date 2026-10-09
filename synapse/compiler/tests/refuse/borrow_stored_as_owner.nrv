# refuse: here it would be a second owner of what it borrows
# A borrow stored into a variable that owns its value.
fn f [lent 'a': list[int8]] -> lent list[int8] { return lend 'a'; }
MAIN {
    let mut 's': list[int8] = [1, 2];
    let mut 'o': list[int8] = [3,];
    change 'o' = f[lend 's'];
}
