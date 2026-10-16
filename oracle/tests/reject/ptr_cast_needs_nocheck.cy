extern.C fn malloc ['bytes': uint64] -> ptr.mut[uint8];
MAIN {
    no_check let 'raw' := malloc[4];
    let 'p' := ptr.cast[int32]['raw'];
}
