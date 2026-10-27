union.tag S:
    a;
    b;
end of union.tag S
!MAIN:
    let $x := match S.a:
        case a:
            1
        end of case a
        case b:
            2
        end of case b
    end of match;
!end of MAIN
