# a deep path names fields its struct has
rec In:
    var k: int.sign.32;
end of rec In
rec Out:
    var inn: In;
end of rec Out
!MAIN:
    let $o := Out[inn = In[k = 1]];
    std::print.shortest($o.inn.j, newln);
!end of MAIN
