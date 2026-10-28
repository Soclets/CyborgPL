# giving a part out of what holds it would leave a hole
rec In:
    var name: str.utf8;
;rec In
rec Out:
    var inn: In;
;rec Out
MAIN:
    let $o := Out[inn = In[name = "a"]];
    let $n := give $o.inn.name;
;MAIN
