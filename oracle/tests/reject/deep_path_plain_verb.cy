# a plain value at the end of a path owns nothing, so it takes no verb
rec In:
    k: int.sign.32;
end of rec In
rec Out:
    inn: In;
end of rec Out
!MAIN:
    let $o := Out[inn = In[k = 1]];
    let $n := copy $o.inn.k;
!end of MAIN
