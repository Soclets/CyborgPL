!MAIN:
    let $s := "text";
    let $f := function [] -> integer.signed.32_bit with [give $s]:
        return std::string.bytes(lend $s);
    end of function;
    std::print.stored($s);
!end of MAIN
