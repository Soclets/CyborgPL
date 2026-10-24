extern.C fn getenv [$n: ptr[u8]] -> ptr[u8];
MAIN {
    let $k := "CYBORG_NOPE_XYZ";
    no_check let $p := getenv(lend $k);
    no_check let $b := ptr.read($p);
}
