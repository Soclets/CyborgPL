collection.function c:
    for $n in [1, 2, 3]:
        named "t" concatenate to.stored.string.utf8($n);
        [] -> integer.signed.32_bit:
            return $n;
        end of named
    end of for
    #generated_2_lines_below
    c.t1 [] -> integer.signed.32_bit:
        return 1;
    end of c.t1
    c.t2 [] -> integer.signed.32_bit:
        return 2;
    end of c.t2
end of collection.function c
!MAIN:
!end of MAIN
