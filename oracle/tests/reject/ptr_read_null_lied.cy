%extern.C proc getenv [$n: ptr[int.unsign.8]] -> ptr[int.unsign.8];
!MAIN:
    let $k := "CYBORG_NOPE_XYZ";
    !no_check let $p := getenv(lend $k);
    !no_check let $b := ptr.read($p);
!end of MAIN
