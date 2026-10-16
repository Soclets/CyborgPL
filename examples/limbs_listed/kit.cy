# Each limb's words work once the kit lists the limb (owner, 2026-10-05/06): here MCPS's `concur`.
# `to.`, `check`, `no_check` and generics were ASP's, IEG's and Genny's then and are the base's now; std
# for printing.

fn first <T: any> ['a': <T>, 'b': <T>] -> <T> {
    return 'a';
}

fn hello [] {
    std::print.useful["from a task", newln];
}

MAIN {
    let 'n': int64 = to.int64["41"] otherwise 0;
    check change 'n' = 'n' + 1;
    no_check change 'n' = 'n' x 1;
    std::print.useful[first['n', 7], newln];
    concur hello[];
}
