# ptr.read of a C enum from C can fail out_of_range (2026-09-28)
layout.C enum Mode { emacs = 0 }
extern.C fn malloc ['bytes': uint64] -> ptr.mut[Mode];
fn f ['p': ptr[Mode]] -> int32 {
    no_check let 'm' := ptr.read['p'];
    return 0;
}
MAIN {
    no_check let 'mem' := malloc[4];
    no_check let 'x' := f['mem'];
}
