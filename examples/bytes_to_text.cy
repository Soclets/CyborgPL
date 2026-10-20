# Bytes into text (owner, 2026-09-30: "yea"): `to.str.utf8` reads a list or array of u8 as UTF-8,
# and fails `not_utf8` when they are not. `é` is the two bytes 195, 169; 255 is never UTF-8.
MAIN {
    let 'good': list[u8] = [104, 195, 169, 108, 108, 111];
    let 'bad': list[u8] = [104, 255, 105];
    let 'a' := to.str.utf8['good'] otherwise "?";
    let 'b' := to.str.utf8['bad'] otherwise "not UTF-8";
    std::print.stored['a', newln, 'b', newln];
    std::print.stored[str.points[lend 'a'], " points, ", str.bytes[lend 'a'], " bytes", newln];
}
