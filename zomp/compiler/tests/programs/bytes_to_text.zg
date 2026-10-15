# Bytes read as text (owner, 2026-09-30: "yea"): `to.str.utf8` reads a
# list or array of uint8 as UTF-8, and fails `not_utf8` when they are not.
# 104 195 169 108 108 111 is "héllo": 5 points, 6 bytes; 255 is never
# UTF-8, so 'b' is "not" (3 bytes); the array is "hi" (2 bytes).
# 5 x 16 + 6 + 3 + 2 = 91.
extern.C fn exit ['code': int32];
MAIN {
    let 'good': list[uint8] = [104, 195, 169, 108, 108, 111];
    let 'bad': list[uint8] = [104, 255, 105];
    let 'arr': array[uint8] = [104, 105];
    let 'a' := to.str.utf8['good'] otherwise "?";
    let 'b' := to.str.utf8['bad'] otherwise "not";
    let 'c' := to.str.utf8['arr'] otherwise "?";
    let 'n': uint64 = str.points[lend 'a'] times 16 plus str.bytes[lend 'a'] plus str.bytes[lend 'b'] plus str.bytes[lend 'c'];
    no_check my.exit[to.int32['n']];
}
