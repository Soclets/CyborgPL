# `gives` is spelled `giveS`, the capital S keeping it apart from `give` (owner, 2026-09-29)
union.tagged S:
    a;
    b;
end of union.tagged S
!MAIN:
    let $x: integer.signed.32_bit = match S.a:
        case a:
            gives 1;
        end of case a
        case b:
            gives 2;
        end of case b
    end of match;
!end of MAIN
