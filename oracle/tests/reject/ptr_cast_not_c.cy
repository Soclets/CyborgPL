%extern.C proc malloc [$bytes: int.unsign.64] -> ptr.mut[int.unsign.8];
rec Plain:
    var x: int.sign.32;
end of rec Plain
!MAIN:
    !no_check let $raw := malloc(4);
    !no_check let $p := ptr.cast[Plain]($raw);
!end of MAIN
