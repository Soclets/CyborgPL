# a case that gives a value needs a match used as one (2026-09-29)
union.tagged S:
    a;
    b;
end of union.tagged S
!MAIN:
    match S.a:
        case a:
            is 1;
        end of case a
        case b:
            is 2;
        end of case b
    end of match
!end of MAIN
