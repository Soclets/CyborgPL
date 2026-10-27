%extern.C proc malloc [let $bytes: int.unsign.64] -> ptr.mut[int.unsign.8];
!MAIN:
    !no_check let $raw := malloc(4);
    let $p := ptr.cast[int.sign.32]($raw);
!end of MAIN
