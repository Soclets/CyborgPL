collection.function say:
    labels [$a as first, $b as second];
    say [$a: integer.signed.32_bit, $b: integer.signed.32_bit] -> integer.signed.32_bit:
        return $a;
    end of say
end of collection.function say
!MAIN:
    let $x := say(second: 2, first: 1);
!end of MAIN
