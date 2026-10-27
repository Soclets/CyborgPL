%extern.C proc malloc [let $n: int.unsign.64] -> ptr.mut[int.unsign.8];
!MAIN:
    !no_check let $p := malloc(2);
    !no_check let $b := ptr.read(ptr.at($p, by: 5));
!end of MAIN
