# refuse: here it would be a second owner of what it borrows
# A borrow stored into a variable that owns its value.
fn f [lent 'a': std::list[i8]] -> lent std::list[i8] { return lend 'a'; }
MAIN {
    let mut 's': std::list[i8] = [1, 2];
    let mut 'o': std::list[i8] = [3,];
    change 'o' = f[lend 's'];
}
