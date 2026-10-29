# `bug "…";` (owner, 2026-10-29): the program found a bug in itself and stops, exit 101, `error[bug]` and the message on
# stderr. Not an effect, so a `fn` may use it, as a statement or after `??` / `otherwise`.
fn half [let $n: i32] -> i32:
    if ($n mod.floor 2) != 0:
        bug "half was handed an odd number";
    ;if
    ret $n div.floor 2;
;fn half

fn text_of [lent $b: std::ls[u8]] -> str.utf8:
    let $t := to.str.utf8(lend $b) ?? bug "text_of was handed bytes that are not UTF-8";
    ret give $t;
;fn text_of

MAIN:
    std::print(half(8), newln);
    let $ok: std::ls[u8] := [104, 105];
    std::print(text_of(lend $ok), newln);
    std::print(half(7), newln);
;MAIN
