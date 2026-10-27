# The the base, alone: no limbs, no std (owner, 2026-10-08, built 2026-10-09).
# Operators, true-value numbers, `to.` conversions, range types and generics are the base's now;
# Signatar, CyborgPL's base, had none of them. Nothing is printed: printing is std's.

function biggest <T: any> [$a: <T>, $b: <T>, $pick_a: boolean] -> <T>:
    if $pick_a:
        return give $a;
    end of if
    return give $b;
end of function biggest

!MAIN:
    let $a: integer.signed.32_bit = 2;
    let $b: integer.signed.32_bit = $a plus 3 times 4;
    let $small := to.integer.signed.8_bit($b);                     # proven to fit: 14
    let $level: integer.signed.32_bit[minimum: 0, maximum: 10] = 7;
    let $wrapped: integer.unsigned.8_bit = 250 plus.wrap 10;
    let $bigger := biggest($b, $a, $b greater_than $a);
    let $same := $bigger equal_to $b and not ($small less_than 0);
!end of MAIN
