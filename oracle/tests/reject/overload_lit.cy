collection.function f:
    versions f:
        [$a: integer.signed.64_bit] -> integer.signed.64_bit:
            return 1;
        end of version
        [$b: integer.signed.32_bit] -> integer.signed.64_bit:
            return 2;
        end of version
    end of versions f
end of collection.function f
!MAIN:
    std::print.shortest(f(1), newline);
!end of MAIN
