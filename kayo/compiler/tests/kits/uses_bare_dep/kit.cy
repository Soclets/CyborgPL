%extern.C fn exit [$code: i32];
MAIN { no_check my.exit(bare_dep::sum(3, 4) plus 1); }
