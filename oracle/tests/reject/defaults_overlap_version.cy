collection.function say:
    labels [$t as value, $k as count];
    defaults [$k = 1];
    versions say:
        [$n: integer.signed.32_bit, $t: integer.signed.32_bit, $k: integer.signed.32_bit] -> integer.signed.32_bit:
            return 1;
        end of version
        [$n: integer.signed.32_bit, $t: integer.signed.32_bit] -> integer.signed.32_bit:
            return 2;
        end of version
    end of versions say
end of collection.function say
!MAIN:
!end of MAIN
