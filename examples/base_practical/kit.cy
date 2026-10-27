# The the base, alone: no limbs, no std (owner, 2026-10-08, built 2026-10-09).
# Operators, true-value numbers, `to.` conversions, range types and generics are the base's now;
# Signatar, CyborgPL's base, had none of them. Nothing is printed: printing is std's.

fn biggest <T: any> [let $a: <T>, let $b: <T>, let $pick_a: bool] -> <T>:
    if $pick_a:
        ret give $a;
    end of if
    ret give $b;
end of fn biggest

!MAIN:
    let $a: int.sign.32 := 2;
    let $b: int.sign.32 := $a plus 3 times 4;
    let $small := to.int.sign.8($b);                     # proven to fit: 14
    let $level: int.sign.32[min: 0, max: 10] := 7;
    let $wrapped: int.unsign.8 := 250 plus.wrap 10;
    let $bigger := biggest($b, $a, $b greater_than $a);
    let $same := $bigger eq $b and not ($small less_than 0);
!end of MAIN
