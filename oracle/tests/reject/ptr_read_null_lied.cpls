extern.C fn getenv ['n': ptr[uint8]] -> ptr[uint8];
MAIN {
    let 'k' := "CYBORG_NOPE_XYZ";
    no_check let 'p' := getenv[lend 'k'];
    no_check let 'b' := ptr.read['p'];
}
