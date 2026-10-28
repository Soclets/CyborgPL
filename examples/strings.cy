# Text literals: the escapes `\"` and `\\`, typed tabs, and `newln`.
MAIN:
    std::print.stored("a\nb", newln);               # newln inside quotes is a backslash and an n
    std::print.stored("say \"hi\"", newln);         # \" is the one escape that is needed
    std::print.stored("back\\slash", newln);        # and \\ for the backslash itself
    std::print.stored("tab	here", newln);           # a tab is typed, not escaped
    let $nl := newln;                              # the newline is the bare newln outside quotes
    std::print.stored("one", $nl, "two", $nl);
;MAIN
