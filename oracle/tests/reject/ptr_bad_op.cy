%extern.C proc malloc [let $n: u64] -> ptr.mut[u8];
MAIN:
    no_check let $p := malloc(2);
    no_check let $q := ptr.deref[$p];
;MAIN
