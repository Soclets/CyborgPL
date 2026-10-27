interface Ordered <T>:
    op less_than [$a: <T>, $b: <T>] -> bool:
        ret true;
    end of op less_than;
end of interface Ordered
!MAIN:
!end of MAIN
