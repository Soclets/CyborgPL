# Range types (design/input.md, 2026-09-20): a type with bounds, provably within them.
rec Percent:
    var value: u8[min: 0, max: 100];
;rec Percent

fn scale [let $p: Percent, let $total: i32[min: 0, max: 1000]] -> i32[min: 0, max: 1000]:
    ret ($total x to.i32($p.value)) div.floor 100;
;fn scale

MAIN:
    let $half := Percent[value = 50];
    std::println(scale(give $half, 800));
    let $raw := 250;
    if $raw >= 0 and $raw <= 100:
        let $ok: i32[min: 0, max: 100] := $raw;                   # the if is the guardrail; a check site
        std::println($ok);
    else:
        std::println.stored("250 is not a percentage");
    ;if
    let $r := to.u8[min: 0, max: 100]("42") ?? 0;          # into a range type: fails, does not check
    let $bad := to.u8[min: 0, max: 100]("142") ?? 0;
    let $huge := to.u8[min: 0, max: 100]("99999999999") ?? 0;   # not even a u8: still out of range, not a crash
    std::println($r, " ", $bad, " ", $huge);
    let $unit: f64[min: 0.0, max: 1.0] := 0.25;
    std::println($unit);
;MAIN
