// A sliver of Markdown for site copy: `code` only. plain() drops the backticks, for search
// and anything else that wants the words.
export const plain = (text) => text.replace(/`([^`]+)`/g, "$1");
export const parts = (text) => text.split(/(`[^`]+`)/).map((p) => (p.length > 1 && p[0] === "`" && p.at(-1) === "`" ? { code: p.slice(1, -1) } : p));
