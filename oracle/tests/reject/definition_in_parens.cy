fn twice ($n: int.sign.64) -> int.sign.64:
    ret $n times 2;
end of fn twice
!MAIN:
    std::print.shortest(twice(2), newln);
!end of MAIN
