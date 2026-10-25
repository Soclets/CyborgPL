# giving a part out of what holds it would leave a hole
struct In { name: str.utf8 }
struct Out { inn: In }
MAIN {
    let $o := Out[inn = In[name = "a"]];
    let $n := give $o.inn.name;
}
