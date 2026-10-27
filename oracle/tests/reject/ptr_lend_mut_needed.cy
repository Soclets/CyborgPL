%extern.C proc fill [let $b: ptr.mut[int.unsign.8]];
!MAIN:
    let $xs := [1, 2];
    !no_check fill(lend $xs);
!end of MAIN
