# Finding 2026-09-23 (generator seed 23, shrunk). Reversed by the owner, dc83c05:
# plain math is exact until it lands, so 112 > 224 is simply false and this runs.
%extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
MAIN {
    let 'v': i8 = 112;
    let 'yes' := "in ";
    if 'v' > ('v' plus 'v') { no_check write(1, lend 'yes', 3); }
    let 'end' := "end";
    no_check write(1, lend 'end', 3);
}
