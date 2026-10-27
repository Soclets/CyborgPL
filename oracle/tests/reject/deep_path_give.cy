# giving a part out of what holds it would leave a hole
record In:
    name: string.utf8;
end of record In
record Out:
    inn: In;
end of record Out
!MAIN:
    let $o := Out[inn = In[name = "a"]];
    let $n := give $o.inn.name;
!end of MAIN
