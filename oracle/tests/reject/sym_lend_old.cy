function n [lent $s: string.utf8] -> integer.unsigned.64_bit:
    return string.bytes[lend $s];
end of function n
!MAIN:
    let $h := "hi";
    let $k := n[&$h];
!end of MAIN
