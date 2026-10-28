fn n [lent $s: str.utf8] -> u64:
    ret str.bytes[lend $s];
;fn n
MAIN:
    let $h := "hi";
    let $k := n[&$h];
;MAIN
