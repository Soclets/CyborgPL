fn half_start <T: any> [$x: <T>] -> <T> {
    let $h: <T> = 0.5;
    return $h;
}
MAIN {
    std::print.useful(half_start(4: i32), newln);
}
