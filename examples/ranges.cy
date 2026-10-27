# Range types (design/input.md, 2026-09-20): a type with bounds, provably within them.
rec Percent:
    var value: int.unsign.8[min: 0, max: 100];
end of rec Percent

fn scale [$p: Percent, $total: int.sign.32[min: 0, max: 1000]] -> int.sign.32[min: 0, max: 1000]:
    ret ($total times to.int.sign.32($p.value)) div.floor 100;
end of fn scale

!MAIN:
    let $half := Percent[value = 50];
    std::print.shortest(scale(give $half, 800), newln);
    let $raw := 250;
    if $raw at_least 0 and $raw at_most 100:
        let $ok: int.sign.32[min: 0, max: 100] = $raw;                   # the if is the guardrail; a check site
        std::print.shortest($ok, newln);
    end of if
    else:
        std::print.stored("250 is not a percentage", newln);
    end of else
    let $r := to.int.unsign.8[min: 0, max: 100]("42") otherwise 0;          # into a range type: fails, does not check
    let $bad := to.int.unsign.8[min: 0, max: 100]("142") otherwise 0;
    let $huge := to.int.unsign.8[min: 0, max: 100]("99999999999") otherwise 0;   # not even a u8: still out of range, not a crash
    std::print.shortest($r, " ", $bad, " ", $huge, newln);
    let $unit: float.bin.64[min: 0.0, max: 1.0] = 0.25;
    std::print.shortest($unit, newln);
!end of MAIN
