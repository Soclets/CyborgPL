# A kit whose writer made its names case-insensitive, in its own code and as reached from other
# kits (owner, 2026-10-06: "`a`, and `c`. Would depend on their respective writers"), kind by kind.
case.any [fn, struct, field, const];

pub const $Volume := 11;

pub fn Shout [$s: str.utf8] -> str.utf8 {
    return $s concat "!";
}

pub fn Whisper [] -> str.utf8 {
    return "psst, " concat shout("volume " concat to.useful.str.utf8(my.$VOLUME));
}

pub struct Level {
    Loud: bool
}
