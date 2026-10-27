fn n [lent $s: str.utf8] -> int.unsign.64:
    ret str.bytes[lend $s];
end of fn n
!MAIN:
    let $h := "hi";
    let $k := n[&$h];
!end of MAIN
