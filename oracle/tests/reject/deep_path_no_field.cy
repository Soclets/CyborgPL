# a deep path names fields its struct has
record In:
    k: integer.signed.32_bit;
end of record In
record Out:
    inn: In;
end of record Out
!MAIN:
    let $o := Out[inn = In[k = 1]];
    std::print.shortest($o.inn.j, newline);
!end of MAIN
