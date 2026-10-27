union.tag S:
    a;
    b;
end of union.tag S
!MAIN:
    let $s := S.a;
    match $s:
        case a $x:
        end of case a
        case b:
        end of case b
    end of match
!end of MAIN
