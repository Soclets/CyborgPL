# A slice whose end is not known is checked at run time: past the end
# of the list, it stops. `strlen` is extern, so 'k' is any `u64`.
%extern.C proc exit [$code: i32];
%extern.C proc strlen [$s: ptr[u8]] -> u64;
MAIN:
    let $s := "nine char";
    no_check let $k: u64 := strlen(lend $s);
    let $xs: std::ls[i64] := [1, 2, 3];
    let $part := std::ls.slice(lend $xs, from: pos 1, to: pos $k);
    no_check my.exit(to.i32(std::ls.len(lend $part)));
;MAIN
