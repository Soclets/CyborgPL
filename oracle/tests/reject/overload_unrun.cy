collection.function ov:
    versions ov.flip:
        [$x: integer.signed.32_bit] -> integer.signed.32_bit:
            return 1;
        end of version
        [$x: integer.signed.64_bit] -> integer.signed.32_bit:
            return 2;
        end of version
    end of versions ov
end of collection.function ov
!MAIN:
    let $p := false;
    std::print.shortest("before", newline);
    if $p:
        std::print.shortest(ov.flip(77), newline);
    end of if
!end of MAIN
