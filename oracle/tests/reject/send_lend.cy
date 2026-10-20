MAIN { let 'c' := channel[str.utf8]; let 's' := "x"; channel.send[lend 'c', value: lend 's']; }
