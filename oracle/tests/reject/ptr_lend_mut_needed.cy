%extern.C proc fill [$b: ptr.mut[u8]];
MAIN:
    let $xs := [1, 2];
    no_check fill(lend $xs);
;MAIN
