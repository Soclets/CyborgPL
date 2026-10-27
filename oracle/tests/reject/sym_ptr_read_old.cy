%extern.C proc malloc [$bytes: int.unsign.64] -> ptr.mut[int.unsign.8];
!MAIN:
    !no_check let $p := malloc(4);
    !no_check let $c := *$p;
!end of MAIN
