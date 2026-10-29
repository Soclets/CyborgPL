# Text: what can be done with a `str.utf8` beyond its units (design/strings.md).
#
# These work on the bytes. A part found by its bytes inside UTF-8 text starts and ends where
# characters do, because no character's bytes begin in the middle of another's, so whatever is
# cut out between two finds is UTF-8 text again.

# Whether $part appears somewhere in $s.
pub fn contains [lent $s: str.utf8, lent $part: str.utf8[min: 1 bytes]] -> bool:
    let $text := bytes_of(lend $s);
    let $want := bytes_of(lend $part);
    let $n := ls.len(lend $text);
    let $m := ls.len(lend $want);
    if $m > $n:
        ret false;
    ;if
    for $pos: u64 [from: 1, to: ($n - $m + 1), by: 1]:
        if sits_at(lend $text, lend $want, $pos):
            ret true;
        ;if
    ;for
    ret false;
;fn contains

# Whether $s begins with $part.
pub fn starts_with [lent $s: str.utf8, lent $part: str.utf8[min: 1 bytes]] -> bool:
    let $text := bytes_of(lend $s);
    let $want := bytes_of(lend $part);
    ret sits_at(lend $text, lend $want, 1);
;fn starts_with

# Whether $s ends with $part.
pub fn ends_with [lent $s: str.utf8, lent $part: str.utf8[min: 1 bytes]] -> bool:
    let $text := bytes_of(lend $s);
    let $want := bytes_of(lend $part);
    let $n := ls.len(lend $text);
    let $m := ls.len(lend $want);
    if $m > $n:
        ret false;
    ;if
    ret sits_at(lend $text, lend $want, $n - $m + 1);
;fn ends_with

# The pieces of $s between each $at, left to right: "a,,b" at "," is "a", `str.empty`, "b".
pub fn split [lent $s: str.utf8, lent $at: str.utf8[min: 1 bytes]] -> ls[str.utf8]:
    let $text := bytes_of(lend $s);
    let $sep := bytes_of(lend $at);
    let $n := ls.len(lend $text);
    let $m := ls.len(lend $sep);
    var $pieces: ls[str.utf8] = [];
    var $piece: ls[u8] = [];
    var $pos: u64[min: 1] = 1;
    while $pos <= $n:
        if sits_at(lend $text, lend $sep, $pos):
            ls.append(lend.mut $pieces, value: text_of(give $piece));
            change $piece to ls.empty;
            change $pos to $pos + $m;
        else:
            ls.append(lend.mut $piece, value: $text[pos: $pos]);
            change $pos to $pos + 1;
        ;if
    ;while
    ls.append(lend.mut $pieces, value: text_of(give $piece));
    ret give $pieces;
;fn split

# The texts in $parts, one after another, with $between between each two.
pub fn join [lent $parts: ls[str.utf8], lent $between: str.utf8] -> str.utf8:
    var $out: ls[u8] = [];
    for $i, $part in lend $parts:
        if $i > 1:
            for $b in str.each.byte(lend $between):
                ls.append(lend.mut $out, value: $b);
            ;for
        ;if
        for $b in str.each.byte(lend $part):
            ls.append(lend.mut $out, value: $b);
        ;for
    ;for
    ret text_of(give $out);
;fn join

# $s with every $from made $to, left to right, never looking inside what it put in:
# "aaa" from "aa" to "b" is "ba".
pub fn replace [lent $s: str.utf8, lent $from: str.utf8[min: 1 bytes], lent $to: str.utf8] -> str.utf8:
    let $text := bytes_of(lend $s);
    let $old := bytes_of(lend $from);
    let $n := ls.len(lend $text);
    let $m := ls.len(lend $old);
    var $out: ls[u8] = [];
    var $pos: u64[min: 1] = 1;
    while $pos <= $n:
        if sits_at(lend $text, lend $old, $pos):
            for $b in str.each.byte(lend $to):
                ls.append(lend.mut $out, value: $b);
            ;for
            change $pos to $pos + $m;
        else:
            ls.append(lend.mut $out, value: $text[pos: $pos]);
            change $pos to $pos + 1;
        ;if
    ;while
    ret text_of(give $out);
;fn replace

# One unit of $s by its position, counting from 1: `std::str.at.byte(lend $s, pos: 2)`, `at.point`,
# `at.grapheme`.
# A position past the last unit fails out_of_range.
pub coll.fn at:
    # The byte at $pos.
    member at.byte [lent $s: str.utf8, let $pos: u64[min: 1]] -> u8 or fail[out_of_range]:
        for $i, $b in str.each.byte(lend $s):
            if $i == $pos:
                ret $b;
            ;if
        ;for
        fail out_of_range;
    ;member at.byte

    # The character (Unicode code point) at $pos.
    member at.point [lent $s: str.utf8, let $pos: u64[min: 1]] -> char.%point or fail[out_of_range]:
        for $i, $p in str.each.point(lend $s):
            if $i == $pos:
                ret $p;
            ;if
        ;for
        fail out_of_range;
    ;member at.point

    # The grapheme (what a person counts as one character) at $pos.
    member at.cluster [lent $s: str.utf8, let $pos: u64[min: 1]] -> char.%cluster or fail[out_of_range]:
        for $i, $g in str.each.cluster(lend $s):
            if $i == $pos:
                ret copy $g;
            ;if
        ;for
        fail out_of_range;
    ;member at.cluster
;coll.fn at

# The part of $s from one position to another, both kept: `std::str.slice.grapheme(lend $s, from: pos 2,
# to: pos 4)`, or `before: pos 5` to leave the end out. It fails out_of_range when the end comes before the
# start or past the last unit, so a slice is never empty.
pub coll.fn slice:
    # Bytes $from to $to, as bytes: an end may land inside a character, so making text of them is the
    # caller's `to.str.utf8(…) otherwise …` (owner, 2026-10-20: "(a)").
    member slice.byte [lent $s: str.utf8, let $from: u64[min: 1], let $to: u64[min: 1]] -> ls[u8] or fail[out_of_range]:
        if $to < $from or $to > str.bytes(lend $s):
            fail out_of_range;
        ;if
        var $out: ls[u8] = [];
        for $i, $b in str.each.byte(lend $s):
            if $i > $to:
                break;
            ;if
            if $i >= $from:
                ls.append(lend.mut $out, value: $b);
            ;if
        ;for
        ret give $out;
    ;member slice.byte

    # Code points $from to $to.
    member slice.point [lent $s: str.utf8, let $from: u64[min: 1], let $to: u64[min: 1]] -> str.utf8 or fail[out_of_range]:
        if $to < $from or $to > str.points(lend $s):
            fail out_of_range;
        ;if
        var $out: ls[u8] = [];
        for $i, $p in str.each.point(lend $s):
            if $i > $to:
                break;
            ;if
            if $i >= $from:
                let $t := to.str.utf8($p);
                for $b in str.each.byte(lend $t):
                    ls.append(lend.mut $out, value: $b);
                ;for
            ;if
        ;for
        ret my.str.text_of(give $out);
    ;member slice.point

    # Graphemes $from to $to.
    member slice.cluster [lent $s: str.utf8, let $from: u64[min: 1], let $to: u64[min: 1]] -> str.utf8 or fail[out_of_range]:
        if $to < $from or $to > str.clusters(lend $s):
            fail out_of_range;
        ;if
        var $out: ls[u8] = [];
        for $i, $g in str.each.cluster(lend $s):
            if $i > $to:
                break;
            ;if
            if $i >= $from:
                let $t := to.str.utf8(copy $g);
                for $b in str.each.byte(lend $t):
                    ls.append(lend.mut $out, value: $b);
                ;for
            ;if
        ;for
        ret my.str.text_of(give $out);
    ;member slice.cluster
;coll.fn slice

# A case change says its mapping (owner, 2026-10-20): `.ascii` changes a to z only, so `café` is `CAFé`;
# `.simple` and `.full` are Unicode's. A bare `str.upper` is refused.
pub coll.fn upper:
    # $s with a to z made A to Z, every other byte as it is.
    member upper.ascii [lent $s: str.utf8] -> str.utf8:
        var $out: ls[u8] = [];
        for $b in str.each.byte(lend $s):
            if $b >= 97 and.then $b <= 122:
                ls.append(lend.mut $out, value: $b - 32);
            else:
                ls.append(lend.mut $out, value: $b);
            ;if
        ;for
        ret my.str.text_of(give $out);
    ;member upper.ascii

    # $s with each character made its Unicode simple uppercase, one for one: `ß` stays `ß`.
    member upper.simple [lent $s: str.utf8] -> str.utf8:
        var $out: ls[u8] = [];
        for $p in str.each.point(lend $s):
            my.str.append_point(lend.mut $out, my.case_tables.simple_upper(char.code($p)));
        ;for
        ret my.str.text_of(give $out);
    ;member upper.simple

    versions upper.full:
        # $s with each character made its Unicode full uppercase, by the rules every language shares
        # (`casing.root`): `ß` is `SS`, `ŉ` is `ʼN`.
        [lent $s: str.utf8] -> str.utf8:
            var $out: ls[u8] = [];
            for $p in str.each.point(lend $s):
                my.str.append_full_upper(lend.mut $out, char.code($p));
            ;for
            ret my.str.text_of(give $out);
        ;version
        # The same, with $lang's own rules from SpecialCasing: in Turkish and Azerbaijani `i` is `İ`; in
        # Lithuanian a dot above after a soft-dotted letter (`i̇`) goes, the letter's own capital having none.
        [lent $s: str.utf8, let $lang: my.casing] -> str.utf8:
            let $c := my.str.codes_of(lend $s);
            var $out: ls[u8] = [];
            for $i: u64 [from: 1, to: ls.len(lend $c), by: 1]:
                let $x := $c[pos: $i];
                if ($lang == my.casing.tr or $lang == my.casing.az) and $x == hex 69:
                    my.str.append_point(lend.mut $out, hex 130);
                else if $lang == my.casing.lt and $x == hex 307 and my.str.after_soft_dotted(lend $c, $i):
                else:
                    my.str.append_full_upper(lend.mut $out, $x);
                ;if
            ;for
            ret my.str.text_of(give $out);
        ;version
    ;versions upper.full
;coll.fn upper

pub coll.fn lower:
    # $s with A to Z made a to z, every other byte as it is.
    member lower.ascii [lent $s: str.utf8] -> str.utf8:
        var $out: ls[u8] = [];
        for $b in str.each.byte(lend $s):
            if $b >= 65 and.then $b <= 90:
                ls.append(lend.mut $out, value: $b + 32);
            else:
                ls.append(lend.mut $out, value: $b);
            ;if
        ;for
        ret my.str.text_of(give $out);
    ;member lower.ascii

    # $s with each character made its Unicode simple lowercase, one for one.
    member lower.simple [lent $s: str.utf8] -> str.utf8:
        var $out: ls[u8] = [];
        for $p in str.each.point(lend $s):
            my.str.append_point(lend.mut $out, my.case_tables.simple_lower(char.code($p)));
        ;for
        ret my.str.text_of(give $out);
    ;member lower.simple

    versions lower.full:
        # $s with each character made its Unicode full lowercase, by the rules every language shares
        # (`casing.root`): `İ` is `i̇`, and a `Σ` ending a word is `ς`.
        [lent $s: str.utf8] -> str.utf8:
            let $c := my.str.codes_of(lend $s);
            var $out: ls[u8] = [];
            for $i: u64 [from: 1, to: ls.len(lend $c), by: 1]:
                my.str.append_full_lower(lend.mut $out, lend $c, $i);
            ;for
            ret my.str.text_of(give $out);
        ;version
        # The same, with $lang's own rules from SpecialCasing: in Turkish and Azerbaijani `I` is `ı` and
        # `İ` is `i`; in Lithuanian `I`, `J` and `Į` keep a dot (`i̇`) when an accent above follows, and
        # `Ì`, `Í`, `Ĩ` are written with one.
        [lent $s: str.utf8, let $lang: my.casing] -> str.utf8:
            let $c := my.str.codes_of(lend $s);
            var $out: ls[u8] = [];
            for $i: u64 [from: 1, to: ls.len(lend $c), by: 1]:
                let $x := $c[pos: $i];
                if $lang == my.casing.tr or $lang == my.casing.az:
                    if $x == hex 130:
                        my.str.append_point(lend.mut $out, hex 69);
                    else if $x == hex 307 and my.str.after_capital_i(lend $c, $i):
                    else if $x == hex 49 and not my.str.before_dot(lend $c, $i):
                        my.str.append_point(lend.mut $out, hex 131);
                    else:
                        my.str.append_full_lower(lend.mut $out, lend $c, $i);
                    ;if
                else if $lang == my.casing.lt:
                    if ($x == hex 49 or $x == hex 4A or $x == hex 12E) and my.str.more_above(lend $c, $i):
                        my.str.append_point(lend.mut $out, my.case_tables.simple_lower($x));
                        my.str.append_point(lend.mut $out, hex 307);
                    else if $x == hex CC or $x == hex CD or $x == hex 128:
                        my.str.append_point(lend.mut $out, hex 69);
                        my.str.append_point(lend.mut $out, hex 307);
                        if $x == hex CC:
                            my.str.append_point(lend.mut $out, hex 300);
                        else if $x == hex CD:
                            my.str.append_point(lend.mut $out, hex 301);
                        else:
                            my.str.append_point(lend.mut $out, hex 303);
                        ;if
                    else:
                        my.str.append_full_lower(lend.mut $out, lend $c, $i);
                    ;if
                else:
                    my.str.append_full_lower(lend.mut $out, lend $c, $i);
                ;if
            ;for
            ret my.str.text_of(give $out);
        ;version
    ;versions lower.full
;coll.fn lower

# $x's full uppercase, after $out: SpecialCasing's longer mapping when it has one, else its simple one.
fn append_full_upper [lent.mut $out: ls[u8], let $x: u32]:
    if my.case_tables.full_upper_has($x):
        my.str.append_point(lend.mut $out, my.case_tables.full_upper_1_of($x));
        let $second := my.case_tables.full_upper_2_of($x);
        if $second != 0:
            my.str.append_point(lend.mut $out, $second);
        ;if
        let $third := my.case_tables.full_upper_3_of($x);
        if $third != 0:
            my.str.append_point(lend.mut $out, $third);
        ;if
    else:
        my.str.append_point(lend.mut $out, my.case_tables.simple_upper($x));
    ;if
;fn append_full_upper

# The full lowercase of the character at $i in $c, after $out, by the shared rules: a `Σ` is `ς` when it ends
# a word, else SpecialCasing's longer mapping when there is one, else the simple one.
fn append_full_lower [lent.mut $out: ls[u8], lent $c: ls[u32], let $i: u64[min: 1]]:
    if $i > ls.len(lend $c):
        ret;
    ;if
    let $x := $c[pos: $i];
    if $x == hex 3A3 and my.str.cased_before(lend $c, $i) and not my.str.cased_after(lend $c, $i):
        my.str.append_point(lend.mut $out, hex 3C2);
    else if my.case_tables.full_lower_has($x):
        my.str.append_point(lend.mut $out, my.case_tables.full_lower_1_of($x));
        let $second := my.case_tables.full_lower_2_of($x);
        if $second != 0:
            my.str.append_point(lend.mut $out, $second);
        ;if
        let $third := my.case_tables.full_lower_3_of($x);
        if $third != 0:
            my.str.append_point(lend.mut $out, $third);
        ;if
    else:
        my.str.append_point(lend.mut $out, my.case_tables.simple_lower($x));
    ;if
;fn append_full_lower

# The code points of $s, in order.
fn codes_of [lent $s: str.utf8] -> ls[u32]:
    var $out: ls[u32] = [];
    for $p in str.each.point(lend $s):
        ls.append(lend.mut $out, value: char.code($p));
    ;for
    ret give $out;
;fn codes_of

# The context of a case change, Unicode's (The Unicode Standard, 3.13, Table 3-17), each looking from the
# character at $i in $c. A mark run is the combining marks after a letter: those of any class but 0.

# Final_Sigma, before: a cased letter comes before $i, only case-ignorable ones between.
fn cased_before [lent $c: ls[u32], let $i: u64[min: 1]] -> bool:
    if $i > ls.len(lend $c):
        ret false;
    ;if
    for $k: u64 [from: $i - 1, to: 1, by: -1]:
        if my.case_tables.cased($c[pos: $k]):
            ret true;
        ;if
        if not my.case_tables.case_ignorable($c[pos: $k]):
            ret false;
        ;if
    ;for
    ret false;
;fn cased_before

# Final_Sigma, after: a cased letter comes after $i, only case-ignorable ones between.
fn cased_after [lent $c: ls[u32], let $i: u64[min: 1]] -> bool:
    if $i >= ls.len(lend $c):
        ret false;
    ;if
    for $k: u64 [from: $i + 1, to: ls.len(lend $c), by: 1]:
        if my.case_tables.cased($c[pos: $k]):
            ret true;
        ;if
        if not my.case_tables.case_ignorable($c[pos: $k]):
            ret false;
        ;if
    ;for
    ret false;
;fn cased_after

# After_Soft_Dotted: a soft-dotted letter (`i`, `j`, `į`, …) comes before $i in the same mark run, no mark
# above between.
fn after_soft_dotted [lent $c: ls[u32], let $i: u64[min: 1]] -> bool:
    if $i > ls.len(lend $c):
        ret false;
    ;if
    for $k: u64 [from: $i - 1, to: 1, by: -1]:
        let $x := $c[pos: $k];
        if my.case_tables.soft_dotted($x):
            ret true;
        ;if
        if my.case_tables.above($x) or not my.case_tables.marks($x):
            ret false;
        ;if
    ;for
    ret false;
;fn after_soft_dotted

# After_I: a capital `I` comes before $i in the same mark run, no mark above between.
fn after_capital_i [lent $c: ls[u32], let $i: u64[min: 1]] -> bool:
    if $i > ls.len(lend $c):
        ret false;
    ;if
    for $k: u64 [from: $i - 1, to: 1, by: -1]:
        let $x := $c[pos: $k];
        if $x == hex 49:
            ret true;
        ;if
        if my.case_tables.above($x) or not my.case_tables.marks($x):
            ret false;
        ;if
    ;for
    ret false;
;fn after_capital_i

# More_Above: a mark above comes after $i in the same mark run.
fn more_above [lent $c: ls[u32], let $i: u64[min: 1]] -> bool:
    if $i >= ls.len(lend $c):
        ret false;
    ;if
    for $k: u64 [from: $i + 1, to: ls.len(lend $c), by: 1]:
        let $x := $c[pos: $k];
        if my.case_tables.above($x):
            ret true;
        ;if
        if not my.case_tables.marks($x):
            ret false;
        ;if
    ;for
    ret false;
;fn more_above

# Before_Dot: a combining dot above (U+0307) comes after $i in the same mark run, no other mark above between.
fn before_dot [lent $c: ls[u32], let $i: u64[min: 1]] -> bool:
    if $i >= ls.len(lend $c):
        ret false;
    ;if
    for $k: u64 [from: $i + 1, to: ls.len(lend $c), by: 1]:
        let $x := $c[pos: $k];
        if $x == hex 307:
            ret true;
        ;if
        if my.case_tables.above($x) or not my.case_tables.marks($x):
            ret false;
        ;if
    ;for
    ret false;
;fn before_dot

# The UTF-8 bytes of the character $c, after $out. Every $c here comes from Unicode's case tables, so it
# is always a character; if it ever is not, std has a bug, and the program stops saying so.
fn append_point [lent.mut $out: ls[u8], let $c: u32]:
    let $ch := char.%point.from_code($c) ?? bug "a case table gave a code that is no character";
    let $t := to.str.utf8($ch);
    for $b in str.each.byte(lend $t):
        ls.append(lend.mut $out, value: $b);
    ;for
;fn append_point

# The bytes of $s, in order.
fn bytes_of [lent $s: str.utf8] -> ls[u8]:
    var $out: ls[u8] = [];
    for $b in str.each.byte(lend $s):
        ls.append(lend.mut $out, value: $b);
    ;for
    ret give $out;
;fn bytes_of

# Whether $part sits in $text with its first byte at $pos.
fn sits_at [lent $text: ls[u8], lent $part: ls[u8], let $pos: u64[min: 1]] -> bool:
    let $n := ls.len(lend $text);
    let $m := ls.len(lend $part);
    if $m > $n:
        ret false;
    ;if
    if $pos > $n - $m + 1:
        ret false;
    ;if
    for $k: u64 [from: $pos, to: ($pos + $m - 1), by: 1]:
        if $text[pos: $k] != $part[pos: $k - $pos + 1]:
            ret false;
        ;if
    ;for
    ret true;
;fn sits_at

# Bytes cut from UTF-8 text where characters start and end, back into text. They are always
# UTF-8; if they ever are not, std has a bug, and the program stops saying so.
fn text_of [let $bytes: ls[u8]] -> str.utf8:
    ret to.str.utf8(lend $bytes) ?? bug "text cut on a character's edge was not UTF-8";
;fn text_of
