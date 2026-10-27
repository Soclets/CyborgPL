# to.Key can fail out_of_range (2026-09-27)
%layout.C enum Key:
    a = 65;
end of enum Key
!MAIN:
    let $k := to.Key(65);
!end of MAIN
