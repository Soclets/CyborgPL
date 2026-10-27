sig Printable <T>:
    fn show [lent $x: <T>] -> str.utf8;
end of sig Printable
rec V:
    var x: int.sign.32;
end of rec V
V satisfies Printable;
!MAIN:
!end of MAIN
