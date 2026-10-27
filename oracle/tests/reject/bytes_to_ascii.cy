!MAIN:
    let $b: std::list[integer.unsigned.8_bit] = [104, 105];
    let $t := to.string.ascii($b) otherwise "?";
!end of MAIN
