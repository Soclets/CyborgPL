function touch [lent $text: string.utf8]:
    change $text = "b";
end of function touch
!MAIN:
    let $s := "a";
    touch(lend $s);
!end of MAIN
