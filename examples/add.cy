# A function that adds two numbers, called from MAIN.
fn add [$a: int.sign.32, $b: int.sign.32] -> int.sign.32:
    ret $a plus $b;
end of fn add

!MAIN:
    let $sum := add(10, 20);
    std::print.shortest("sum: ", $sum, newln);
!end of MAIN
