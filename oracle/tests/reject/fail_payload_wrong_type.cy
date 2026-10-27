function f [] -> fail[why: integer.signed.32_bit]:
    fail why["x"];
end of function f
!MAIN:
    f() otherwise:
        case why $w:
        end of case why
    end of otherwise;
!end of MAIN
