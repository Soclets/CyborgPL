# Each limb's words work once the kit lists the limb (owner, 2026-10-05/06): here MCPS's `concur`.
# `to.`, `check`, `no_check` and generics were ASP's, IEG's and Genny's then and are the base's now; std
# for printing.

fn first <T: any> [let $a: <T>, let $b: <T>] -> <T>:
    ret $a;
;fn first

proc hello []:
    std::println("from a task");
;proc hello

MAIN:
    var $n: i64 = to.i64("41") ?? 0;
    check change $n += 1;
    no_check change $n x= 1;
    std::println(first($n, 7));
    %spawn.task hello();
;MAIN
