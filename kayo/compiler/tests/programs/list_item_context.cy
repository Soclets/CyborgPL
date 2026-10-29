# A list literal's items take the element type as their context (owner,
# 2026-09-29: a): 0 -~ 1 in a list[u8] wraps at 8 bits to 255, 250 +~ 10
# to 4; 2 shl 3 in a list[u32] is 16 and not.bit 0 is 4294967295; a
# nested list passes it down (0 -~ 2 is 254); bare numbers alone stay exact
# (check claude's seed 197). examples/list_item_context.cpls as an exit
# code: one bit per right answer, 127.
%extern.C proc exit [let $code: i32];
fn flag [let $b: bool, let $v: i32] -> i32:
    if $b:
        ret $v;
    ;if
    ret 0;
;fn flag
MAIN:
    let $i: u8 := 5;
    let $xs: std::ls[u8] := [0 minus.wrap 1, 0 minus.wrap $i, 250 plus.wrap 10];
    let $ys: std::ls[u32] := [2 shl 3, not.bit 0];
    let $zs: std::ls[std::ls[u8]] := [[0 minus.wrap 2,], [1,]];
    let $big: std::ls[u64] := [9692729768110883998 - 3166240359165959856,];
    let $inner := copy $zs[pos: 1];
    no_check my.exit(flag($xs[pos: 1] == 255, 1) + flag($xs[pos: 2] == 251, 2) + flag($xs[pos: 3] == 4, 4) + flag($ys[pos: 1] == 16, 8) + flag($ys[pos: 2] == 4294967295, 16) + flag($inner[pos: 1] == 254, 32) + flag($big[pos: 1] == 6526489408944924142, 64));
;MAIN
