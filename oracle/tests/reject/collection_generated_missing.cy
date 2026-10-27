collection.function c:
    for $n in [1, 2]:
        named "t" concatenate to.stored.string.utf8($n);
        [] -> integer.signed.32_bit:
            return $n;
        end of named
    end of for
end of collection.function c
!MAIN:
!end of MAIN
