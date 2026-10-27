%extern.C proc malloc [let $n: int.unsign.64] -> ptr.mut[int.unsign.8];
!MAIN:
    !no_check let $p := malloc(4);
    let $b := ptr.read($p);
!end of MAIN
