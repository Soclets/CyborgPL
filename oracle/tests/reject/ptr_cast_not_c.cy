%extern.C proc malloc [let $bytes: u64] -> ptr.mut[u8];
rec Plain:
    var x: i32;
;rec Plain
MAIN:
    no_check let $raw := malloc(4);
    no_check let $p := ptr.cast[Plain]($raw);
;MAIN
