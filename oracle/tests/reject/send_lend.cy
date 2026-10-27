!MAIN:
    let $c: %channel[string.utf8] = %channel.empty; let $s := "x"; %channel.send(lend $c, value: lend $s);
!end of MAIN
