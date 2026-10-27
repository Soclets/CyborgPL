fn g [$p: int.unsign.8] -> int.unsign.8:
    if false:
        let shadow $p: bool = true;
        ret $p;
    end of if
    ret $p;
end of fn g
!MAIN:
    let $x := g(1);
!end of MAIN
