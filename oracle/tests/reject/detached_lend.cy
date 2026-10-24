MAIN { let $x := "a"; concur.detached with [lend $x] { }; }
