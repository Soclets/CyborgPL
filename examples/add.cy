# A function that adds two numbers, called from MAIN.
fn add ['a': int32, 'b': int32] -> int32 {
    return 'a' + 'b';
}

MAIN {
    let 'sum' := add[10, 20];
    std::print.useful["sum: ", 'sum', newln];
}
