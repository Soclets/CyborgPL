%extern.C proc malloc [$bytes: u64] -> ptr.mut[u8];
MAIN:
    no_check let $p := malloc(4);
    no_check change *$p to 65;
;MAIN
