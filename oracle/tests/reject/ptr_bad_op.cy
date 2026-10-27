%extern.C proc malloc [let $n: int.unsign.64] -> ptr.mut[int.unsign.8];
!MAIN:
    !no_check let $p := malloc(2);
    !no_check let $q := ptr.deref[$p];
!end of MAIN
