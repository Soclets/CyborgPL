# Low level: raw memory from C, bits, a wrapping hash, SIMD, C layout, and the OS through `write`.
# a block of C's functions (owner, 2026-10-02); one alone may be a line, `extern.C fn exit [$code: i32];`
%extern.C:
    proc write [let $fd: int.sign.32, let $buf: ptr[int.unsign.8], let $len: int.unsign.64] -> int.sign.64;
    proc malloc [let $bytes: int.unsign.64] -> ptr.mut[int.unsign.8];
    proc free [let $p: ptr.mut[int.unsign.8]];
end of %extern.C

%layout.C rec Header:
    var magic: int.unsign.32;
    var flags: int.unsign.8;
    var length: int.unsign.64;
end of rec Header

# FNV-1a: the wrap *is* the algorithm, so `x~`, never a check
fn fnv1a [lent $bytes: arr[int.unsign.8]] -> int.unsign.64:
    var $h: int.unsign.64 = 14695981039346656037;
    for $b in lend $bytes:
        change $h to ($h xor to.int.unsign.64($b)) times.wrap 1099511628211;
    end of for
    ret $h;
end of fn fnv1a

# count the 1 bits, one shift at a time
fn popcount [let $x: int.unsign.32] -> int.unsign.32:
    var $n: int.unsign.32 = $x;
    var $count: int.unsign.32 = 0;
    while $n greater_than 0:
        change $count to $count plus ($n and.bit 1);
        change $n to $n shr 1;
    end of while
    ret $count;
end of fn popcount

# 16 bytes at once: is every byte ASCII (below 128)?
fn all_ascii [lent $bytes: arr[int.unsign.8]] -> bool:
    let $v: simd[int.unsign.8, count: 16] := simd.load(lend $bytes, pos: 1);
    ret simd.all($v less_than simd.fill(value: 128));
end of fn all_ascii

!MAIN:
    # 8 bytes nobody owns: C's memory, written and read through a pointer
    !no_check let $raw := malloc(8);
    for $i [from: 0, to: 7, by: 1]:
        !no_check ptr.write(ptr.at($raw, by: $i), value: 65 plus $i);          # 'A' 'B' 'C' …
    end of for
    !no_check let $third := ptr.read(ptr.at($raw, by: 2));
    !no_check free($raw);

    let $text: arr[int.unsign.8] := [104, 101, 108, 108, 111, 44, 32, 108, 111, 119, 32, 108, 101, 118, 101, 108];
    let $msg := "straight to fd 1 via write(2)";
    !no_check let $n := write(1, lend $msg, std::str.bytes(lend $msg));

    std::print.shortest(newln,
        "third raw byte: ", $third, newln,
        "fnv1a:          ", fnv1a(lend $text), newln,
        "popcount(0xF0F0): ", popcount(61680), newln,
        "all ascii:      ", all_ascii(lend $text), newln,
        "Header: ", size.of[Header], " bytes, aligned to ", align.of[Header], newln);
!end of MAIN
