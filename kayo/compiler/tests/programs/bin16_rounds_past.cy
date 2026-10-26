# A f64 into a f16, known only at run time: 65520 rounds past a f16's largest: the narrowing's check fires.
# (oracle 3996169: rounded where it lands; what rounds to a finite value fits)
%extern.C fn exit [$code: i32];
%extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
MAIN {
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    let mut $x: f64 = 1.0;
    if $there { change $x = 65520.0; }
    let $h: f16 = $x;
    if $h equal_to 65504.0 { no_check my.exit(3); }
    no_check my.exit(4);
}
