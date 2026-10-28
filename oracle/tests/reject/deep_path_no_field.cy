# a deep path names fields its struct has
rec In:
    var k: i32;
;rec In
rec Out:
    var inn: In;
;rec Out
MAIN:
    let $o := Out[inn = In[k = 1]];
    std::println($o.inn.j);
;MAIN
