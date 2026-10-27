fn half [let $n: int.sign.32] -> int.sign.32 or fail[odd, negative: int.sign.32]:
    if $n lt 0:
        fail negative[$n];
    end of if
    ret $n;
end of fn half
coll.handler h:
    member h.zero:
        !case odd:
            ret 0;
        !end of case odd
    end of member h.zero
end of coll.handler h
fn f [] -> int.sign.32:
    let $v := half(3) otherwise h.!zero;
    ret $v;
end of fn f
!MAIN:
    std::print.shortest(f(), newln);
!end of MAIN
