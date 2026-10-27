# Low level: raw memory from C, bits, a wrapping hash, SIMD, C layout, and the OS through `write`.
# a block of C's functions (owner, 2026-10-02); one alone may be a line, `extern.C fn exit [$code: i32];`
%external.C:
    procedure write [$fd: integer.signed.32_bit, $buf: pointer[integer.unsigned.8_bit], $len: integer.unsigned.64_bit] -> integer.signed.64_bit;
    procedure malloc [$bytes: integer.unsigned.64_bit] -> pointer.mutable[integer.unsigned.8_bit];
    procedure free [$p: pointer.mutable[integer.unsigned.8_bit]];
end of %external.C

%layout.C record Header:
    magic: integer.unsigned.32_bit, flags: integer.unsigned.8_bit, length: integer.unsigned.64_bit
end of layout

# FNV-1a: the wrap *is* the algorithm, so `x~`, never a check
function fnv1a [lent $bytes: array[integer.unsigned.8_bit]] -> integer.unsigned.64_bit:
    variable $h: integer.unsigned.64_bit = 14695981039346656037;
    for $b in lend $bytes:
        change $h = ($h xor to.integer.unsigned.64_bit($b)) times.wrap 1099511628211;
    end of for
    return $h;
end of function fnv1a

# count the 1 bits, one shift at a time
function popcount [$x: integer.unsigned.32_bit] -> integer.unsigned.32_bit:
    variable $n: integer.unsigned.32_bit = $x;
    variable $count: integer.unsigned.32_bit = 0;
    while $n greater_than 0:
        change $count = $count plus ($n and.bit 1);
        change $n = $n shift_right 1;
    end of while
    return $count;
end of function popcount

# 16 bytes at once: is every byte ASCII (below 128)?
function all_ascii [lent $bytes: array[integer.unsigned.8_bit]] -> boolean:
    let $v: simd[integer.unsigned.8_bit, count: 16] = simd.load(lend $bytes, position: 1);
    return simd.all($v less_than simd.fill(value: 128));
end of function all_ascii

!MAIN:
    # 8 bytes nobody owns: C's memory, written and read through a pointer
    !no_check let $raw := malloc(8);
    for $i [from: 0, to: 7, by: 1]:
        !no_check pointer.write(pointer.at($raw, by: $i), value: 65 plus $i);          # 'A' 'B' 'C' …
    end of for
    !no_check let $third := pointer.read(pointer.at($raw, by: 2));
    !no_check free($raw);

    let $text: array[integer.unsigned.8_bit] = [104, 101, 108, 108, 111, 44, 32, 108, 111, 119, 32, 108, 101, 118, 101, 108];
    let $msg := "straight to fd 1 via write(2)";
    !no_check let $n := write(1, lend $msg, std::string.bytes(lend $msg));

    std::print.shortest(newline,
        "third raw byte: ", $third, newline,
        "fnv1a:          ", fnv1a(lend $text), newline,
        "popcount(0xF0F0): ", popcount(61680), newline,
        "all ascii:      ", all_ascii(lend $text), newline,
        "Header: ", size.of[Header], " bytes, aligned to ", align.of[Header], newline);
!end of MAIN
