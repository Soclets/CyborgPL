# to.Key can fail out_of_range (2026-09-27)
%layout.C enumeration Key:
    a = 65;
end of enumeration Key
!MAIN:
    let $k := to.Key(65);
!end of MAIN
