# Each limb's words work once the kit lists the limb (owner, 2026-10-05/06): here MCPS's `%spawn.task`.
# `to.`, `check`, `no_check` and generics were ASP's, IEG's and Genny's then and are the base's now; std
# for printing.

fn first <T: any> [let $a: <T>, let $b: <T>] -> <T>:
    ret $a;
;fn first

proc hello []:
    std::print("from a task", newln);
;proc hello

MAIN:
    var $n: i64 = to.i64("41") ?? 0;
    check change $n to $n + 1;
    no_check change $n to $n x 1;
    std::print(first($n, 7), newln);
    %spawn.task hello();
;MAIN
