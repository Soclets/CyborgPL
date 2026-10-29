# Low level: raw memory from C, bits, a wrapping hash, SIMD, C layout, and the OS through `write`.
# a block of C's functions (owner, 2026-10-02); one alone may be a line, `extern.C fn exit [$code: i32];`
%extern.C:
    proc write [$fd: i32, $buf: ptr[u8], $len: u64] -> i64;
    proc malloc [$bytes: u64] -> ptr.mut[u8];
    proc free [$p: ptr.mut[u8]];
;%extern.C

%layout.C rec Header:
    var magic: u32;
    var flags: u8;
    var length: u64;
;%layout.C rec Header

# FNV-1a: the wrap *is* the algorithm, so `times.wrap`, never a check
fn fnv1a [lent $bytes: arr[u8]] -> u64:
    var $h: u64 = 14695981039346656037;
    for $b in lend $bytes:
        change $h to ($h xor to.u64($b)) times.wrap 1099511628211;
    ;for
    ret $h;
;fn fnv1a

# count the 1 bits, one shift at a time
fn popcount [$x: u32] -> u32:
    var $n: u32 = $x;
    var $count: u32 = 0;
    while $n > 0:
        change $count to $count + ($n and.bit 1);
        change $n to $n shr 1;
    ;while
    ret $count;
;fn popcount

# 16 bytes at once: is every byte ASCII (below 128)?
fn all_ascii [lent $bytes: arr[u8]] -> bool:
    let $v: simd[u8, count: 16] := simd.load(lend $bytes, pos: 1);
    ret simd.all($v < simd.fill(value: 128));
;fn all_ascii

MAIN:
    # 8 bytes nobody owns: C's memory, written and read through a pointer
    no_check let $raw := malloc(8);
    for $i [from: 0, to: 7, by: 1]:
        no_check ptr.write(ptr.at($raw, by: $i), value: 65 + $i);          # 'A' 'B' 'C' …
    ;for
    no_check let $third := ptr.read(ptr.at($raw, by: 2));
    no_check free($raw);

    let $text: arr[u8] := [104, 101, 108, 108, 111, 44, 32, 108, 111, 119, 32, 108, 101, 118, 101, 108];
    let $msg := "straight to fd 1 via write(2)";
    no_check let $n := write(1, lend $msg, std::str.bytes(lend $msg));

    std::print(newln,
        "third raw byte: ", $third, newln,
        "fnv1a:          ", fnv1a(lend $text), newln,
        "popcount(0xF0F0): ", popcount(61680), newln,
        "all ascii:      ", all_ascii(lend $text), newln,
        "Header: ", size.of[Header], " bytes, aligned to ", align.of[Header], newln);
;MAIN
