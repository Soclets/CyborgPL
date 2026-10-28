%extern.C proc strlen [let $s: ptr[u8]] -> u64[min: 0, max: 10];
MAIN:
    let $s := "x"; no_check let $a := strlen(lend $s);
;MAIN
