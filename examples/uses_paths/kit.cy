# Another kit's names always take its path (owner, 2026-10-22): `greet::hello()`, `greet::Tally[…]`, `greet::Tone.loud`.
# Its operators, word and glyph, may be written bare or with the path, and a `use` names one of its functions for this file.
use greet::formal as formal;

function describe [$t: greet::Tone] -> string.utf8:
    return match $t:
        case soft:
            is "soft";
        end of case soft
        case loud:
            is "loud";
        end of case loud
    end of match;
end of function describe

!MAIN:
    std::print.shortest(greet::hello(), " ", greet::wave(), " ", formal(), newline);
    let $t := greet::Tally[n = 2] merge greet::Tally[n = 3];
    let $u := greet::Tally[n = 1] greet::merge $t;
    let $v := $t <+> $u;
    let $w := $t greet::<+> $u;
    std::print.shortest($t.n, " ", $u.n, " ", $v.n, " ", $w.n, newline);
    let $b: greet::Note = greet::Note[text = "with the path"];
    std::print.shortest($b.text, ", ", describe(greet::Tone.loud), newline);
!end of MAIN
