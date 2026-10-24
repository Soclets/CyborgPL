# C's numbers are checked wherever they come in (owner, 2026-09-27): a
# `ptr.read` of a C enum, and what C wrote through a `ptr.mut` lent to it,
# checked when the call ends; `NO_FUCKING_CHECKS` trusts them instead. The
# oracle cannot link calloc, memset or memcpy, so the answer is worked out
# by hand: calloc's 0 is no key, after the write the read is b, memset's
# 0x41414141 is no key, memcpy's b and a are keys, and the trusted read is b
# — each right answer is one bit, 1 + 2 + 4 + 8 + 16 = 31.
extern.C fn exit [$code: i32];
extern.C fn calloc [$n: u64, $size: u64] -> ptr.mut[Key];
extern.C fn free [$p: ptr.mut[Key]];
extern.C fn memset [$s: ptr.mut[Key], $c: i32, $n: u64] -> ptr.mut[Key];
extern.C fn memcpy [$d: ptr.mut[Key], $s: ptr[Key], $n: u64] -> ptr.mut[Key];
layout.C enum Key { a = 65, b = 66, escape = 256 }

fn num [$k: Key] -> i32 {
    match $k {
        a { return 1; }
        b { return 2; }
        escape { return 3; }
    }
}

fn flag [$b: bool, $v: i32] -> i32 {
    if $b { return $v; }
    return 0;
}

fn by_memset [] -> i32 {
    let mut $keys: std::list[Key] = [Key.a, Key.a];
    no_check let $q := memset(lend.mut $keys, 65, 8) otherwise { out_of_range { return 3; } };
    return 0;
}

fn by_memcpy [] -> i32 {
    let $src: std::list[Key] = [Key.b, Key.a];
    let mut $dst: std::list[Key] = [Key.escape, Key.escape];
    no_check let $q := memcpy(lend.mut $dst, lend $src, 8) otherwise { out_of_range { return 9; } };
    return num(copy $dst[pos: 1]) plus num(copy $dst[pos: 2]) times 2;
}

MAIN {
    no_check let $p := calloc(1, 4);
    no_check let $zero := ptr.read($p) otherwise Key.escape;
    no_check ptr.write($p, value: Key.b);
    no_check let $two := ptr.read($p) otherwise Key.escape;
    NO_FUCKING_CHECKS let $trusted := ptr.read($p);
    no_check free($p);
    let $bits := flag(num(give $zero) equal_to 3, 1) plus flag(num(give $two) equal_to 2, 2) plus flag(by_memset() equal_to 3, 4) plus flag(by_memcpy() equal_to 4, 8) plus flag(num(give $trusted) equal_to 2, 16);
    no_check my.exit($bits);
}
