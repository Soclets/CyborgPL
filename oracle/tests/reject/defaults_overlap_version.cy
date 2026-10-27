coll.fn say:
    labels [$t as value, $k as count];
    defaults [$k = 1];
    versions say:
        [let $n: int.sign.32, let $t: int.sign.32, let $k: int.sign.32] -> int.sign.32:
            ret 1;
        end of version
        [let $n: int.sign.32, let $t: int.sign.32] -> int.sign.32:
            ret 2;
        end of version
    end of versions say
end of coll.fn say
!MAIN:
!end of MAIN
