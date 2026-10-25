# a deep path names fields its struct has
struct In { k: i32 }
struct Out { inn: In }
MAIN {
    let $o := Out[inn = In[k = 1]];
    std::print.useful($o.inn.j, newln);
}
