import { useState } from "react";
import { Link } from "react-router";
import Icon from "./Icon.jsx";
import LanguagePicker from "./LanguagePicker.jsx";
import LanguageLogo from "./LanguageLogo.jsx";
import Md from "./Md.jsx";
import { COMPARISON_LANGUAGES, COMPARISON_ROWS } from "../lib/comparison.js";

// Keep the choice when Home returns from a search or another page.
let lastPair = ["nerve", "rust"];

function LanguageHeading({ language }) {
  return <span className="comparison-heading"><LanguageLogo id={language.id} /><span><b>{language.name}</b><small>{language.version ? <a href={language.versionUrl} target="_blank" rel="noopener noreferrer">{language.version}</a> : language.status}</small></span></span>;
}

function Fact({ entry, language, topic }) {
  return <>
    <p><Md text={entry.text} /></p>
    <span className="comparison-links">
      {entry.sources.map((s) => <a key={s.url} href={s.url} target="_blank" rel="noopener noreferrer" aria-label={`${language.name}: ${topic} — ${s.label}`}>{s.label}<Icon name="external" size={11} /></a>)}
    </span>
  </>;
}

export default function Comparison() {
  const [pair, setPair] = useState(lastPair);
  const choose = (next) => { lastPair = next; setPair(next); };
  const [left, right] = pair.map((id) => COMPARISON_LANGUAGES.find((l) => l.id === id));
  return (
    <section className="comparison" id="comparison" aria-labelledby="comparison-title">
      <h2 className="section" id="comparison-title">Compare languages</h2>
      <p className="comparison-intro">Choose two languages. Compare their behavior, tooling and costs; performance depends on the workload.</p>
      <div className="comparison-controls">
        <LanguagePicker side="Left" value={left.id} other={right.id} onChange={(id) => choose([id, right.id])} />
        <button type="button" className="comparison-swap" aria-label="Swap comparison languages" title="Swap sides" onClick={() => choose([right.id, left.id])}><span aria-hidden="true">⇄</span></button>
        <LanguagePicker side="Right" value={right.id} other={left.id} onChange={(id) => choose([left.id, id])} />
      </div>
      <p className="comparison-note">Checked 3 Oct 2026. Nerve reflects current development and its stated design. Python means CPython, with free-threaded differences called out. <Link to="/more#logo-credits">Logo credits</Link>.</p>
      <table className="comparison-table" role="table">
        <caption className="sr-only">{left.name} compared with {right.name}</caption>
        <colgroup><col style={{ width: "20%" }} /><col style={{ width: "40%" }} /><col style={{ width: "40%" }} /></colgroup>
        <thead role="rowgroup"><tr role="row">
          <th scope="col" role="columnheader">Topic</th>
          <th scope="col" role="columnheader"><LanguageHeading language={left} /></th>
          <th scope="col" role="columnheader"><LanguageHeading language={right} /></th>
        </tr></thead>
        <tbody role="rowgroup">
          {COMPARISON_ROWS.map((r) => <tr key={r.id} role="row">
            <th scope="row" role="rowheader">{r.title}</th>
            <td role="cell"><Fact entry={r.cells[left.id]} language={left} topic={r.title} /></td>
            <td role="cell"><Fact entry={r.cells[right.id]} language={right} topic={r.title} /></td>
          </tr>)}
        </tbody>
      </table>
    </section>
  );
}
