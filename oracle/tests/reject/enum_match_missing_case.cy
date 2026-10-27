union.tag S:
    a;
    b;
    c;
end of union.tag S
!MAIN:
    let $s := S.a;
    match $s:
        case a:
        end of case a
        case b:
        end of case b
    end of match
!end of MAIN
