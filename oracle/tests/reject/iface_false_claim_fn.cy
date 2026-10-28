sig Printable <T>:
    fn show [lent $x: <T>] -> str.utf8;
;sig Printable
rec V:
    var x: i32;
;rec V
V satisfies Printable;
MAIN:
;MAIN
