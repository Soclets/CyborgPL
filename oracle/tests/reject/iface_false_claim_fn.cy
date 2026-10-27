interface Printable <T>:
    function show [lent $x: <T>] -> string.utf8;
end of interface Printable
record V:
    x: integer.signed.32_bit;
end of record V
V satisfies Printable;
!MAIN:
!end of MAIN
