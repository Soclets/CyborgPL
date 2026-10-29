%extern.C proc getenv [$n: ptr[u8]] -> ptr[u8];
MAIN:
    let $k := "PATH";
    no_check let $p := getenv(lend $k);
    no_check ptr.write($p, value: 1);
;MAIN
