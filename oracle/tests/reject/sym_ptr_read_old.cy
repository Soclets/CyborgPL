extern.C fn malloc ['bytes': uint64] -> ptr.mut[uint8];
MAIN {
    no_check let 'p' := malloc[4];
    no_check let 'c' := *'p';
}
