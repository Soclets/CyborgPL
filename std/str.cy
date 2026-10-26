# Text: what can be done with a `str.utf8` beyond its units (design/strings.md).
#
# These work on the bytes. A part found by its bytes inside UTF-8 text starts and ends where
# characters do, because no character's bytes begin in the middle of another's, so whatever is
# cut out between two finds is UTF-8 text again.

# Whether $part appears somewhere in $s.
pub fn contains [lent $s: str.utf8, lent $part: str.utf8[min: 1 bytes]] -> bool {
    let $text := bytes_of(lend $s);
    let $want := bytes_of(lend $part);
    let $n := list.length(lend $text);
    let $m := list.length(lend $want);
    if $m > $n { return false; }
    for $pos: u64 [from: 1, to: ($n - $m + 1), by: 1] {
        if sits_at(lend $text, lend $want, $pos) { return true; }
    }
    return false;
}

# Whether $s begins with $part.
pub fn starts_with [lent $s: str.utf8, lent $part: str.utf8[min: 1 bytes]] -> bool {
    let $text := bytes_of(lend $s);
    let $want := bytes_of(lend $part);
    return sits_at(lend $text, lend $want, 1);
}

# Whether $s ends with $part.
pub fn ends_with [lent $s: str.utf8, lent $part: str.utf8[min: 1 bytes]] -> bool {
    let $text := bytes_of(lend $s);
    let $want := bytes_of(lend $part);
    let $n := list.length(lend $text);
    let $m := list.length(lend $want);
    if $m > $n { return false; }
    return sits_at(lend $text, lend $want, $n - $m + 1);
}

# The pieces of $s between each $at, left to right: "a,,b" at "," is "a", `str.empty`, "b".
pub fn split [lent $s: str.utf8, lent $at: str.utf8[min: 1 bytes]] -> list[str.utf8] {
    let $text := bytes_of(lend $s);
    let $sep := bytes_of(lend $at);
    let $n := list.length(lend $text);
    let $m := list.length(lend $sep);
    let $pieces: list[str.utf8] = [];
    let $piece: list[u8] = [];
    let $pos: u64[min: 1] = 1;
    while $pos at_most $n {
        if sits_at(lend $text, lend $sep, $pos) {
            list.append(lend.mut $pieces, value: text_of(give $piece));
            change $piece = list.empty;
            change $pos = $pos + $m;
        } else {
            list.append(lend.mut $piece, value: $text[pos: $pos]);
            change $pos = $pos + 1;
        }
    }
    list.append(lend.mut $pieces, value: text_of(give $piece));
    return give $pieces;
}

# The texts in $parts, one after another, with $between between each two.
pub fn join [lent $parts: list[str.utf8], lent $between: str.utf8] -> str.utf8 {
    let $out: list[u8] = [];
    for $i, $part in lend $parts {
        if $i > 1 {
            for $b in str.each.byte(lend $between) { list.append(lend.mut $out, value: $b); }
        }
        for $b in str.each.byte(lend $part) { list.append(lend.mut $out, value: $b); }
    }
    return text_of(give $out);
}

# $s with every $from made $to, left to right, never looking inside what it put in:
# "aaa" from "aa" to "b" is "ba".
pub fn replace [lent $s: str.utf8, lent $from: str.utf8[min: 1 bytes], lent $to: str.utf8] -> str.utf8 {
    let $text := bytes_of(lend $s);
    let $old := bytes_of(lend $from);
    let $n := list.length(lend $text);
    let $m := list.length(lend $old);
    let $out: list[u8] = [];
    let $pos: u64[min: 1] = 1;
    while $pos at_most $n {
        if sits_at(lend $text, lend $old, $pos) {
            for $b in str.each.byte(lend $to) { list.append(lend.mut $out, value: $b); }
            change $pos = $pos + $m;
        } else {
            list.append(lend.mut $out, value: $text[pos: $pos]);
            change $pos = $pos + 1;
        }
    }
    return text_of(give $out);
}

# One unit of $s by its position, counting from 1: `std::str.at.byte(lend $s, pos: 2)`, `at.point`,
# `at.grapheme`.
# A position past the last unit fails out_of_range.
pub coll.fn at {
    # The byte at $pos.
    at.byte [lent $s: str.utf8, $pos: u64[min: 1]] -> u8 or fail[out_of_range] {
        for $i, $b in str.each.byte(lend $s) {
            if $i equal_to $pos { return $b; }
        }
        fail out_of_range;
    }

    # The character (Unicode code point) at $pos.
    at.point [lent $s: str.utf8, $pos: u64[min: 1]] -> char.%point or fail[out_of_range] {
        for $i, $p in str.each.point(lend $s) {
            if $i equal_to $pos { return $p; }
        }
        fail out_of_range;
    }

    # The grapheme (what a person counts as one character) at $pos.
    at.grapheme [lent $s: str.utf8, $pos: u64[min: 1]] -> char.%grapheme or fail[out_of_range] {
        for $i, $g in str.each.grapheme(lend $s) {
            if $i equal_to $pos { return copy $g; }
        }
        fail out_of_range;
    }
}

# The part of $s from one position to another, both kept: `std::str.slice.grapheme(lend $s, from: pos 2,
# to: pos 4)`, or `before: pos 5` to leave the end out. It fails out_of_range when the end comes before the
# start or past the last unit, so a slice is never empty.
pub coll.fn slice {
    # Bytes $from to $to, as bytes: an end may land inside a character, so making text of them is the
    # caller's `to.str.utf8(…) otherwise …` (owner, 2026-10-20: "(a)").
    slice.byte [lent $s: str.utf8, $from: u64[min: 1], $to: u64[min: 1]] -> list[u8] or fail[out_of_range] {
        if $to < $from or $to > str.bytes(lend $s) { fail out_of_range; }
        let $out: list[u8] = [];
        for $i, $b in str.each.byte(lend $s) {
            if $i > $to { break; }
            if $i at_least $from { list.append(lend.mut $out, value: $b); }
        }
        return give $out;
    }

    # Code points $from to $to.
    slice.point [lent $s: str.utf8, $from: u64[min: 1], $to: u64[min: 1]] -> str.utf8 or fail[out_of_range] {
        if $to < $from or $to > str.points(lend $s) { fail out_of_range; }
        let $out: list[u8] = [];
        for $i, $p in str.each.point(lend $s) {
            if $i > $to { break; }
            if $i at_least $from {
                let $t := to.str.utf8($p);
                for $b in str.each.byte(lend $t) { list.append(lend.mut $out, value: $b); }
            }
        }
        return my.str.text_of(give $out);
    }

    # Graphemes $from to $to.
    slice.grapheme [lent $s: str.utf8, $from: u64[min: 1], $to: u64[min: 1]] -> str.utf8 or fail[out_of_range] {
        if $to < $from or $to > str.graphemes(lend $s) { fail out_of_range; }
        let $out: list[u8] = [];
        for $i, $g in str.each.grapheme(lend $s) {
            if $i > $to { break; }
            if $i at_least $from {
                let $t := to.str.utf8(copy $g);
                for $b in str.each.byte(lend $t) { list.append(lend.mut $out, value: $b); }
            }
        }
        return my.str.text_of(give $out);
    }
}

# A case change says its mapping (owner, 2026-10-20): `.ascii` changes a to z only, so `café` is `CAFé`;
# `.simple` and `.full` are Unicode's. A bare `str.upper` is refused.
pub coll.fn upper {
    # $s with a to z made A to Z, every other byte as it is.
    upper.ascii [lent $s: str.utf8] -> str.utf8 {
        let $out: list[u8] = [];
        for $b in str.each.byte(lend $s) {
            if $b at_least 97 and.then $b at_most 122 { list.append(lend.mut $out, value: $b - 32); }
            else { list.append(lend.mut $out, value: $b); }
        }
        return my.str.text_of(give $out);
    }

    # $s with each character made its Unicode simple uppercase, one for one: `ß` stays `ß`.
    upper.simple [lent $s: str.utf8] -> str.utf8 {
        let $out: list[u8] = [];
        for $p in str.each.point(lend $s) {
            my.str.append_point(lend.mut $out, my.case_tables.simple_upper(char.code($p)));
        }
        return my.str.text_of(give $out);
    }

    versions upper.full {
        # $s with each character made its Unicode full uppercase, by the rules every language shares
        # (`casing.root`): `ß` is `SS`, `ŉ` is `ʼN`.
        [lent $s: str.utf8] -> str.utf8 {
            let $out: list[u8] = [];
            for $p in str.each.point(lend $s) { my.str.append_full_upper(lend.mut $out, char.code($p)); }
            return my.str.text_of(give $out);
        }
        # The same, with $lang's own rules from SpecialCasing: in Turkish and Azerbaijani `i` is `İ`; in
        # Lithuanian a dot above after a soft-dotted letter (`i̇`) goes, the letter's own capital having none.
        [lent $s: str.utf8, $lang: my.casing] -> str.utf8 {
            let $c := my.str.codes_of(lend $s);
            let $out: list[u8] = [];
            for $i: u64 [from: 1, to: list.length(lend $c), by: 1] {
                let $x := $c[pos: $i];
                if ($lang equal_to my.casing.tr or $lang equal_to my.casing.az) and $x equal_to hex 69 {
                    my.str.append_point(lend.mut $out, hex 130);
                } else if $lang equal_to my.casing.lt and $x equal_to hex 307 and my.str.after_soft_dotted(lend $c, $i) {
                } else {
                    my.str.append_full_upper(lend.mut $out, $x);
                }
            }
            return my.str.text_of(give $out);
        }
    }
}

pub coll.fn lower {
    # $s with A to Z made a to z, every other byte as it is.
    lower.ascii [lent $s: str.utf8] -> str.utf8 {
        let $out: list[u8] = [];
        for $b in str.each.byte(lend $s) {
            if $b at_least 65 and.then $b at_most 90 { list.append(lend.mut $out, value: $b + 32); }
            else { list.append(lend.mut $out, value: $b); }
        }
        return my.str.text_of(give $out);
    }

    # $s with each character made its Unicode simple lowercase, one for one.
    lower.simple [lent $s: str.utf8] -> str.utf8 {
        let $out: list[u8] = [];
        for $p in str.each.point(lend $s) {
            my.str.append_point(lend.mut $out, my.case_tables.simple_lower(char.code($p)));
        }
        return my.str.text_of(give $out);
    }

    versions lower.full {
        # $s with each character made its Unicode full lowercase, by the rules every language shares
        # (`casing.root`): `İ` is `i̇`, and a `Σ` ending a word is `ς`.
        [lent $s: str.utf8] -> str.utf8 {
            let $c := my.str.codes_of(lend $s);
            let $out: list[u8] = [];
            for $i: u64 [from: 1, to: list.length(lend $c), by: 1] {
                my.str.append_full_lower(lend.mut $out, lend $c, $i);
            }
            return my.str.text_of(give $out);
        }
        # The same, with $lang's own rules from SpecialCasing: in Turkish and Azerbaijani `I` is `ı` and
        # `İ` is `i`; in Lithuanian `I`, `J` and `Į` keep a dot (`i̇`) when an accent above follows, and
        # `Ì`, `Í`, `Ĩ` are written with one.
        [lent $s: str.utf8, $lang: my.casing] -> str.utf8 {
            let $c := my.str.codes_of(lend $s);
            let $out: list[u8] = [];
            for $i: u64 [from: 1, to: list.length(lend $c), by: 1] {
                let $x := $c[pos: $i];
                if $lang equal_to my.casing.tr or $lang equal_to my.casing.az {
                    if $x equal_to hex 130 { my.str.append_point(lend.mut $out, hex 69); }
                    else if $x equal_to hex 307 and my.str.after_capital_i(lend $c, $i) { }
                    else if $x equal_to hex 49 and not my.str.before_dot(lend $c, $i) { my.str.append_point(lend.mut $out, hex 131); }
                    else { my.str.append_full_lower(lend.mut $out, lend $c, $i); }
                } else if $lang equal_to my.casing.lt {
                    if ($x equal_to hex 49 or $x equal_to hex 4A or $x equal_to hex 12E) and my.str.more_above(lend $c, $i) {
                        my.str.append_point(lend.mut $out, my.case_tables.simple_lower($x));
                        my.str.append_point(lend.mut $out, hex 307);
                    } else if $x equal_to hex CC or $x equal_to hex CD or $x equal_to hex 128 {
                        my.str.append_point(lend.mut $out, hex 69);
                        my.str.append_point(lend.mut $out, hex 307);
                        if $x equal_to hex CC { my.str.append_point(lend.mut $out, hex 300); }
                        else if $x equal_to hex CD { my.str.append_point(lend.mut $out, hex 301); }
                        else { my.str.append_point(lend.mut $out, hex 303); }
                    } else {
                        my.str.append_full_lower(lend.mut $out, lend $c, $i);
                    }
                } else {
                    my.str.append_full_lower(lend.mut $out, lend $c, $i);
                }
            }
            return my.str.text_of(give $out);
        }
    }
}

# $x's full uppercase, after $out: SpecialCasing's longer mapping when it has one, else its simple one.
fn append_full_upper [lent.mut $out: list[u8], $x: u32] {
    if my.case_tables.full_upper_has($x) {
        my.str.append_point(lend.mut $out, my.case_tables.full_upper_1_of($x));
        let $second := my.case_tables.full_upper_2_of($x);
        if $second not_equal 0 { my.str.append_point(lend.mut $out, $second); }
        let $third := my.case_tables.full_upper_3_of($x);
        if $third not_equal 0 { my.str.append_point(lend.mut $out, $third); }
    } else {
        my.str.append_point(lend.mut $out, my.case_tables.simple_upper($x));
    }
}

# The full lowercase of the character at $i in $c, after $out, by the shared rules: a `Σ` is `ς` when it ends
# a word, else SpecialCasing's longer mapping when there is one, else the simple one.
fn append_full_lower [lent.mut $out: list[u8], lent $c: list[u32], $i: u64[min: 1]] {
    if $i > list.length(lend $c) { return; }
    let $x := $c[pos: $i];
    if $x equal_to hex 3A3 and my.str.cased_before(lend $c, $i) and not my.str.cased_after(lend $c, $i) {
        my.str.append_point(lend.mut $out, hex 3C2);
    } else if my.case_tables.full_lower_has($x) {
        my.str.append_point(lend.mut $out, my.case_tables.full_lower_1_of($x));
        let $second := my.case_tables.full_lower_2_of($x);
        if $second not_equal 0 { my.str.append_point(lend.mut $out, $second); }
        let $third := my.case_tables.full_lower_3_of($x);
        if $third not_equal 0 { my.str.append_point(lend.mut $out, $third); }
    } else {
        my.str.append_point(lend.mut $out, my.case_tables.simple_lower($x));
    }
}

# The code points of $s, in order.
fn codes_of [lent $s: str.utf8] -> list[u32] {
    let $out: list[u32] = [];
    for $p in str.each.point(lend $s) { list.append(lend.mut $out, value: char.code($p)); }
    return give $out;
}

# The context of a case change, Unicode's (The Unicode Standard, 3.13, Table 3-17), each looking from the
# character at $i in $c. A mark run is the combining marks after a letter: those of any class but 0.

# Final_Sigma, before: a cased letter comes before $i, only case-ignorable ones between.
fn cased_before [lent $c: list[u32], $i: u64[min: 1]] -> bool {
    if $i > list.length(lend $c) { return false; }
    for $k: u64 [from: $i - 1, to: 1, by: -1] {
        if my.case_tables.cased($c[pos: $k]) { return true; }
        if not my.case_tables.case_ignorable($c[pos: $k]) { return false; }
    }
    return false;
}

# Final_Sigma, after: a cased letter comes after $i, only case-ignorable ones between.
fn cased_after [lent $c: list[u32], $i: u64[min: 1]] -> bool {
    if $i at_least list.length(lend $c) { return false; }
    for $k: u64 [from: $i + 1, to: list.length(lend $c), by: 1] {
        if my.case_tables.cased($c[pos: $k]) { return true; }
        if not my.case_tables.case_ignorable($c[pos: $k]) { return false; }
    }
    return false;
}

# After_Soft_Dotted: a soft-dotted letter (`i`, `j`, `į`, …) comes before $i in the same mark run, no mark
# above between.
fn after_soft_dotted [lent $c: list[u32], $i: u64[min: 1]] -> bool {
    if $i > list.length(lend $c) { return false; }
    for $k: u64 [from: $i - 1, to: 1, by: -1] {
        let $x := $c[pos: $k];
        if my.case_tables.soft_dotted($x) { return true; }
        if my.case_tables.above($x) or not my.case_tables.marks($x) { return false; }
    }
    return false;
}

# After_I: a capital `I` comes before $i in the same mark run, no mark above between.
fn after_capital_i [lent $c: list[u32], $i: u64[min: 1]] -> bool {
    if $i > list.length(lend $c) { return false; }
    for $k: u64 [from: $i - 1, to: 1, by: -1] {
        let $x := $c[pos: $k];
        if $x equal_to hex 49 { return true; }
        if my.case_tables.above($x) or not my.case_tables.marks($x) { return false; }
    }
    return false;
}

# More_Above: a mark above comes after $i in the same mark run.
fn more_above [lent $c: list[u32], $i: u64[min: 1]] -> bool {
    if $i at_least list.length(lend $c) { return false; }
    for $k: u64 [from: $i + 1, to: list.length(lend $c), by: 1] {
        let $x := $c[pos: $k];
        if my.case_tables.above($x) { return true; }
        if not my.case_tables.marks($x) { return false; }
    }
    return false;
}

# Before_Dot: a combining dot above (U+0307) comes after $i in the same mark run, no other mark above between.
fn before_dot [lent $c: list[u32], $i: u64[min: 1]] -> bool {
    if $i at_least list.length(lend $c) { return false; }
    for $k: u64 [from: $i + 1, to: list.length(lend $c), by: 1] {
        let $x := $c[pos: $k];
        if $x equal_to hex 307 { return true; }
        if my.case_tables.above($x) or not my.case_tables.marks($x) { return false; }
    }
    return false;
}

# The UTF-8 bytes of the character $c, after $out. Every $c here comes from Unicode's case tables, so it
# is always a character; if it ever is not, std has a bug, and the program stops saying so.
fn append_point [lent.mut $out: list[u8], $c: u32] {
    let $ch := char.%point.from_code($c) otherwise {
        not_a_char {
            std::print.useful.stderr("std: a bug: a case table gave a code that is no character", newln);
            std::exit(code: 101);
        }
    };
    let $t := to.str.utf8($ch);
    for $b in str.each.byte(lend $t) { list.append(lend.mut $out, value: $b); }
}

# The bytes of $s, in order.
fn bytes_of [lent $s: str.utf8] -> list[u8] {
    let $out: list[u8] = [];
    for $b in str.each.byte(lend $s) { list.append(lend.mut $out, value: $b); }
    return give $out;
}

# Whether $part sits in $text with its first byte at $pos.
fn sits_at [lent $text: list[u8], lent $part: list[u8], $pos: u64[min: 1]] -> bool {
    let $n := list.length(lend $text);
    let $m := list.length(lend $part);
    if $m > $n { return false; }
    if $pos > $n - $m + 1 { return false; }
    for $k: u64 [from: $pos, to: ($pos + $m - 1), by: 1] {
        if $text[pos: $k] not_equal $part[pos: $k - $pos + 1] { return false; }
    }
    return true;
}

# Bytes cut from UTF-8 text where characters start and end, back into text. They are always
# UTF-8; if they ever are not, std has a bug, and the program stops saying so.
fn text_of [$bytes: list[u8]] -> str.utf8 {
    let $t := to.str.utf8($bytes) otherwise {
        not_utf8 {
            std::print.useful.stderr("std: a bug: text cut on a character's edge was not UTF-8", newln);
            std::exit(code: 101);
        }
    };
    return give $t;
}
