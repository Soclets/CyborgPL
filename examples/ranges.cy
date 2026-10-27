# Range types (design/input.md, 2026-09-20): a type with bounds, provably within them.
record Percent:
    value: integer.unsigned.8_bit[minimum: 0, maximum: 100];
end of record Percent

function scale [$p: Percent, $total: integer.signed.32_bit[minimum: 0, maximum: 1000]] -> integer.signed.32_bit[minimum: 0, maximum: 1000]:
    return ($total times to.integer.signed.32_bit($p.value)) divide.floor 100;
end of function scale

!MAIN:
    let $half := Percent[value = 50];
    std::print.shortest(scale(give $half, 800), newline);
    let $raw := 250;
    if $raw at_least 0 and $raw at_most 100:
        let $ok: integer.signed.32_bit[minimum: 0, maximum: 100] = $raw;                   # the if is the guardrail; a check site
        std::print.shortest($ok, newline);
    end of if
    else:
        std::print.stored("250 is not a percentage", newline);
    end of else
    let $r := to.integer.unsigned.8_bit[minimum: 0, maximum: 100]("42") otherwise 0;          # into a range type: fails, does not check
    let $bad := to.integer.unsigned.8_bit[minimum: 0, maximum: 100]("142") otherwise 0;
    let $huge := to.integer.unsigned.8_bit[minimum: 0, maximum: 100]("99999999999") otherwise 0;   # not even a u8: still out of range, not a crash
    std::print.shortest($r, " ", $bad, " ", $huge, newline);
    let $unit: float.binary.64_bit[minimum: 0.0, maximum: 1.0] = 0.25;
    std::print.shortest($unit, newline);
!end of MAIN
