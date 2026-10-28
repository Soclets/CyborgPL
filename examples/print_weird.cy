# `std::print.arrows` (was `print.weird`) — C++'s `cout <<`, CyborgPL style (owner, 2026-10-01): each value follows a `<-`,
# the arrow pointing into the output, as `<-` points into a variable in `inline.ASM`. `.arrows.stored`
# shows the stored form, as `print.stored` does; `.arrows.shortest` the shortest, as `print` does. The
# strongest modifier comes first: `.arrows` before the form.
MAIN:
    let $tenth: f64 := 0.1;
    let $n := 3;
    std::print.arrows.shortest <- "tenth is " <- $tenth <- ", n is " <- $n <- newln;
    std::print.arrows.stored <- "stored: " <- $tenth <- newln;
;MAIN
