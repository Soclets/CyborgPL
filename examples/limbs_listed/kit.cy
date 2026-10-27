# Each limb's words work once the kit lists the limb (owner, 2026-10-05/06): here MCPS's `concur`.
# `to.`, `check`, `no_check` and generics were ASP's, IEG's and Genny's then and are the base's now; std
# for printing.

function first <T: any> [$a: <T>, $b: <T>] -> <T>:
    return $a;
end of function first

procedure hello []:
    std::print.shortest("from a task", newline);
end of procedure hello

!MAIN:
    variable $n: integer.signed.64_bit = to.integer.signed.64_bit("41") otherwise 0;
    check change $n = $n plus 1;
    !no_check change $n = $n times 1;
    std::print.shortest(first($n, 7), newline);
    %spawn.task hello();
!end of MAIN
