# a plain value at the end of a path owns nothing, so it takes no verb
struct In { k: i32 }
struct Out { inn: In }
MAIN {
    let $o := Out[inn = In[k = 1]];
    let $n := copy $o.inn.k;
}
