# an owned value at the end of a deep path takes a verb, as one step does
rec In:
    var name: str.utf8;
;rec In
rec Out:
    var inn: In;
;rec Out
MAIN:
    let $o := Out[inn = In[name = "a"]];
    let $n := $o.inn.name;
;MAIN
