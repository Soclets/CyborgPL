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
enum Way { nearest_even, half_away, down, up, toward_zero }

# Round to a whole number, or to `places:` places, a tie to even.
pub coll.fn near {
    versions near {
        [$x: f16] -> f16 {
            if $x at_least 1024.0 { return $x; }
            if $x at_most -1024.0 { return $x; }
            let $t := $x div.trunc 1.0;
            let $r := $x - $t;
            if $r > 0.5 or ($r equal_to 0.5 and ($t mod.floor 2.0) not_equal 0.0) { return $t + 1.0; }
            if $r < -0.5 or ($r equal_to -0.5 and ($t mod.floor 2.0) not_equal 0.0) { return $t - 1.0; }
            return $t;
        }
        [$x: f16, $places: i32[min: 0]] -> f16 {
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.nearest_even);
            return to.f16($cut) otherwise { not_a_number { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } out_of_range { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } };
        }
        [$x: f32] -> f32 {
            if $x at_least 8388608.0 { return $x; }
            if $x at_most -8388608.0 { return $x; }
            let $t := $x div.trunc 1.0;
            let $r := $x - $t;
            if $r > 0.5 or ($r equal_to 0.5 and ($t mod.floor 2.0) not_equal 0.0) { return $t + 1.0; }
            if $r < -0.5 or ($r equal_to -0.5 and ($t mod.floor 2.0) not_equal 0.0) { return $t - 1.0; }
            return $t;
        }
        [$x: f32, $places: i32[min: 0]] -> f32 {
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.nearest_even);
            return to.f32($cut) otherwise { not_a_number { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } out_of_range { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } };
        }
        [$x: f64] -> f64 {
            if $x at_least 4503599627370496.0 { return $x; }
            if $x at_most -4503599627370496.0 { return $x; }
            let $t := $x div.trunc 1.0;
            let $r := $x - $t;
            if $r > 0.5 or ($r equal_to 0.5 and ($t mod.floor 2.0) not_equal 0.0) { return $t + 1.0; }
            if $r < -0.5 or ($r equal_to -0.5 and ($t mod.floor 2.0) not_equal 0.0) { return $t - 1.0; }
            return $t;
        }
        [$x: f64, $places: i32[min: 0]] -> f64 {
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.nearest_even);
            return to.f64($cut) otherwise { not_a_number { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } out_of_range { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } };
        }
        [$x: d32] -> d32 { return my.round.whole_d32($x, Way.nearest_even); }
        [$x: d32, $places: i32[min: 0]] -> d32 { return my.round.places_d32($x, $places, Way.nearest_even); }
        [$x: d64] -> d64 { return my.round.whole_d64($x, Way.nearest_even); }
        [$x: d64, $places: i32[min: 0]] -> d64 { return my.round.places_d64($x, $places, Way.nearest_even); }
        [$x: d128] -> d128 { return my.round.whole_d128($x, Way.nearest_even); }
        [$x: d128, $places: i32[min: 0]] -> d128 { return my.round.places_d128($x, $places, Way.nearest_even); }
    }
}

# Round to a whole number, or to `places:` places, a tie away from zero.
pub coll.fn half_up {
    versions half_up {
        [$x: f16] -> f16 {
            if $x at_least 1024.0 { return $x; }
            if $x at_most -1024.0 { return $x; }
            let $t := $x div.trunc 1.0;
            let $r := $x - $t;
            if $r at_least 0.5 { return $t + 1.0; }
            if $r at_most -0.5 { return $t - 1.0; }
            return $t;
        }
        [$x: f16, $places: i32[min: 0]] -> f16 {
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.half_away);
            return to.f16($cut) otherwise { not_a_number { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } out_of_range { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } };
        }
        [$x: f32] -> f32 {
            if $x at_least 8388608.0 { return $x; }
            if $x at_most -8388608.0 { return $x; }
            let $t := $x div.trunc 1.0;
            let $r := $x - $t;
            if $r at_least 0.5 { return $t + 1.0; }
            if $r at_most -0.5 { return $t - 1.0; }
            return $t;
        }
        [$x: f32, $places: i32[min: 0]] -> f32 {
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.half_away);
            return to.f32($cut) otherwise { not_a_number { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } out_of_range { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } };
        }
        [$x: f64] -> f64 {
            if $x at_least 4503599627370496.0 { return $x; }
            if $x at_most -4503599627370496.0 { return $x; }
            let $t := $x div.trunc 1.0;
            let $r := $x - $t;
            if $r at_least 0.5 { return $t + 1.0; }
            if $r at_most -0.5 { return $t - 1.0; }
            return $t;
        }
        [$x: f64, $places: i32[min: 0]] -> f64 {
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.half_away);
            return to.f64($cut) otherwise { not_a_number { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } out_of_range { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } };
        }
        [$x: d32] -> d32 { return my.round.whole_d32($x, Way.half_away); }
        [$x: d32, $places: i32[min: 0]] -> d32 { return my.round.places_d32($x, $places, Way.half_away); }
        [$x: d64] -> d64 { return my.round.whole_d64($x, Way.half_away); }
        [$x: d64, $places: i32[min: 0]] -> d64 { return my.round.places_d64($x, $places, Way.half_away); }
        [$x: d128] -> d128 { return my.round.whole_d128($x, Way.half_away); }
        [$x: d128, $places: i32[min: 0]] -> d128 { return my.round.places_d128($x, $places, Way.half_away); }
    }
}

# Round to a whole number, or to `places:` places, down.
pub coll.fn floor {
    versions floor {
        [$x: f16] -> f16 {
            if $x at_least 1024.0 { return $x; }
            if $x at_most -1024.0 { return $x; }
            return $x div.floor 1.0;
        }
        [$x: f16, $places: i32[min: 0]] -> f16 {
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.down);
            return to.f16($cut) otherwise { not_a_number { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } out_of_range { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } };
        }
        [$x: f32] -> f32 {
            if $x at_least 8388608.0 { return $x; }
            if $x at_most -8388608.0 { return $x; }
            return $x div.floor 1.0;
        }
        [$x: f32, $places: i32[min: 0]] -> f32 {
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.down);
            return to.f32($cut) otherwise { not_a_number { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } out_of_range { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } };
        }
        [$x: f64] -> f64 {
            if $x at_least 4503599627370496.0 { return $x; }
            if $x at_most -4503599627370496.0 { return $x; }
            return $x div.floor 1.0;
        }
        [$x: f64, $places: i32[min: 0]] -> f64 {
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.down);
            return to.f64($cut) otherwise { not_a_number { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } out_of_range { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } };
        }
        [$x: d32] -> d32 { return my.round.whole_d32($x, Way.down); }
        [$x: d32, $places: i32[min: 0]] -> d32 { return my.round.places_d32($x, $places, Way.down); }
        [$x: d64] -> d64 { return my.round.whole_d64($x, Way.down); }
        [$x: d64, $places: i32[min: 0]] -> d64 { return my.round.places_d64($x, $places, Way.down); }
        [$x: d128] -> d128 { return my.round.whole_d128($x, Way.down); }
        [$x: d128, $places: i32[min: 0]] -> d128 { return my.round.places_d128($x, $places, Way.down); }
    }
}

# Round to a whole number, or to `places:` places, up.
pub coll.fn ceil {
    versions ceil {
        [$x: f16] -> f16 {
            if $x at_least 1024.0 { return $x; }
            if $x at_most -1024.0 { return $x; }
            return $x div.ceil 1.0;
        }
        [$x: f16, $places: i32[min: 0]] -> f16 {
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.up);
            return to.f16($cut) otherwise { not_a_number { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } out_of_range { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } };
        }
        [$x: f32] -> f32 {
            if $x at_least 8388608.0 { return $x; }
            if $x at_most -8388608.0 { return $x; }
            return $x div.ceil 1.0;
        }
        [$x: f32, $places: i32[min: 0]] -> f32 {
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.up);
            return to.f32($cut) otherwise { not_a_number { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } out_of_range { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } };
        }
        [$x: f64] -> f64 {
            if $x at_least 4503599627370496.0 { return $x; }
            if $x at_most -4503599627370496.0 { return $x; }
            return $x div.ceil 1.0;
        }
        [$x: f64, $places: i32[min: 0]] -> f64 {
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.up);
            return to.f64($cut) otherwise { not_a_number { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } out_of_range { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } };
        }
        [$x: d32] -> d32 { return my.round.whole_d32($x, Way.up); }
        [$x: d32, $places: i32[min: 0]] -> d32 { return my.round.places_d32($x, $places, Way.up); }
        [$x: d64] -> d64 { return my.round.whole_d64($x, Way.up); }
        [$x: d64, $places: i32[min: 0]] -> d64 { return my.round.places_d64($x, $places, Way.up); }
        [$x: d128] -> d128 { return my.round.whole_d128($x, Way.up); }
        [$x: d128, $places: i32[min: 0]] -> d128 { return my.round.places_d128($x, $places, Way.up); }
    }
}

# Round to a whole number, or to `places:` places, toward zero.
pub coll.fn trunc {
    versions trunc {
        [$x: f16] -> f16 {
            if $x at_least 1024.0 { return $x; }
            if $x at_most -1024.0 { return $x; }
            return $x div.trunc 1.0;
        }
        [$x: f16, $places: i32[min: 0]] -> f16 {
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.toward_zero);
            return to.f16($cut) otherwise { not_a_number { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } out_of_range { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } };
        }
        [$x: f32] -> f32 {
            if $x at_least 8388608.0 { return $x; }
            if $x at_most -8388608.0 { return $x; }
            return $x div.trunc 1.0;
        }
        [$x: f32, $places: i32[min: 0]] -> f32 {
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.toward_zero);
            return to.f32($cut) otherwise { not_a_number { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } out_of_range { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } };
        }
        [$x: f64] -> f64 {
            if $x at_least 4503599627370496.0 { return $x; }
            if $x at_most -4503599627370496.0 { return $x; }
            return $x div.trunc 1.0;
        }
        [$x: f64, $places: i32[min: 0]] -> f64 {
            let $text := to.stored.str.utf8($x);
            let $cut := my.round.digits(lend $text, $places, Way.toward_zero);
            return to.f64($cut) otherwise { not_a_number { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } out_of_range { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } };
        }
        [$x: d32] -> d32 { return my.round.whole_d32($x, Way.toward_zero); }
        [$x: d32, $places: i32[min: 0]] -> d32 { return my.round.places_d32($x, $places, Way.toward_zero); }
        [$x: d64] -> d64 { return my.round.whole_d64($x, Way.toward_zero); }
        [$x: d64, $places: i32[min: 0]] -> d64 { return my.round.places_d64($x, $places, Way.toward_zero); }
        [$x: d128] -> d128 { return my.round.whole_d128($x, Way.toward_zero); }
        [$x: d128, $places: i32[min: 0]] -> d128 { return my.round.places_d128($x, $places, Way.toward_zero); }
    }
}

# A bin's stored digits ($t: a `-` perhaps, digits, a `.` and more digits perhaps, no exponent) cut to
# $n places the way $way says.
fn digits [lent $t: str.utf8, $n: i32[min: 0], $way: Way] -> str.utf8 {
    let $whole: list[u8] = [];
    let $frac: list[u8] = [];
    let mut $neg := false;
    let mut $after_point := false;
    for $b in str.each.byte(lend $t) {
        if $b equal_to 45 { change $neg = true; }
        else if $b equal_to 46 { change $after_point = true; }
        else if $after_point { list.append(lend.mut $frac, value: $b); }
        else { list.append(lend.mut $whole, value: $b); }
    }
    let $places: u64 = to.u64($n);
    if list.length(lend $frac) at_most $places { return copy $t; }

    # What is kept, and what is cut off.
    let $kept: list[u8] = copy $whole;
    let $rest: list[u8] = [];
    for $i, $d in lend $frac {
        if $i at_most $places { list.append(lend.mut $kept, value: $d); }
        else { list.append(lend.mut $rest, value: $d); }
    }
    let mut $first : u8 = 48;
    let mut $rest_nonzero := false;
    let mut $after_first_nonzero := false;
    for $i, $d in lend $rest {
        if $i equal_to 1 { change $first = $d; }
        else if $d not_equal 48 { change $after_first_nonzero = true; }
        if $d not_equal 48 { change $rest_nonzero = true; }
    }
    let $last_odd := (list.length(lend $kept) at_least 1) and.then ($kept[pos: list.length(lend $kept)] mod.floor 2) equal_to 1;
    let $up := match $way {
        nearest_even { is $first > 53 or ($first equal_to 53 and ($after_first_nonzero or $last_odd)); }
        half_away { is $first at_least 53; }
        down { is $neg and $rest_nonzero; }
        up { is not $neg and $rest_nonzero; }
        toward_zero { is false; }
    };

    # Add one at the last kept digit, carrying: every 9 after the last digit that is not a 9 becomes 0, and
    # that digit goes up one; if every digit is a 9, a 1 comes in front.
    let mut $last_below_9: u64 = 0;
    for $i, $d in lend $kept {
        if $d < 57 { change $last_below_9 = $i; }
    }
    let mut $whole_len: u64 = list.length(lend $whole);
    let $digits: list[u8] = [];
    if $up and $last_below_9 equal_to 0 {
        list.append(lend.mut $digits, value: 49);
        change $whole_len = $whole_len + 1;
    }
    for $i, $d in lend $kept {
        if not $up or $i < $last_below_9 { list.append(lend.mut $digits, value: $d); }
        else if $i equal_to $last_below_9 {
            if $d < 57 { list.append(lend.mut $digits, value: $d + 1); }
            else { list.append(lend.mut $digits, value: 48); }
        }
        else { list.append(lend.mut $digits, value: 48); }
    }

    # Back to text: the sign only if something is left, then the point after the whole digits.
    let mut $nonzero := false;
    for $d in lend $digits { if $d not_equal 48 { change $nonzero = true; } }
    let $out: list[u8] = [];
    if $neg and $nonzero { list.append(lend.mut $out, value: 45); }
    for $i, $d in lend $digits {
        if $i - 1 equal_to $whole_len { list.append(lend.mut $out, value: 46); }
        list.append(lend.mut $out, value: $d);
    }
    let $text := to.str.utf8($out) otherwise { not_utf8 { std::print.useful.stderr("std: a bug: a rounded bin's digits did not read back", newln); std::exit(code: 101); } };
    return give $text;
}


# A d32 rounded to a whole number the way $way says. One at or past 10000000 has no places left.
fn whole_d32 [$x: d32, $way: Way] -> d32 {
    let $big: d32 = exactly 10000000;
    if $x at_least $big { return $x; }
    if $x at_most -$big { return $x; }
    let $one: d32 = exactly 1;
    let $two: d32 = exactly 2;
    let $none: d32 = exactly 0;
    let $half: d32 = exactly 0.5;
    let $t := $x div.trunc $one;
    let $r := $x - $t;
    let $odd := ($t mod.floor $two) not_equal $none;
    match $way {
        toward_zero { return $t; }
        down { return $x div.floor $one; }
        up { return $x div.ceil $one; }
        half_away {
            if $r at_least $half { return $t + $one; }
            if $r at_most -$half { return $t - $one; }
            return $t;
        }
        nearest_even {
            if $r > $half { return $t + $one; }
            if $r < -$half { return $t - $one; }
            if $r equal_to $half and $odd { return $t + $one; }
            if $r equal_to -$half and $odd { return $t - $one; }
            return $t;
        }
    }
}

# A d32 rounded to $n places the way $way says, keeping $n places as a deci does ("2.500") as far as
# its digits go. The point moves right until the number is whole or $n places are passed.
fn places_d32 [$x: d32, $n: i32[min: 0], $way: Way] -> d32 {
    let $one: d32 = exactly 1;
    let $ten: d32 = exactly 10;
    let $tenth: d32 = exactly 0.1;
    let mut $limit: i32 = $n;
    if $limit > 110 { change $limit = 110; }
    let mut $y := $x;
    let mut $k: i32 = 0;
    while $k < $limit and $y not_equal ($y div.trunc $one) {
        change $y = $y x $ten;
        change $k = $k + 1;
    }
    let mut $whole := $y div.trunc $one;
    if $y not_equal $whole { change $whole = my.round.whole_d32($y, copy $way); }
    let mut $scale := $one;
    for $i [from: 1, to: $k, by: 1] { change $scale = $scale x $ten; }
    let mut $zero: d32 = exactly 0;
    for $i [from: 1, to: $limit, by: 1] { change $zero = $zero x $tenth; }
    return ($whole div.near $scale) + $zero;
}

# A d64 rounded to a whole number the way $way says. One at or past 10000000000000000 has no places left.
fn whole_d64 [$x: d64, $way: Way] -> d64 {
    let $big: d64 = exactly 10000000000000000;
    if $x at_least $big { return $x; }
    if $x at_most -$big { return $x; }
    let $one: d64 = exactly 1;
    let $two: d64 = exactly 2;
    let $none: d64 = exactly 0;
    let $half: d64 = exactly 0.5;
    let $t := $x div.trunc $one;
    let $r := $x - $t;
    let $odd := ($t mod.floor $two) not_equal $none;
    match $way {
        toward_zero { return $t; }
        down { return $x div.floor $one; }
        up { return $x div.ceil $one; }
        half_away {
            if $r at_least $half { return $t + $one; }
            if $r at_most -$half { return $t - $one; }
            return $t;
        }
        nearest_even {
            if $r > $half { return $t + $one; }
            if $r < -$half { return $t - $one; }
            if $r equal_to $half and $odd { return $t + $one; }
            if $r equal_to -$half and $odd { return $t - $one; }
            return $t;
        }
    }
}

# A d64 rounded to $n places the way $way says, keeping $n places as a deci does ("2.500") as far as
# its digits go. The point moves right until the number is whole or $n places are passed.
fn places_d64 [$x: d64, $n: i32[min: 0], $way: Way] -> d64 {
    let $one: d64 = exactly 1;
    let $ten: d64 = exactly 10;
    let $tenth: d64 = exactly 0.1;
    let mut $limit: i32 = $n;
    if $limit > 400 { change $limit = 400; }
    let mut $y := $x;
    let mut $k: i32 = 0;
    while $k < $limit and $y not_equal ($y div.trunc $one) {
        change $y = $y x $ten;
        change $k = $k + 1;
    }
    let mut $whole := $y div.trunc $one;
    if $y not_equal $whole { change $whole = my.round.whole_d64($y, copy $way); }
    let mut $scale := $one;
    for $i [from: 1, to: $k, by: 1] { change $scale = $scale x $ten; }
    let mut $zero: d64 = exactly 0;
    for $i [from: 1, to: $limit, by: 1] { change $zero = $zero x $tenth; }
    return ($whole div.near $scale) + $zero;
}

# A d128 rounded to a whole number the way $way says. One at or past 10000000000000000000000000000000000 has no places left.
fn whole_d128 [$x: d128, $way: Way] -> d128 {
    let $big: d128 = exactly 10000000000000000000000000000000000;
    if $x at_least $big { return $x; }
    if $x at_most -$big { return $x; }
    let $one: d128 = exactly 1;
    let $two: d128 = exactly 2;
    let $none: d128 = exactly 0;
    let $half: d128 = exactly 0.5;
    let $t := $x div.trunc $one;
    let $r := $x - $t;
    let $odd := ($t mod.floor $two) not_equal $none;
    match $way {
        toward_zero { return $t; }
        down { return $x div.floor $one; }
        up { return $x div.ceil $one; }
        half_away {
            if $r at_least $half { return $t + $one; }
            if $r at_most -$half { return $t - $one; }
            return $t;
        }
        nearest_even {
            if $r > $half { return $t + $one; }
            if $r < -$half { return $t - $one; }
            if $r equal_to $half and $odd { return $t + $one; }
            if $r equal_to -$half and $odd { return $t - $one; }
            return $t;
        }
    }
}

# A d128 rounded to $n places the way $way says, keeping $n places as a deci does ("2.500") as far as
# its digits go. The point moves right until the number is whole or $n places are passed.
fn places_d128 [$x: d128, $n: i32[min: 0], $way: Way] -> d128 {
    let $one: d128 = exactly 1;
    let $ten: d128 = exactly 10;
    let $tenth: d128 = exactly 0.1;
    let mut $limit: i32 = $n;
    if $limit > 6200 { change $limit = 6200; }
    let mut $y := $x;
    let mut $k: i32 = 0;
    while $k < $limit and $y not_equal ($y div.trunc $one) {
        change $y = $y x $ten;
        change $k = $k + 1;
    }
    let mut $whole := $y div.trunc $one;
    if $y not_equal $whole { change $whole = my.round.whole_d128($y, copy $way); }
    let mut $scale := $one;
    for $i [from: 1, to: $k, by: 1] { change $scale = $scale x $ten; }
    let mut $zero: d128 = exactly 0;
    for $i [from: 1, to: $limit, by: 1] { change $zero = $zero x $tenth; }
    return ($whole div.near $scale) + $zero;
}
