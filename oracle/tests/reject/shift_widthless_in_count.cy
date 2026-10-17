# a shift's count gives nothing inside it a width (owner, 2026-10-17: "(a)")
MAIN {
    let 'r': int64 = to.int64[1] shl (5 shr to.uint8[1]);
    std::print.useful['r'];
}
