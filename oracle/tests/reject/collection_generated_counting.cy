collection.function c:
    for $n [from: 1, before: 3, by: 1]:
        named "t" concatenate to.stored.string.utf8($n);
        [] -> integer.signed.32_bit:
            return $n;
        end of named
    end of for
    #generated_3_lines_below
    c.t1 [] -> integer.signed.32_bit:
        return 1;
    end of c.t1
    c.t2 [] -> integer.signed.32_bit:
        return 2;
    end of c.t2
    c.t3 [] -> integer.signed.32_bit:
        return 3;
    end of c.t3
end of collection.function c
!MAIN:
!end of MAIN
