fn touch [lent $text: str.utf8]:
    change $text to "b";
;fn touch
MAIN:
    let $s := "a";
    touch(lend $s);
;MAIN
