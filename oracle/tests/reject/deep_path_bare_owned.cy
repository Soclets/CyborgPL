# an owned value at the end of a deep path takes a verb, as one step does
record In:
    name: string.utf8;
end of record In
record Out:
    inn: In;
end of record Out
!MAIN:
    let $o := Out[inn = In[name = "a"]];
    let $n := $o.inn.name;
!end of MAIN
