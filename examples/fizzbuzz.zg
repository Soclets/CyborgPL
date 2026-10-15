# FizzBuzz.
fn say ['n': int32] {
    let 'by3' := ('n' mod.floor 3) equal_to 0;
    let 'by5' := ('n' mod.floor 5) equal_to 0;
    if 'by3' and 'by5' {
        std::print.stored["FizzBuzz", newln];
    } else if 'by3' {
        std::print.stored["Fizz", newln];
    } else if 'by5' {
        std::print.stored["Buzz", newln];
    } else {
        std::print.stored['n', newln];
    }
}

MAIN {
    let immut 'limit': int32 = 15;
    let 'i' := 1;
    while 'i' at_most 'limit' {
        say['i'];
        change 'i' = 'i' plus 1;
    }
}
