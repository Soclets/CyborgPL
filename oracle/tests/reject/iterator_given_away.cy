struct Counter { now: i32, stop: i32 }
fn Counter.next [lent.mut 'c': Counter] -> optional[i32] {
    if 'c'.now equal_to 'c'.stop { return optional.none; }
    change 'c'.now = 'c'.now plus 1;
    return optional.value['c'.now];
}
MAIN {
    let mut 'c' := Counter[now = 0, stop = 2];
    for 'x' in give 'c' { }
    let 'again' := Counter.next[lend.mut 'c'];
}
