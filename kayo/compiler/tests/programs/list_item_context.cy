# A list literal's items take the element type as their context (owner,
# 2026-09-29: a): 0 -~ 1 in a list[u8] wraps at 8 bits to 255, 250 +~ 10
# to 4; 2 shl 3 in a list[u32] is 16 and not.bit 0 is 4294967295; a
# nested list passes it down (0 -~ 2 is 254); bare numbers alone stay exact
# (check claude's seed 197). examples/list_item_context.cpls as an exit
# code: one bit per right answer, 127.
%extern.C fn exit [$code: i32];
fn flag [$b: bool, $v: i32] -> i32 {
    if $b { return $v; }
    return 0;
}
MAIN {
    let $i: u8 = 5;
    let $xs: std::list[u8] = [0 minus.wrap 1, 0 minus.wrap $i, 250 plus.wrap 10];
    let $ys: std::list[u32] = [2 shl 3, not.bit 0];
    let $zs: std::list[std::list[u8]] = [[0 minus.wrap 2,], [1,]];
    let $big: std::list[u64] = [9692729768110883998 minus 3166240359165959856,];
    let $inner := copy $zs[pos: 1];
    no_check my.exit(flag($xs[pos: 1] equal_to 255, 1) plus flag($xs[pos: 2] equal_to 251, 2) plus flag($xs[pos: 3] equal_to 4, 4) plus flag($ys[pos: 1] equal_to 16, 8) plus flag($ys[pos: 2] equal_to 4294967295, 16) plus flag($inner[pos: 1] equal_to 254, 32) plus flag($big[pos: 1] equal_to 6526489408944924142, 64));
}
