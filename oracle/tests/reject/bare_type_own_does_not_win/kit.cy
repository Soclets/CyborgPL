bare.kit_prefix.never [struct];

pub struct Note {
    text: str.utf8
}

MAIN {
    let 'n' := Note[text = "x"];
}
