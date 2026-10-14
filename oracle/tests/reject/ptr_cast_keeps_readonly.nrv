extern.C fn getenv ['name': ptr[uint8]] -> ptr[uint8];
MAIN {
    let 'key' := "HOME";
    no_check let 'home' := getenv[lend 'key'];
    no_check let 'p' := ptr.cast[int8]['home'];
    no_check ptr.write['p', value 0];
}
