!MAIN:
    let $c: %channel[int.sign.32] := %channel.empty;
    std::channel.send(lend $c, value: 1);
!end of MAIN
