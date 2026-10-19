import { Md } from "../components/md";
import ABOUT from "../generated/about.json";
import type { Route } from "./+types/legal";

export const meta: Route.MetaFunction = () => [
  { title: "Legal · CyborgPL" },
  { name: "description", content: "CyborgPL's licences, its names and logo, and what the site keeps about you." },
];

// Legal (owner, 2026-10-19: "Change License to Legal, and include 1 & 2 in there": the licences, the
// trademarks and logo, and privacy). The licences, the warning and the trademark text are the
// README's own words, read when the site is built.
const LOGOS = [
  { file: "logo.svg", label: "Logo", note: "The mark in its black square" },
  { file: "mark-white.svg", label: "Mark, white", note: "For dark backgrounds" },
  { file: "mark-black.svg", label: "Mark, black", note: "For light backgrounds" },
  { file: "avatar.svg", label: "Avatar", note: "Square, for profiles" },
  { file: "logo-512.png", label: "Logo, 512 px", note: "PNG" },
  { file: "avatar-1024.png", label: "Avatar, 1024 px", note: "PNG" },
];

export default function Legal() {
  return (
    <main className="doc">
      <h1>Legal</h1>
      <section className="doc-warning">{ABOUT.warning.map((p, i) => <p key={i}><Md text={p} /></p>)}</section>

      <section>
        <h2>Licences</h2>
        <ul>{ABOUT.licences.map((l, i) => <li key={i}><Md text={l} /></li>)}</ul>
        <ul className="doc-credit-lines">{ABOUT.creditLines.map((l, i) => <li key={i}><Md text={l} /></li>)}</ul>
        <p><Md text={ABOUT.asked} /></p>
      </section>

      <section>
        <h2>Names and logo</h2>
        <p><Md text={ABOUT.trademark} /></p>
        <div className="doc-logos">
          {LOGOS.map((l) => (
            <a key={l.file} className="doc-logo" href={`/brand/${l.file}`} download>
              <span className={`doc-logo-art${l.file.includes("black") ? " on-light" : ""}`}><img src={`/brand/${l.file}`} alt="" loading="lazy" /></span>
              <span className="doc-logo-label">{l.label}</span>
              <span className="doc-logo-note">{l.note}</span>
            </a>
          ))}
        </div>
        <p className="doc-small">The logos are under CC BY 4.0, credited as above.</p>
      </section>

      <section>
        <h2>Privacy</h2>
        <p>This site sets no cookies and stores nothing on your device. It counts visits with Cloudflare Web Analytics, which uses no cookies and no fingerprinting, and keeps no personal data: just the pages visited, the site a visit came from, and the rough browser, device and country.</p>
        <p>Cloudflare serves the site, so, like any web server, it sees each request's address to answer it. What Cloudflare does with that is in <a href="https://www.cloudflare.com/privacypolicy/" target="_blank" rel="noopener noreferrer">Cloudflare's privacy policy</a>.</p>
      </section>

      <p className="doc-small">{ABOUT.copyright}</p>
    </main>
  );
}
