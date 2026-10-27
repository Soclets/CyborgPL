coll.fn say:
    labels [$t as value, $k as count];
    defaults [$k = 1];
    versions say:
        [$n: int.sign.32, $t: int.sign.32, $k: int.sign.32] -> int.sign.32:
            ret 1;
        end of version
        [$n: int.sign.32, $t: int.sign.32] -> int.sign.32:
            ret 2;
        end of version
    end of versions say
end of coll.fn say
!MAIN:
!end of MAIN
