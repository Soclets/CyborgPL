fn keep <T: any> [$v: <T>] -> <T> {
    let mut $x := $v;
    return $x;
}

MAIN {
    std::print.useful("ok", newln);
}
