!MAIN:
    let $b: boolean = false;
    if $b:
        let $r := to.integer.signed.64_bit(5 shift_right to.integer.unsigned.8_bit(1));
    end of if
!end of MAIN
