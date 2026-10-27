# giving a part out of what holds it would leave a hole
rec In:
    name: str.utf8;
end of rec In
rec Out:
    inn: In;
end of rec Out
!MAIN:
    let $o := Out[inn = In[name = "a"]];
    let $n := give $o.inn.name;
!end of MAIN
