# Bytes into text (owner, 2026-09-30: "yea"): `to.str.utf8` reads a list or array of u8 as UTF-8,
# and fails `not_utf8` when they are not. `é` is the two bytes 195, 169; 255 is never UTF-8.
!MAIN:
    let $good: std::list[integer.unsigned.8_bit] = [104, 195, 169, 108, 108, 111];
    let $bad: std::list[integer.unsigned.8_bit] = [104, 255, 105];
    let $a := to.string.utf8($good) otherwise "?";
    let $b := to.string.utf8($bad) otherwise "not UTF-8";
    std::print.stored($a, newline, $b, newline);
    std::print.stored(std::string.code_points(lend $a), " points, ", std::string.bytes(lend $a), " bytes", newline);
!end of MAIN
