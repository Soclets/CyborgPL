# Text literals: the escapes `\"` and `\\`, typed tabs, and `newln`.
MAIN:
    std::println.stored("a\nb");               # newln inside quotes is a backslash and an n
    std::println.stored("say \"hi\"");         # \" is the one escape that is needed
    std::println.stored("back\\slash");        # and \\ for the backslash itself
    std::println.stored("tab	here");           # a tab is typed, not escaped
    let $nl := newln;                              # the newline is the bare newln outside quotes
    std::print.stored("one", $nl, "two", $nl);
;MAIN
