fn keep <T: any> [let $v: <T>] -> <T>:
    var $x = $v;
    ret $x;
;fn keep

MAIN:
    std::println("ok");
;MAIN
