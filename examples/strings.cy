# Text literals: the escapes `\"` and `\\`, typed tabs, and `newln`.
!MAIN:
    std::print.stored("a\nb", newline);               # newln inside quotes is a backslash and an n
    std::print.stored("say \"hi\"", newline);         # \" is the one escape that is needed
    std::print.stored("back\\slash", newline);        # and \\ for the backslash itself
    std::print.stored("tab	here", newline);           # a tab is typed, not escaped
    let $nl := newline;                              # the newline is the bare newln outside quotes
    std::print.stored("one", $nl, "two", $nl);
!end of MAIN
