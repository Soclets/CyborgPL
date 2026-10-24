incl.C.system["stdio.h", "stdlib.h"];
MAIN {
    no_check let $p := malloc(16);
    no_check fclose($p);
}
