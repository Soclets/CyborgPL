fn half [$n: int.sign.32] -> int.sign.32 or fail[odd, negative: int.sign.32]:
    if $n less_than 0:
        fail negative[$n];
    end of if
    ret $n;
end of fn half
coll.handler h:
    member h.zero:
        !case odd:
            ret 0;
        !end of case odd
        !case negative $v:
            ret 0;
        !end of case negative
    end of member h.zero
end of coll.handler h
fn f [] -> int.sign.32:
    let $v := half(3) otherwise h.nope;
    ret $v;
end of fn f
!MAIN:
!end of MAIN
