extern.C fn malloc ['bytes': u64] -> ptr.mut[u8];
MAIN {
    no_check let 'p' := malloc[4];
    no_check let 'c' := *'p';
}
