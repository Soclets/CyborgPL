MAIN { let $c: channel[str.utf8] = channel.empty; let $s := "x"; channel.send(lend $c, value: lend $s); }
