incl.%C.system["stdio.h"];

# C's printf from stdio.h, called bare (owner, 2026-10-10): std's old `printf` spelling no longer takes the name
MAIN:
    let $f := %C.str.make("%d apples");
    no_check printf(lend $f, 5);
;MAIN
