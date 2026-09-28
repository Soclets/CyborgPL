import { parts } from "../lib/md.js";

// Site copy with `code` shown as code.
export default function Md({ text }) {
  return parts(text).map((p, i) => (typeof p === "string" ? p : <code key={i} className="ic">{p.code}</code>));
}
