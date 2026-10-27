!MAIN:
    let $n := to.int.sign.32[min: 0, max: 100]("42")-
    otherwise:
        case not_a_number:
            std::exit(code: 1);
        end of case not_a_number
    end of otherwise;
!end of MAIN
