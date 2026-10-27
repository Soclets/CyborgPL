interface Ordered <T>:
    operator less_than [$a: <T>, $b: <T>] -> boolean:
        return true;
    end of operator <;
end of interface Ordered
!MAIN:
!end of MAIN
