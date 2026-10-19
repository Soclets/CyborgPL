import type { ReactNode } from "react";

// The little Markdown the README's wording uses: `code`, **bold**, *italic* and [links](…). A link
// into the repo (LICENSE, DECISIONS.md, brand/…) goes to it on GitHub; any link off the site opens
// in a new tab.
const REPO = "https://github.com/Soclets/CyborgPL";
const href = (to: string) => (/^https?:/.test(to) ? to : `${REPO}/${to.endsWith("/") ? "tree" : "blob"}/main/${to}`);
const TOKEN = /(`[^`]+`)|(\*\*[^*]+\*\*)|(\*[^*]+\*)|(\[[^\]]+\]\([^)]+\))/g;

export function Md({ text }: { text: string }): ReactNode {
  const out: ReactNode[] = [];
  let last = 0, k = 0;
  for (const m of text.matchAll(TOKEN)) {
    if (m.index! > last) out.push(text.slice(last, m.index));
    const t = m[0];
    if (m[1]) out.push(<code key={k++}>{t.slice(1, -1)}</code>);
    else if (m[2]) out.push(<strong key={k++}><Md text={t.slice(2, -2)} /></strong>);
    else if (m[3]) out.push(<em key={k++}><Md text={t.slice(1, -1)} /></em>);
    else {
      const [, label, to] = t.match(/^\[([^\]]+)\]\(([^)]+)\)$/)!;
      out.push(<a key={k++} href={href(to!)} target="_blank" rel="noopener noreferrer"><Md text={label!} /></a>);
    }
    last = m.index! + t.length;
  }
  if (last < text.length) out.push(text.slice(last));
  return <>{out}</>;
}
