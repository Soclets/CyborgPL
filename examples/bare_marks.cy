# How other kits write the path to each `pub` thing (owner, 2026-10-06): every one is covered by a
# mark on itself, its module or its kind, and the marks that cover it agree. There is no default. A `pub` enum
# also says whether its values may be written without its name, `bare.enum_prefix.…` (owner, 2026-10-20).

bare.kit_prefix.never [fn, struct];                 # every pub fn and struct in this kit: the path is required

pub const 'unit' := 1;                    # a const takes no mark: its path is always written

pub bare.kit_prefix.never module shapes {            # everything in this block module
    pub fn area ['side': i32] -> i32 {
        return 'side' x 'side';
    }

    pub struct Square {
        side: i32
    }
}

pub bare.kit_prefix.only bare.enum_prefix.optional enum Mood {   # its name bare only, never with the path;
    calm,                                                          # its values may drop `Mood.` (2026-10-20)
    loud
}

fn say ['m': Mood] -> str.utf8 {
    match 'm' { calm { return "calm"; } loud { return "LOUD"; } }
}

MAIN {
    std::print.useful[shapes.area[3], " ", my.'unit', newln];
    std::print.useful[say[calm], " ", say[Mood.loud], newln];   # the parameter says Mood, so `calm` is enough
}
