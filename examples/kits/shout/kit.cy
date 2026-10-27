# A kit whose writer made its names case-insensitive, in its own code and as reached from other
# kits (owner, 2026-10-06: "`a`, and `c`. Would depend on their respective writers"), kind by kind.
case.any [function, record, field, constant];

public constant $Volume := 11;

public function Shout [$s: string.utf8] -> string.utf8:
    return $s concatenate "!";
end of function Shout

public function Whisper [] -> string.utf8:
    return "psst, " concatenate shout("volume " concatenate to.shortest.string.utf8(my.$VOLUME));
end of function Whisper

public record Level:
    Loud: boolean;
end of record Level
