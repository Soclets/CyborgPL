# a plain value at the end of a path owns nothing, so it takes no verb
rec In:
    var k: i32;
;rec In
rec Out:
    var inn: In;
;rec Out
MAIN:
    let $o := Out[inn = In[k = 1]];
    let $n := copy $o.inn.k;
;MAIN
