MAIN {
    let 's' := "hello";
    let 'i': i64 = 0;
    let 'b' := std::str.at.byte[lend 's', pos: 'i'] otherwise 0;
    std::print.useful['b', newln];
}
