%extern.C proc free [$p: ptr.mut[int.unsign.8]];
!MAIN:
    !no_check free(ptr.null[]);
!end of MAIN
