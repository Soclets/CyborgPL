%extern.C proc free [let $p: ptr.mut[int.unsign.8]];
!MAIN:
    !no_check free(ptr.null[]);
!end of MAIN
