# C's structs (owner, 2026-10-10). One whose fields CyborgPL can spell comes through as a `layout.C struct`
# with C's field names (`tm`, 56 bytes as C lays it out); one CyborgPL never looks inside comes through
# opaque, named as C names it (`FILE`, not the C library's own `__sFILE`), only pointed at:
# `fopen` gives a `ptr.mut[FILE]`, and another struct's pointer handed to `fclose` is refused.
# A field named like a CyborgPL keyword keeps its name after the dot (`$b.size`); a function so named is
# reached by path (`my.size(…)`).
incl.%C.system["time.h", "stdio.h"];
incl.%C["headers/shapes.h"];

!MAIN:
    let $b := buf[size = 42, count = 7];
    std::print.shortest("tm is ", size.of[tm], " bytes, buf ", size.of[buf], "; size ", $b.size, ", count ", $b.count, newln);
!end of MAIN
