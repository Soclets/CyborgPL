// Solarized Light for code, with the standard Solarized roles.
export const solarizedLight = {
  name: "solarized-light-cyborgpl",
  type: "light",
  colors: { "editor.background": "#00000000", "editor.foreground": "#657b83" },
  settings: [
    { settings: { foreground: "#657b83" } },
    { scope: ["comment"], settings: { foreground: "#93a1a1", fontStyle: "italic" } },
    { scope: ["string"], settings: { foreground: "#2aa198" } },
    { scope: ["constant.character.escape", "constant.character.newline"], settings: { foreground: "#dc322f" } },
    { scope: ["constant.numeric", "keyword.other.unit", "constant.language"], settings: { foreground: "#d33682" } },
    { scope: ["keyword.control", "keyword.declaration", "keyword.other", "storage.modifier", "keyword.operator.assignment.infer", "keyword.operator.word"], settings: { foreground: "#859900" } },
    { scope: ["keyword.operator"], settings: { foreground: "#657b83" } },
    { scope: ["support.type", "entity.name.type"], settings: { foreground: "#b58900" } },
    { scope: ["entity.name.function", "entity.name.namespace"], settings: { foreground: "#268bd2" } },
    { scope: ["variable.other.enummember", "entity.name.tag"], settings: { foreground: "#cb4b16" } },
    { scope: ["variable.other"], settings: { foreground: "#6c71c4" } },
    { scope: ["variable.other.member", "variable.other.member.declaration"], settings: { foreground: "#586e75" } },
    { scope: ["punctuation"], settings: { foreground: "#93a1a1" } },
  ],
};

// Solarized Dark: the same roles, with the base tones mirrored (Solarized's own rule:
// base0 for body text, base01 for comments and punctuation, base1 for emphasis).
export const solarizedDark = {
  name: "solarized-dark-cyborgpl",
  type: "dark",
  colors: { "editor.background": "#00000000", "editor.foreground": "#839496" },
  settings: solarizedLight.settings.map((s) => {
    const swap = { "#657b83": "#839496", "#93a1a1": "#586e75", "#586e75": "#93a1a1" };
    const fg = s.settings.foreground;
    return { ...s, settings: { ...s.settings, ...(fg ? { foreground: swap[fg] ?? fg } : {}) } };
  }),
};
