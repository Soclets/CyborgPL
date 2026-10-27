# A map's count bound (decided 2026-09-22), and bounds on the things inside — types composing.
!MAIN:
    variable $small: std::map[string.utf8[minimum: 1, maximum: 3 bytes], integer.signed.32_bit[minimum: 0, maximum: 10]][minimum: 0, maximum: 2] = std::map.empty;
    std::map.set(lend.mutable $small, key: "ab", value: 5);
    std::map.set(lend.mutable $small, key: "cd", value: 7);
    let $n := std::map.count(lend $small);         # u64[min: 0, max: 2]
    let $tiny: integer.signed.8_bit = $n;                      # proven by the bound
    std::print.shortest($tiny, " entries", newline);
    std::map.remove(lend.mutable $small, key: "ab") otherwise:
        case missing:
        end of case missing
    end of otherwise;
    std::map.set(lend.mutable $small, key: "ef", value: 10);         # back to two: fits
    std::print.shortest(std::map.count(lend $small), newline);
!end of MAIN
