%extern.C proc getenv [$name: ptr[int.unsign.8]] -> ptr[int.unsign.8];
!MAIN:
    let $key := "HOME";
    !no_check let $home := getenv(lend $key);
    !no_check let $p := ptr.cast[int.sign.8]($home);
    !no_check ptr.write($p, value: 0);
!end of MAIN
