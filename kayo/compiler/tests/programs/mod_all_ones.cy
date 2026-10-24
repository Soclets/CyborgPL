# A remainder is not monotone in the dividend: both ends of `u64` leave 0
# when divided by 255 or 65535 (2^64 - 1 is a multiple of each), and layer 1
# took the ends for the whole range and folded "always 0" (check claude,
# generator seed 106). `strlen` is extern, so its answer is any `u64`.
extern.C fn exit [$code: i32];
extern.C fn strlen [$s: ptr[u8]] -> u64;
MAIN {
    let $s := "three hundred characters are too many to type, so this is fewer";
    no_check let $n: u64 = strlen(lend $s);
    let $a: u64 = $n mod.floor 255;
    let $b: u64 = $n mod.floor 65535;
    no_check my.exit(to.i32($a plus $b));
}
