fn touch [lent $text: str.utf8]:
    change $text to "b";
end of fn touch
!MAIN:
    let $s := "a";
    touch(lend $s);
!end of MAIN
