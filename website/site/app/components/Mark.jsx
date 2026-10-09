import { iconSVG, wordmarkSVG } from "../lib/wordmark.js";

// The Nerve icon beside the NERVE wordmark, in the current text colour (see lib/wordmark.js).
export default function Mark({ height = 18 }) {
  return <span className="wordmark" dangerouslySetInnerHTML={{ __html: iconSVG(Math.round(height * 1.5)) + wordmarkSVG("wordmark-top", height) }} />;
}
