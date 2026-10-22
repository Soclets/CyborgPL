# A kit's operators may carry its path or not (owner, 2026-10-22: "It's always optional"), so std's word operators
# may be written with `std::`, as `std::pow` has since 2026-10-02.

MAIN {
    let 'a': i32 = 2 std::plus 3 std::times 4;
    let 'b': i32 = 7 std::div.floor 2;
    let 'c': u32 = 1 std::shl 3;
    std::print.useful['a', " ", 'b', " ", 'c', newln];
}
