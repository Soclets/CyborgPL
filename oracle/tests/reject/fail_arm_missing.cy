fn f [] -> int.sign.32 or fail[a, b]:
    fail a;
end of fn f
!MAIN:
    let $x := f()-
    otherwise:
        !case a:
            ret;
        !end of case a
    end of otherwise;
!end of MAIN
