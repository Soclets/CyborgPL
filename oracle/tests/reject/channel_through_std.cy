!MAIN:
    let $c: %channel[integer.signed.32_bit] = %channel.empty;
    std::channel.send(lend $c, value: 1);
!end of MAIN
