extern.C fn malloc ['bytes': uint64] -> ptr.mut[uint8];
struct Plain {
    x: int32
}
MAIN {
    no_check let 'raw' := malloc[4];
    no_check let 'p' := ptr.cast[Plain]['raw'];
}
