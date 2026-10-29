# Rounding (design/conversion.md; owner, 2026-10-17: "(c)", one family; 2026-10-20: overloading): round.near
# (a tie to even), round.half_up (a tie away from zero), round.floor, round.ceil, round.trunc, each `($x)`
# for a whole number or `($x, places: N)`, one version per type, and the answer is the kind that went in.
#
# A bin is rounded from its exact digits (its stored form, every digit), so `round.near(2.675: f64,
# places: 2)` is 2.67, the bin being a little under 2.675; the rounded digits are then read back as the
# nearest bin of the same width. A whole number is a division by 1 that rounds as named; a bin at or past 2
# to the power of its fraction bits (1024 for a f16, 2^23 for a f32, 2^52 for a f64) is whole already
# and comes back as it is. A deci keeps the places it is rounded to, as decis do ("2.500"), as far as its
# digits go; the decis are the Money limb's, which std lists (owner, 2026-10-20: "(a)").

# Which way a cut goes, for rounding a bin's digits as text.
union.tag Way:
    nearest_even; half_away; down; up; toward_zero;
;union.tag Way

# Round to a whole number, or to `places:` places, a tie to even.
pub coll.fn near:
    versions near:
        [let $x: f16] -> f16:
            if $x >= 1024.0:
                ret $x;
            ;if
            if $x <= -1024.0:
                ret $x;
            ;if
            let $t := $x div.trunc 1.0;
            let $r := $x - $t;
            if $r > 0.5 or ($r == 0.5 and ($t mod.floor 2.0) != 0.0):
                ret $t + 1.0;
            ;if
            if $r < -0.5 or ($r == -0.5 and ($t mod.floor 2.0) != 0.0):
                ret $t - 1.0;
            ;if
            ret $t;
        ;version
        [let $x: f16, let $places: i32[min: 0]] -> f16:
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.nearest_even);
            ret to.f16(lend $cut) ?? bug "a rounded bin's digits did not read back";
        ;version
        [let $x: f32] -> f32:
            if $x >= 8388608.0:
                ret $x;
            ;if
            if $x <= -8388608.0:
                ret $x;
            ;if
            let $t := $x div.trunc 1.0;
            let $r := $x - $t;
            if $r > 0.5 or ($r == 0.5 and ($t mod.floor 2.0) != 0.0):
                ret $t + 1.0;
            ;if
            if $r < -0.5 or ($r == -0.5 and ($t mod.floor 2.0) != 0.0):
                ret $t - 1.0;
            ;if
            ret $t;
        ;version
        [let $x: f32, let $places: i32[min: 0]] -> f32:
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.nearest_even);
            ret to.f32(lend $cut) ?? bug "a rounded bin's digits did not read back";
        ;version
        [let $x: f64] -> f64:
            if $x >= 4503599627370496.0:
                ret $x;
            ;if
            if $x <= -4503599627370496.0:
                ret $x;
            ;if
            let $t := $x div.trunc 1.0;
            let $r := $x - $t;
            if $r > 0.5 or ($r == 0.5 and ($t mod.floor 2.0) != 0.0):
                ret $t + 1.0;
            ;if
            if $r < -0.5 or ($r == -0.5 and ($t mod.floor 2.0) != 0.0):
                ret $t - 1.0;
            ;if
            ret $t;
        ;version
        [let $x: f64, let $places: i32[min: 0]] -> f64:
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.nearest_even);
            ret to.f64(lend $cut) ?? bug "a rounded bin's digits did not read back";
        ;version
        [let $x: %d32] -> %d32:
            ret my.round.whole_d32($x, Way.nearest_even);
        ;version
        [let $x: %d32, let $places: i32[min: 0]] -> %d32:
            ret my.round.places_d32($x, $places, Way.nearest_even);
        ;version
        [let $x: %d64] -> %d64:
            ret my.round.whole_d64($x, Way.nearest_even);
        ;version
        [let $x: %d64, let $places: i32[min: 0]] -> %d64:
            ret my.round.places_d64($x, $places, Way.nearest_even);
        ;version
        [let $x: %d128] -> %d128:
            ret my.round.whole_d128($x, Way.nearest_even);
        ;version
        [let $x: %d128, let $places: i32[min: 0]] -> %d128:
            ret my.round.places_d128($x, $places, Way.nearest_even);
        ;version
    ;versions near
;coll.fn near

# Round to a whole number, or to `places:` places, a tie away from zero.
pub coll.fn half_up:
    versions half_up:
        [let $x: f16] -> f16:
            if $x >= 1024.0:
                ret $x;
            ;if
            if $x <= -1024.0:
                ret $x;
            ;if
            let $t := $x div.trunc 1.0;
            let $r := $x - $t;
            if $r >= 0.5:
                ret $t + 1.0;
            ;if
            if $r <= -0.5:
                ret $t - 1.0;
            ;if
            ret $t;
        ;version
        [let $x: f16, let $places: i32[min: 0]] -> f16:
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.half_away);
            ret to.f16(lend $cut) ?? bug "a rounded bin's digits did not read back";
        ;version
        [let $x: f32] -> f32:
            if $x >= 8388608.0:
                ret $x;
            ;if
            if $x <= -8388608.0:
                ret $x;
            ;if
            let $t := $x div.trunc 1.0;
            let $r := $x - $t;
            if $r >= 0.5:
                ret $t + 1.0;
            ;if
            if $r <= -0.5:
                ret $t - 1.0;
            ;if
            ret $t;
        ;version
        [let $x: f32, let $places: i32[min: 0]] -> f32:
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.half_away);
            ret to.f32(lend $cut) ?? bug "a rounded bin's digits did not read back";
        ;version
        [let $x: f64] -> f64:
            if $x >= 4503599627370496.0:
                ret $x;
            ;if
            if $x <= -4503599627370496.0:
                ret $x;
            ;if
            let $t := $x div.trunc 1.0;
            let $r := $x - $t;
            if $r >= 0.5:
                ret $t + 1.0;
            ;if
            if $r <= -0.5:
                ret $t - 1.0;
            ;if
            ret $t;
        ;version
        [let $x: f64, let $places: i32[min: 0]] -> f64:
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.half_away);
            ret to.f64(lend $cut) ?? bug "a rounded bin's digits did not read back";
        ;version
        [let $x: %d32] -> %d32:
            ret my.round.whole_d32($x, Way.half_away);
        ;version
        [let $x: %d32, let $places: i32[min: 0]] -> %d32:
            ret my.round.places_d32($x, $places, Way.half_away);
        ;version
        [let $x: %d64] -> %d64:
            ret my.round.whole_d64($x, Way.half_away);
        ;version
        [let $x: %d64, let $places: i32[min: 0]] -> %d64:
            ret my.round.places_d64($x, $places, Way.half_away);
        ;version
        [let $x: %d128] -> %d128:
            ret my.round.whole_d128($x, Way.half_away);
        ;version
        [let $x: %d128, let $places: i32[min: 0]] -> %d128:
            ret my.round.places_d128($x, $places, Way.half_away);
        ;version
    ;versions half_up
;coll.fn half_up

# Round to a whole number, or to `places:` places, down.
pub coll.fn floor:
    versions floor:
        [let $x: f16] -> f16:
            if $x >= 1024.0:
                ret $x;
            ;if
            if $x <= -1024.0:
                ret $x;
            ;if
            ret $x div.floor 1.0;
        ;version
        [let $x: f16, let $places: i32[min: 0]] -> f16:
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.down);
            ret to.f16(lend $cut) ?? bug "a rounded bin's digits did not read back";
        ;version
        [let $x: f32] -> f32:
            if $x >= 8388608.0:
                ret $x;
            ;if
            if $x <= -8388608.0:
                ret $x;
            ;if
            ret $x div.floor 1.0;
        ;version
        [let $x: f32, let $places: i32[min: 0]] -> f32:
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.down);
            ret to.f32(lend $cut) ?? bug "a rounded bin's digits did not read back";
        ;version
        [let $x: f64] -> f64:
            if $x >= 4503599627370496.0:
                ret $x;
            ;if
            if $x <= -4503599627370496.0:
                ret $x;
            ;if
            ret $x div.floor 1.0;
        ;version
        [let $x: f64, let $places: i32[min: 0]] -> f64:
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.down);
            ret to.f64(lend $cut) ?? bug "a rounded bin's digits did not read back";
        ;version
        [let $x: %d32] -> %d32:
            ret my.round.whole_d32($x, Way.down);
        ;version
        [let $x: %d32, let $places: i32[min: 0]] -> %d32:
            ret my.round.places_d32($x, $places, Way.down);
        ;version
        [let $x: %d64] -> %d64:
            ret my.round.whole_d64($x, Way.down);
        ;version
        [let $x: %d64, let $places: i32[min: 0]] -> %d64:
            ret my.round.places_d64($x, $places, Way.down);
        ;version
        [let $x: %d128] -> %d128:
            ret my.round.whole_d128($x, Way.down);
        ;version
        [let $x: %d128, let $places: i32[min: 0]] -> %d128:
            ret my.round.places_d128($x, $places, Way.down);
        ;version
    ;versions floor
;coll.fn floor

# Round to a whole number, or to `places:` places, up.
pub coll.fn ceil:
    versions ceil:
        [let $x: f16] -> f16:
            if $x >= 1024.0:
                ret $x;
            ;if
            if $x <= -1024.0:
                ret $x;
            ;if
            ret $x div.ceil 1.0;
        ;version
        [let $x: f16, let $places: i32[min: 0]] -> f16:
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.up);
            ret to.f16(lend $cut) ?? bug "a rounded bin's digits did not read back";
        ;version
        [let $x: f32] -> f32:
            if $x >= 8388608.0:
                ret $x;
            ;if
            if $x <= -8388608.0:
                ret $x;
            ;if
            ret $x div.ceil 1.0;
        ;version
        [let $x: f32, let $places: i32[min: 0]] -> f32:
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.up);
            ret to.f32(lend $cut) ?? bug "a rounded bin's digits did not read back";
        ;version
        [let $x: f64] -> f64:
            if $x >= 4503599627370496.0:
                ret $x;
            ;if
            if $x <= -4503599627370496.0:
                ret $x;
            ;if
            ret $x div.ceil 1.0;
        ;version
        [let $x: f64, let $places: i32[min: 0]] -> f64:
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.up);
            ret to.f64(lend $cut) ?? bug "a rounded bin's digits did not read back";
        ;version
        [let $x: %d32] -> %d32:
            ret my.round.whole_d32($x, Way.up);
        ;version
        [let $x: %d32, let $places: i32[min: 0]] -> %d32:
            ret my.round.places_d32($x, $places, Way.up);
        ;version
        [let $x: %d64] -> %d64:
            ret my.round.whole_d64($x, Way.up);
        ;version
        [let $x: %d64, let $places: i32[min: 0]] -> %d64:
            ret my.round.places_d64($x, $places, Way.up);
        ;version
        [let $x: %d128] -> %d128:
            ret my.round.whole_d128($x, Way.up);
        ;version
        [let $x: %d128, let $places: i32[min: 0]] -> %d128:
            ret my.round.places_d128($x, $places, Way.up);
        ;version
    ;versions ceil
;coll.fn ceil

# Round to a whole number, or to `places:` places, toward zero.
pub coll.fn trunc:
    versions trunc:
        [let $x: f16] -> f16:
            if $x >= 1024.0:
                ret $x;
            ;if
            if $x <= -1024.0:
                ret $x;
            ;if
            ret $x div.trunc 1.0;
        ;version
        [let $x: f16, let $places: i32[min: 0]] -> f16:
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.toward_zero);
            ret to.f16(lend $cut) ?? bug "a rounded bin's digits did not read back";
        ;version
        [let $x: f32] -> f32:
            if $x >= 8388608.0:
                ret $x;
            ;if
            if $x <= -8388608.0:
                ret $x;
            ;if
            ret $x div.trunc 1.0;
        ;version
        [let $x: f32, let $places: i32[min: 0]] -> f32:
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.toward_zero);
            ret to.f32(lend $cut) ?? bug "a rounded bin's digits did not read back";
        ;version
        [let $x: f64] -> f64:
            if $x >= 4503599627370496.0:
                ret $x;
            ;if
            if $x <= -4503599627370496.0:
                ret $x;
            ;if
            ret $x div.trunc 1.0;
        ;version
        [let $x: f64, let $places: i32[min: 0]] -> f64:
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.toward_zero);
            ret to.f64(lend $cut) ?? bug "a rounded bin's digits did not read back";
        ;version
        [let $x: %d32] -> %d32:
            ret my.round.whole_d32($x, Way.toward_zero);
        ;version
        [let $x: %d32, let $places: i32[min: 0]] -> %d32:
            ret my.round.places_d32($x, $places, Way.toward_zero);
        ;version
        [let $x: %d64] -> %d64:
            ret my.round.whole_d64($x, Way.toward_zero);
        ;version
        [let $x: %d64, let $places: i32[min: 0]] -> %d64:
            ret my.round.places_d64($x, $places, Way.toward_zero);
        ;version
        [let $x: %d128] -> %d128:
            ret my.round.whole_d128($x, Way.toward_zero);
        ;version
        [let $x: %d128, let $places: i32[min: 0]] -> %d128:
            ret my.round.places_d128($x, $places, Way.toward_zero);
        ;version
    ;versions trunc
;coll.fn trunc

# A bin's stored digits ($t: a `-` perhaps, digits, a `.` and more digits perhaps, no exponent) cut to
# $n places the way $way says.
fn digits [lent $t: str.utf8, let $n: i32[min: 0], let $way: Way] -> str.utf8:
    var $whole: ls[u8] = [];
    var $frac: ls[u8] = [];
    var $neg = false;
    var $after_point = false;
    for $b in str.each.byte(lend $t):
        if $b == 45:
            change $neg to true;
        else if $b == 46:
            change $after_point to true;
        else if $after_point:
            ls.append(lend.mut $frac, value: $b);
        else:
            ls.append(lend.mut $whole, value: $b);
        ;if
    ;for
    let $places: u64 := to.u64($n);
    if ls.len(lend $frac) <= $places:
        ret copy $t;
    ;if

    # What is kept, and what is cut off.
    var $kept: ls[u8] = copy $whole;
    var $rest: ls[u8] = [];
    for $i, $d in lend $frac:
        if $i <= $places:
            ls.append(lend.mut $kept, value: $d);
        else:
            ls.append(lend.mut $rest, value: $d);
        ;if
    ;for
    var $first : u8 = 48;
    var $rest_nonzero = false;
    var $after_first_nonzero = false;
    for $i, $d in lend $rest:
        if $i == 1:
            change $first to $d;
        else if $d != 48:
            change $after_first_nonzero to true;
        ;if
        if $d != 48:
            change $rest_nonzero to true;
        ;if
    ;for
    let $last_odd := (ls.len(lend $kept) >= 1) and.then ($kept[pos: ls.len(lend $kept)] mod.floor 2) == 1;
    let $up := match $way:
        case nearest_even:
            is $first > 53 or ($first == 53 and ($after_first_nonzero or $last_odd));
        ;case nearest_even
        case half_away:
            is $first >= 53;
        ;case half_away
        case down:
            is $neg and $rest_nonzero;
        ;case down
        case up:
            is not $neg and $rest_nonzero;
        ;case up
        case toward_zero:
            is false;
        ;case toward_zero
    ;match;

    # Add one at the last kept digit, carrying: every 9 after the last digit that is not a 9 becomes 0, and
    # that digit goes up one; if every digit is a 9, a 1 comes in front.
    var $last_below_9: u64 = 0;
    for $i, $d in lend $kept:
        if $d < 57:
            change $last_below_9 to $i;
        ;if
    ;for
    var $whole_len: u64 = ls.len(lend $whole);
    var $digits: ls[u8] = [];
    if $up and $last_below_9 == 0:
        ls.append(lend.mut $digits, value: 49);
        change $whole_len to $whole_len + 1;
    ;if
    for $i, $d in lend $kept:
        if not $up or $i < $last_below_9:
            ls.append(lend.mut $digits, value: $d);
        else if $i == $last_below_9:
            if $d < 57:
                ls.append(lend.mut $digits, value: $d + 1);
            else:
                ls.append(lend.mut $digits, value: 48);
            ;if
        else:
            ls.append(lend.mut $digits, value: 48);
        ;if
    ;for

    # Back to text: the sign only if something is left, then the point after the whole digits.
    var $nonzero = false;
    for $d in lend $digits:
        if $d != 48:
            change $nonzero to true;
        ;if
    ;for
    var $out: ls[u8] = [];
    if $neg and $nonzero:
        ls.append(lend.mut $out, value: 45);
    ;if
    for $i, $d in lend $digits:
        if $i - 1 == $whole_len:
            ls.append(lend.mut $out, value: 46);
        ;if
        ls.append(lend.mut $out, value: $d);
    ;for
    ret to.str.utf8(lend $out) ?? bug "the rounded digits were not UTF-8";
;fn digits


# A d32 rounded to a whole number the way $way says. One at or past 10000000 has no places left.
fn whole_d32 [let $x: %d32, let $way: Way] -> %d32:
    let $big: %d32 := exactly 10000000;
    if $x >= $big:
        ret $x;
    ;if
    if $x <= -$big:
        ret $x;
    ;if
    let $one: %d32 := exactly 1;
    let $two: %d32 := exactly 2;
    let $none: %d32 := exactly 0;
    let $half: %d32 := exactly 0.5;
    let $t := $x div.trunc $one;
    let $r := $x - $t;
    let $odd := ($t mod.floor $two) != $none;
    match $way:
        case toward_zero:
            ret $t;
        ;case toward_zero
        case down:
            ret $x div.floor $one;
        ;case down
        case up:
            ret $x div.ceil $one;
        ;case up
        case half_away:
            if $r >= $half:
                ret $t + $one;
            ;if
            if $r <= -$half:
                ret $t - $one;
            ;if
            ret $t;
        ;case half_away
        case nearest_even:
            if $r > $half:
                ret $t + $one;
            ;if
            if $r < -$half:
                ret $t - $one;
            ;if
            if $r == $half and $odd:
                ret $t + $one;
            ;if
            if $r == -$half and $odd:
                ret $t - $one;
            ;if
            ret $t;
        ;case nearest_even
    ;match
;fn whole_d32

# A d32 rounded to $n places the way $way says, keeping $n places as a deci does ("2.500") as far as
# its digits go. The point moves right until the number is whole or $n places are passed.
fn places_d32 [let $x: %d32, let $n: i32[min: 0], let $way: Way] -> %d32:
    let $one: %d32 := exactly 1;
    let $ten: %d32 := exactly 10;
    let $tenth: %d32 := exactly 0.1;
    var $limit: i32 = $n;
    if $limit > 110:
        change $limit to 110;
    ;if
    var $y = $x;
    var $k: i32 = 0;
    while $k < $limit and $y != ($y div.trunc $one):
        change $y to $y x $ten;
        change $k to $k + 1;
    ;while
    var $whole = $y div.trunc $one;
    if $y != $whole:
        change $whole to my.round.whole_d32($y, $way);
    ;if
    var $scale = $one;
    for $i [from: 1, to: $k, by: 1]:
        change $scale to $scale x $ten;
    ;for
    var $zero: %d32 = exactly 0;
    for $i [from: 1, to: $limit, by: 1]:
        change $zero to $zero x $tenth;
    ;for
    ret ($whole div.near $scale) + $zero;
;fn places_d32

# A d64 rounded to a whole number the way $way says. One at or past 10000000000000000 has no places left.
fn whole_d64 [let $x: %d64, let $way: Way] -> %d64:
    let $big: %d64 := exactly 10000000000000000;
    if $x >= $big:
        ret $x;
    ;if
    if $x <= -$big:
        ret $x;
    ;if
    let $one: %d64 := exactly 1;
    let $two: %d64 := exactly 2;
    let $none: %d64 := exactly 0;
    let $half: %d64 := exactly 0.5;
    let $t := $x div.trunc $one;
    let $r := $x - $t;
    let $odd := ($t mod.floor $two) != $none;
    match $way:
        case toward_zero:
            ret $t;
        ;case toward_zero
        case down:
            ret $x div.floor $one;
        ;case down
        case up:
            ret $x div.ceil $one;
        ;case up
        case half_away:
            if $r >= $half:
                ret $t + $one;
            ;if
            if $r <= -$half:
                ret $t - $one;
            ;if
            ret $t;
        ;case half_away
        case nearest_even:
            if $r > $half:
                ret $t + $one;
            ;if
            if $r < -$half:
                ret $t - $one;
            ;if
            if $r == $half and $odd:
                ret $t + $one;
            ;if
            if $r == -$half and $odd:
                ret $t - $one;
            ;if
            ret $t;
        ;case nearest_even
    ;match
;fn whole_d64

# A d64 rounded to $n places the way $way says, keeping $n places as a deci does ("2.500") as far as
# its digits go. The point moves right until the number is whole or $n places are passed.
fn places_d64 [let $x: %d64, let $n: i32[min: 0], let $way: Way] -> %d64:
    let $one: %d64 := exactly 1;
    let $ten: %d64 := exactly 10;
    let $tenth: %d64 := exactly 0.1;
    var $limit: i32 = $n;
    if $limit > 400:
        change $limit to 400;
    ;if
    var $y = $x;
    var $k: i32 = 0;
    while $k < $limit and $y != ($y div.trunc $one):
        change $y to $y x $ten;
        change $k to $k + 1;
    ;while
    var $whole = $y div.trunc $one;
    if $y != $whole:
        change $whole to my.round.whole_d64($y, $way);
    ;if
    var $scale = $one;
    for $i [from: 1, to: $k, by: 1]:
        change $scale to $scale x $ten;
    ;for
    var $zero: %d64 = exactly 0;
    for $i [from: 1, to: $limit, by: 1]:
        change $zero to $zero x $tenth;
    ;for
    ret ($whole div.near $scale) + $zero;
;fn places_d64

# A d128 rounded to a whole number the way $way says. One at or past 10000000000000000000000000000000000 has no places left.
fn whole_d128 [let $x: %d128, let $way: Way] -> %d128:
    let $big: %d128 := exactly 10000000000000000000000000000000000;
    if $x >= $big:
        ret $x;
    ;if
    if $x <= -$big:
        ret $x;
    ;if
    let $one: %d128 := exactly 1;
    let $two: %d128 := exactly 2;
    let $none: %d128 := exactly 0;
    let $half: %d128 := exactly 0.5;
    let $t := $x div.trunc $one;
    let $r := $x - $t;
    let $odd := ($t mod.floor $two) != $none;
    match $way:
        case toward_zero:
            ret $t;
        ;case toward_zero
        case down:
            ret $x div.floor $one;
        ;case down
        case up:
            ret $x div.ceil $one;
        ;case up
        case half_away:
            if $r >= $half:
                ret $t + $one;
            ;if
            if $r <= -$half:
                ret $t - $one;
            ;if
            ret $t;
        ;case half_away
        case nearest_even:
            if $r > $half:
                ret $t + $one;
            ;if
            if $r < -$half:
                ret $t - $one;
            ;if
            if $r == $half and $odd:
                ret $t + $one;
            ;if
            if $r == -$half and $odd:
                ret $t - $one;
            ;if
            ret $t;
        ;case nearest_even
    ;match
;fn whole_d128

# A d128 rounded to $n places the way $way says, keeping $n places as a deci does ("2.500") as far as
# its digits go. The point moves right until the number is whole or $n places are passed.
fn places_d128 [let $x: %d128, let $n: i32[min: 0], let $way: Way] -> %d128:
    let $one: %d128 := exactly 1;
    let $ten: %d128 := exactly 10;
    let $tenth: %d128 := exactly 0.1;
    var $limit: i32 = $n;
    if $limit > 6200:
        change $limit to 6200;
    ;if
    var $y = $x;
    var $k: i32 = 0;
    while $k < $limit and $y != ($y div.trunc $one):
        change $y to $y x $ten;
        change $k to $k + 1;
    ;while
    var $whole = $y div.trunc $one;
    if $y != $whole:
        change $whole to my.round.whole_d128($y, $way);
    ;if
    var $scale = $one;
    for $i [from: 1, to: $k, by: 1]:
        change $scale to $scale x $ten;
    ;for
    var $zero: %d128 = exactly 0;
    for $i [from: 1, to: $limit, by: 1]:
        change $zero to $zero x $tenth;
    ;for
    ret ($whole div.near $scale) + $zero;
;fn places_d128
