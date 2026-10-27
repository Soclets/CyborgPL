collection.function f:
    f [$a: integer.signed.64_bit] -> integer.signed.64_bit:
        return 1;
    end of f
    f [$a: integer.signed.64_bit, $b: integer.signed.64_bit] -> integer.signed.64_bit:
        return 2;
    end of f
end of collection.function f
!MAIN:
!end of MAIN
