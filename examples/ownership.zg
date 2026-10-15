# Ownership: `lend`, `lend.mut`, `give` and `copy`.
fn count [lent 'items': list[str.utf8]] -> int32 {
    let 'n' := 0;
    for 'item' in lend 'items' {
        change 'n' = 'n' plus 1;
    }
    return 'n';
}

fn shout [lent.mut 'text': str.utf8] {
    change 'text' = copy 'text' ++ "!";
}

fn greet ['name': str.utf8] -> str.utf8 {
    return "hello " ++ 'name';
}

MAIN {
    let 's' := "line 1" ++ newln ++ "line 2";
    let 't' := give 's';
    let 'u' := copy 't';
    shout[lend.mut 'u'];
    std::print.stored['t', newln];
    std::print.stored['u', newln];
    let 'g' := greet[copy 'u'];
    std::print.stored['g', newln];
    std::print.stored['u', newln];
    let 'words' := ["a", "b", "c"];
    let 'n' := count[lend 'words'];
    std::print.useful['n', " words", newln];
    let 'm' := count[["x", "y"]];
    std::print.useful['m', " fresh", newln];
    let 'r' := lend 'words';
    std::print.useful['r', newln];
}
