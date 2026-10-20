# Names other kits may write with or without `greet::`, or only without it (owner, 2026-10-06).

pub bare.kit_prefix.optional fn hello [] -> str.utf8 {
    return "hello";
}

pub bare.kit_prefix.only fn wave [] -> str.utf8 {
    return "wave";
}

pub bare.kit_prefix.never fn formal [] -> str.utf8 {
    return "good day";
}
