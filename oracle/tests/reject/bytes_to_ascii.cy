MAIN:
    let $b: std::ls[u8] := [104, 105];
    let $t := to.str.ascii($b) ?? "?";
;MAIN
