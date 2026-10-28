# a lookup only looks at its key, so a bare name says `lend` (owner, 2026-10-25)
MAIN:
    let $m: std::map[str.utf8, i32] := std::map.empty;
    let $k: str.utf8 := "a";
    let $v := std::map.get(lend $m, key: $k) ?? 0;
;MAIN
