import { wordmarkSVG } from "../lib/wordmark.js";

// The NERVE wordmark in the current text colour (see lib/wordmark.js).
export default function Mark({ height = 18 }) {
  return <span className="wordmark" dangerouslySetInnerHTML={{ __html: wordmarkSVG("wordmark-top", height) }} />;
}
