function f [] -> integer.signed.32_bit or fail[a]:
    fail a;
end of function f
!MAIN:
    let $x := f() otherwise:
        case a:
            std::print.stored("hm");
        end of case a
    end of otherwise;
!end of MAIN
