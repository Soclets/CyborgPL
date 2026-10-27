!MAIN:
    let $b: bool := false;
    if $b:
        let $r := to.int.sign.64(5 shr to.int.unsign.8(1));
    end of if
!end of MAIN
