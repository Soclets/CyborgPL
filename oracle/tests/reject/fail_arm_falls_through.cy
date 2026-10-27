fn f [] -> int.sign.32 or fail[a]:
    fail a;
end of fn f
!MAIN:
    let $x := f()-
    otherwise:
        case a:
            std::print.stored("hm");
        end of case a
    end of otherwise;
!end of MAIN
