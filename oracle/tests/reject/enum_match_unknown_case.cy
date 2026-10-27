union.tagged S:
    a;
    b;
end of union.tagged S
!MAIN:
    let $s := S.a;
    match $s:
        case a:
        end of case a
        case b:
        end of case b
        case z:
        end of case z
    end of match
!end of MAIN
