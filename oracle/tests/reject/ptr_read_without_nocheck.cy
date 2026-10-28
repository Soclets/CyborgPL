%extern.C proc malloc [let $n: u64] -> ptr.mut[u8];
MAIN:
    no_check let $p := malloc(4);
    let $b := ptr.read($p);
;MAIN
