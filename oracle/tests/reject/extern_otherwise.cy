extern.C fn f [$a: i32] -> i32;
MAIN {
    no_check let $x := f(1) otherwise 0;
}
