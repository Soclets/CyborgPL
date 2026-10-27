coll.fn f:
    versions f:
        [let $a: int.sign.64] -> int.sign.64:
            ret 1;
        end of version
        [let $b: int.sign.32] -> int.sign.64:
            ret 2;
        end of version
    end of versions f
end of coll.fn f
!MAIN:
    std::print.shortest(f(1), newln);
!end of MAIN
