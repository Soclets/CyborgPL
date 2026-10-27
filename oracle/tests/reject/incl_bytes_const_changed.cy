!MAIN:
    constant $b := include.bytes["incl_assets/cafe.txt"];
    change $b[position: 1] = 0;
!end of MAIN
