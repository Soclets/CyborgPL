%extern.C proc free [$p: ptr.mut[u8]];
MAIN:
    no_check free(ptr.null[]);
;MAIN
