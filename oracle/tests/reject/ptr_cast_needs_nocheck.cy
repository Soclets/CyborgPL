%extern.C proc malloc [let $bytes: u64] -> ptr.mut[u8];
MAIN:
    no_check let $raw := malloc(4);
    let $p := ptr.cast[i32]($raw);
;MAIN
