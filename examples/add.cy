# A function that adds two numbers, called from MAIN.
fn add ['a': i32, 'b': i32] -> i32 {
    return 'a' + 'b';
}

MAIN {
    let 'sum' := add[10, 20];
    std::print.useful["sum: ", 'sum', newln];
}
