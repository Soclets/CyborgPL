# The digits of pi, by the Rabinowitz–Wagon spigot: integers only, no bin anywhere.
fn pi_digits ['n': i32] {
    let 'len' := (10 x 'n') div.floor 3 + 1;
    let mut 'a' := list.empty;
    for 'i' from 1 to 'len' by 1 { list.append[lend.mut 'a', value 2]; }
    let mut 'nines' := 0;
    let mut 'predigit' := 0;
    for 'j' from 1 to 'n' by 1 {
        let mut 'q' := 0;
        for 'i' from 'len' to 1 by -1 {
            let 'x' := (10 x 'a'[pos 'i']) + ('q' x 'i');
            let 'den' := (2 x 'i') - 1;
            change 'a'[pos 'i'] = 'x' mod.floor 'den';
            change 'q' = 'x' div.floor 'den';
        }
        change 'a'[pos 1] = 'q' mod.floor 10;
        change 'q' = 'q' div.floor 10;
        if 'q' equal_to 9 {
            change 'nines' = 'nines' + 1;
        } else if 'q' equal_to 10 {
            std::print.useful['predigit' + 1];
            for 'k' from 1 to 'nines' by 1 { std::print.useful[0]; }
            change 'predigit' = 0;
            change 'nines' = 0;
        } else {
            if 'j' > 1 { std::print.useful['predigit']; }
            if 'j' equal_to 2 { std::print.useful["."]; }
            change 'predigit' = 'q';
            for 'k' from 1 to 'nines' by 1 { std::print.useful[9]; }
            change 'nines' = 0;
        }
    }
    std::print.useful['predigit', newln];
}

MAIN { pi_digits[60]; }
