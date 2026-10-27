%extern.C proc getenv [$n: ptr[int.unsign.8]] -> ptr[int.unsign.8];
!MAIN:
    let $k := "PATH";
    !no_check let $p := getenv(lend $k);
    !no_check ptr.write($p, value: 1);
!end of MAIN
