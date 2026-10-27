!MAIN:
    let $n := to.integer.signed.32_bit[minimum: 0, maximum: 100]("42") otherwise:
        case not_a_number:
            std::exit(code: 1);
        end of case not_a_number
    end of otherwise;
!end of MAIN
