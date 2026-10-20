# std marks everything `bare.kit_prefix.optional` (owner, 2026-10-06), so its word operators may carry `std::`
# as `std::pow` has since 2026-10-02.

MAIN {
    let 'a': i32 = 2 std::plus 3 std::times 4;
    let 'b': i32 = 7 std::div.floor 2;
    let 'c': u32 = 1 std::shl 3;
    std::print.useful['a', " ", 'b', " ", 'c', newln];
}
