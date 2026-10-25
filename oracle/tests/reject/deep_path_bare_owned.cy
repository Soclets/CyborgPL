# an owned value at the end of a deep path takes a verb, as one step does
struct In { name: str.utf8 }
struct Out { inn: In }
MAIN {
    let $o := Out[inn = In[name = "a"]];
    let $n := $o.inn.name;
}
