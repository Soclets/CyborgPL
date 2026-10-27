# A module one folder down: its path still starts at the kit's root, not at `ui/`.
const $greeting_text := incl.str.utf8["assets/greeting.txt"];

pub.kit fn greeting [] -> str.utf8:
    ret $greeting_text;
end of fn greeting
