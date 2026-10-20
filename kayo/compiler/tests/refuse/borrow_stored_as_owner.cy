# refuse: here it would be a second owner of what it borrows
# A borrow stored into a variable that owns its value.
fn f [lent 'a': list[i8]] -> lent list[i8] { return lend 'a'; }
MAIN {
    let mut 's': list[i8] = [1, 2];
    let mut 'o': list[i8] = [3,];
    change 'o' = f[lend 's'];
}
