# A f64 into a f16, known only at run time: 65519 rounds to a f16's largest, 65504: it fits.
# (oracle 3996169: rounded where it lands; what rounds to a finite value fits)
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
MAIN:
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    var $x: f64 = 1.0;
    if $there:
        change $x to 65519.0;
    ;if
    let $h: f16 := $x;
    if $h == 65504.0:
        no_check my.exit(3);
    ;if
    no_check my.exit(4);
;MAIN
