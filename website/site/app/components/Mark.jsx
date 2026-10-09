import { iconSVG, wordmarkSVG } from "../lib/wordmark.js";

// The Nerve icon on its tile beside the NERVE wordmark (see lib/wordmark.js).
export default function Mark({ height = 18 }) {
  return <span className="wordmark" dangerouslySetInnerHTML={{ __html: iconSVG(Math.round(height * 1.65)) + wordmarkSVG("wordmark-top", height) }} />;
}
