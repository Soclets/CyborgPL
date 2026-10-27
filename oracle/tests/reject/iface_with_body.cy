interface Ordered <T>:
    operator less_than [$a: <T>, $b: <T>] -> boolean:
        return true;
    end of operator less_than;
end of interface Ordered
!MAIN:
!end of MAIN
