fn f [] -> fail[why]:
    fail why;
end of fn f
!MAIN:
    f()-
    otherwise:
        case why $w:
        end of case why
    end of otherwise;
!end of MAIN
