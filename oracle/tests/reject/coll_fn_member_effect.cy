coll.fn greet:
    member greet [lent $who: str.utf8] -> str.utf8:
        ret $who;
    end of member greet
    member greet.loud [lent $who: str.utf8]:
        std::print.shortest($who, newln);
    end of member greet.loud
end of coll.fn greet

!MAIN:
    greet.loud("hi");
!end of MAIN
