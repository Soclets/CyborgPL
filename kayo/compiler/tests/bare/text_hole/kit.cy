# A format hole holding text, with no OS: the one scratch buffer could not
# be sure to hold it, so it is not built yet.
MAIN:
    std::print.format("{width: 8 cols}{newln}", "abc");
    std::exit(code: 0);
;MAIN
