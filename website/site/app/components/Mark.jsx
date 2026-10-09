import { wordmarkSVG } from "../lib/wordmark.js";

// The Nerve logo: the wordmark with its cross (see lib/wordmark.js).
export default function Mark({ height = 18 }) {
  return <span className="wordmark logo" dangerouslySetInnerHTML={{ __html: wordmarkSVG("wordmark-top", height) }} />;
}
