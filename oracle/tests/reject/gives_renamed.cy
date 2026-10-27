# `gives` is spelled `giveS`, the capital S keeping it apart from `give` (owner, 2026-09-29)
union.tag S:
    a;
    b;
end of union.tag S
!MAIN:
    let $x: int.sign.32 = match S.a:
        case a:
            gives 1;
        end of case a
        case b:
            gives 2;
        end of case b
    end of match;
!end of MAIN
