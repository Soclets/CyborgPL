fn f [] -> fail[why: int.sign.32]:
    fail why["x"];
end of fn f
!MAIN:
    f()-
    otherwise:
        case why $w:
        end of case why
    end of otherwise;
!end of MAIN
