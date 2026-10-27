collection.function mx:
    versions mx:
        [$a: integer.signed.32_bit, $b: integer.signed.32_bit] -> integer.signed.32_bit:
            return $a plus $b;
        end of version
        [$a: integer.signed.32_bit, $b: integer.signed.64_bit] -> integer.signed.32_bit:
            return $a minus to.integer.signed.32_bit($b);
        end of version
    end of versions mx
end of collection.function mx
!MAIN:
    let $v: integer.signed.32_bit = 4;
    let $p: boolean = to.integer.signed.64_bit(1) equal_to to.integer.signed.64_bit(2);
    std::print.shortest("before", newline);
    if $p:
        std::print.shortest(mx($v, 5), newline);
    end of if
    std::print.shortest("after", newline);
!end of MAIN
