function f [] -> integer.signed.32_bit or fail[a]:
    fail a;
end of function f
!MAIN:
    let $x := f() otherwise:
        case a:
            std::exit(code: 1);
        end of case a
        case z:
            std::exit(code: 1);
        end of case z
    end of otherwise;
!end of MAIN
