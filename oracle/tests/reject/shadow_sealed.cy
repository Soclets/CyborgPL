# `no_shadow` in front: the name cannot be shadowed (owner, 2026-09-25)
MAIN:
    let no_shadow $c := 1;
    let shadow $c := 2;
;MAIN
