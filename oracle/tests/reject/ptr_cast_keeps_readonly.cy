extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
MAIN {
    let 'key' := "HOME";
    no_check let 'home' := getenv[lend 'key'];
    no_check let 'p' := ptr.cast[i8]['home'];
    no_check ptr.write['p', value: 0];
}
