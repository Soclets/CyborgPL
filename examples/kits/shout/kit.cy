# A kit whose writer made its names case-insensitive, in its own code and as reached from other
# kits (owner, 2026-10-06: "`a`, and `c`. Would depend on their respective writers"), kind by kind.
case.any [fn, rec, field, const];

pub const $Volume := 11;

pub fn Shout [let $s: str.utf8] -> str.utf8:
    ret $s concat "!";
end of fn Shout

pub fn Whisper [] -> str.utf8:
    ret "psst, " concat shout("volume " concat to.shortest.str.utf8(my.$VOLUME));
end of fn Whisper

pub rec Level:
    var Loud: bool;
end of rec Level
