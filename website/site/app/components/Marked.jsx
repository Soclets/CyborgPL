import { parts } from "../lib/md.js";
import { marks } from "../lib/search.js";

const hi = (text, q, key) => marks(text, q).map((part, i) => (i % 2 ? <mark key={`${key}.${i}`}>{part}</mark> : part));

// Text with the query's words picked out, and any `code` in it shown as code.
export default function Marked({ text, q }) {
  return parts(text).map((p, i) => (typeof p === "string" ? hi(p, q, i) : <code key={i} className="ic">{hi(p.code, q, i)}</code>));
}
