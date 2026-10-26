%extern.C fn malloc [$bytes: u64] -> ptr.mut[u8];
struct Plain {
    x: i32
}
MAIN {
    no_check let $raw := malloc(4);
    no_check let $p := ptr.cast[Plain]($raw);
}
