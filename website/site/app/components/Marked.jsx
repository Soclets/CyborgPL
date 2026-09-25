import { marks } from "../lib/search.js";

// Text with the query's words picked out.
export default function Marked({ text, q }) {
  return marks(text, q).map((part, i) => (i % 2 ? <mark key={i}>{part}</mark> : part));
}
