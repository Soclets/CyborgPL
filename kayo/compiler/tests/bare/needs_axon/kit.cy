# A growing list needs Axon, the runtime, which is not built for a chip
# with no OS: refused before linking, naming what needs it.
MAIN:
    var $xs: std::ls[i32] = [1, 2];
    std::ls.append(lend.mut $xs, value: 3);
;MAIN
