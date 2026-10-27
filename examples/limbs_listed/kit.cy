# Each limb's words work once the kit lists the limb (owner, 2026-10-05/06): here MCPS's `concur`.
# `to.`, `check`, `no_check` and generics were ASP's, IEG's and Genny's then and are the base's now; std
# for printing.

fn first <T: any> [let $a: <T>, let $b: <T>] -> <T>:
    ret $a;
end of fn first

proc hello []:
    std::print.shortest("from a task", newln);
end of proc hello

!MAIN:
    var $n: int.sign.64 := to.int.sign.64("41") otherwise 0;
    check change $n to $n plus 1;
    !no_check change $n to $n times 1;
    std::print.shortest(first($n, 7), newln);
    %spawn.task hello();
!end of MAIN
