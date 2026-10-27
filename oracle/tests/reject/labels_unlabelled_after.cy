collection.function say:
    labels [$t as value];
    say [$n: integer.signed.32_bit, $t: integer.signed.32_bit] -> integer.signed.32_bit:
        return $t;
    end of say
end of collection.function say
!MAIN:
    let $x := say(value: 2, 1);
!end of MAIN
