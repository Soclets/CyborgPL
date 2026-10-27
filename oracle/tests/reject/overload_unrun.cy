coll.fn ov:
    versions ov.flip:
        [$x: int.sign.32] -> int.sign.32:
            ret 1;
        end of version
        [$x: int.sign.64] -> int.sign.32:
            ret 2;
        end of version
    end of versions ov
end of coll.fn ov
!MAIN:
    let $p := false;
    std::print.shortest("before", newln);
    if $p:
        std::print.shortest(ov.flip(77), newln);
    end of if
!end of MAIN
