import { useRef, useState } from "react";
import { Link } from "react-router";
import Icon from "../components/Icon.jsx";
import Logo from "../components/Logo.jsx";
import { ACCENTS, DESCRIPTION, FEATURES, REPO_URL } from "../lib/content.js";

export const meta = () => [
  { title: "CyborgPL" },
  { name: "description", content: DESCRIPTION },
];

export default function Home() {
  const row = useRef(null);
  const [card, setCard] = useState(0);
  // Dots follow the swipeable row: the card nearest the left edge is the current one.
  const onScroll = () => {
    const el = row.current;
    if (!el) return;
    const w = el.firstElementChild?.getBoundingClientRect().width ?? 1;
    setCard(Math.min(FEATURES.length - 1, Math.round(el.scrollLeft / (w + 12))));
  };
  return (
    <div className="home">
      <div className="top">
        <Logo size={36} />
        <span className="badge">In development</span>
      </div>
      <h1 className="huge">Selfless dev<br />meets <span>Selfish</span><br />language.</h1>
      <p className="lede">{DESCRIPTION}</p>
      <div className="actions">
        <Link className="btn primary" to="/download"><Icon name="download" size={18} stroke={2.3} />Download</Link>
        <Link className="btn" to="/learn">Read shit</Link>
        <a className="btn round" href={REPO_URL} target="_blank" rel="noopener noreferrer" aria-label="Source on GitHub"><Icon name="github" size={20} stroke={1.9} /></a>
      </div>
      <h2 className="section">Core features</h2>
      <div className="cards" ref={row} onScroll={onScroll}>
        {FEATURES.map((f, i) => (
          <Link key={f.id} className="fcard" to={`/learn#${f.id}`} style={{ "--c": ACCENTS[i] }}>
            <span className="fi"><Icon name={f.icon} size={22} /></span>
            <b>{f.title}</b>
            <small>{f.text}</small>
          </Link>
        ))}
      </div>
      <div className="dots" aria-hidden="true">
        {FEATURES.map((f, i) => <i key={f.id} className={i === card ? "on" : ""} />)}
      </div>
    </div>
  );
}
