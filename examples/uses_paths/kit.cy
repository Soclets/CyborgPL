# Another kit's names always take its path (owner, 2026-10-22): `greet::hello()`, `greet::Tally[…]`, `greet::Tone.loud`.
# Its operators, word and glyph, may be written bare or with the path, and a `use` names one of its functions for this file.
use greet::formal as formal;

fn describe [$t: greet::Tone] -> str.utf8:
    ret match $t:
        case soft:
            is "soft";
        ;case soft
        case loud:
            is "loud";
        ;case loud
    ;match;
;fn describe

MAIN:
    std::print(greet::hello(), " ", greet::wave(), " ", formal(), newln);
    let $t := greet::Tally[n = 2] merge greet::Tally[n = 3];
    let $u := greet::Tally[n = 1] greet::merge $t;
    let $v := $t <+> $u;
    let $w := $t greet::<+> $u;
    std::print($t.n, " ", $u.n, " ", $v.n, " ", $w.n, newln);
    let $b: greet::Note := greet::Note[text = "with the path"];
    std::print($b.text, ", ", describe(greet::Tone.loud), newln);
;MAIN
