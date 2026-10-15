// `code` in site copy becomes <code>; the rest stays text.
export default function Md({ text }) {
  return text.split(/(`[^`]+`)/).map((part, i) =>
    part.startsWith("`") && part.endsWith("`") ? <code key={i}>{part.slice(1, -1)}</code> : part);
}
