// The code colours: a deep-blue panel, keywords in the logo's yellow, names in sky blue.
export const ZAG_THEME = {
  name: "zag-night",
  type: "dark",
  colors: { "editor.background": "#0D2244", "editor.foreground": "#E8F0FC" },
  tokenColors: [
    { scope: ["comment", "punctuation.definition.comment"], settings: { foreground: "#8EA3C7", fontStyle: "italic" } },
    { scope: ["keyword", "storage", "entity.name.function.main"], settings: { foreground: "#FFD43B" } },
    { scope: ["keyword.other.range.label", "keyword.other.position"], settings: { foreground: "#FFE9A3", fontStyle: "italic" } },
    { scope: ["keyword.operator"], settings: { foreground: "#F3A8FF" } },
    { scope: ["keyword.operator.assignment", "keyword.operator.arrow"], settings: { foreground: "#C9B8FF" } },
    { scope: ["variable.other", "variable.language"], settings: { foreground: "#8FD3FF" } },
    { scope: ["string", "punctuation.definition.string"], settings: { foreground: "#A8F0B4" } },
    { scope: ["constant.numeric"], settings: { foreground: "#FFB38A" } },
    { scope: ["constant.language", "constant.character", "constant.other"], settings: { foreground: "#FF9FB2" } },
    { scope: ["entity.name.function"], settings: { foreground: "#FFFFFF" } },
    { scope: ["entity.name.type", "support.type", "entity.name.tag"], settings: { foreground: "#7DE3D3" } },
    { scope: ["entity.name.namespace", "variable.other.member", "variable.other.enummember"], settings: { foreground: "#C3D3F0" } },
    { scope: ["punctuation"], settings: { foreground: "#9FB4D8" } },
    { scope: ["invalid.deprecated"], settings: { foreground: "#FF6B6B", fontStyle: "strikethrough" } },
  ],
};
