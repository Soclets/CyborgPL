bare.kit_prefix.never [fn];

pub fn hello [] -> str.utf8 {
    return "mine";
}

MAIN {
    print.useful[hello[], newln];
}
