# a plain value at the end of a path owns nothing, so it takes no verb
record In:
    k: integer.signed.32_bit;
end of record In
record Out:
    inn: In;
end of record Out
!MAIN:
    let $o := Out[inn = In[k = 1]];
    let $n := copy $o.inn.k;
!end of MAIN
