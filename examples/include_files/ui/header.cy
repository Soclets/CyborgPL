# A module one folder down: its path still starts at the kit's root, not at `ui/`.
constant $greeting_text := include.string.utf8["assets/greeting.txt"];

public.kit function greeting [] -> string.utf8:
    return $greeting_text;
end of function greeting
