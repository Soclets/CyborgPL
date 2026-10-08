import { Link } from "react-router";
import Icon from "../components/Icon.jsx";
import LanguageLogo from "../components/LanguageLogo.jsx";
import { LANGUAGE_ARTWORK } from "../lib/language-artwork.js";
import { REPO_URL, RRX2_URL } from "../lib/content.js";
import { seo } from "../lib/seo.js";

export const meta = () => seo({ title: "More · Nerve", description: "Download Nerve, read its source and its licence, and find RRx2, its verifiers.", path: "/more" });

export default function More() {
  return (
    <div className="more">
      <h1 className="h2big">More</h1>
      <div className="list">
        <Link to="/download" className="row"><span className="ri"><Icon name="download" size={19} /></span><b>Download</b><Icon name="chev" size={18} /></Link>
        <a href={REPO_URL} target="_blank" rel="noopener noreferrer" className="row"><span className="ri"><Icon name="github" size={19} stroke={1.8} /></span><b>Source</b><Icon name="external" size={17} /></a>
        <Link to="/good-practice" className="row"><span className="ri"><Icon name="check" size={19} stroke={2.4} /></span><b>Good practice</b><Icon name="chev" size={18} /></Link>
        <Link to="/licence" className="row"><span className="ri"><Icon name="licence" size={19} /></span><b>Licence</b><Icon name="chev" size={18} /></Link>
      </div>
      <h2 className="section">Related</h2>
      <div className="list">
        <a href={RRX2_URL} target="_blank" rel="noopener noreferrer" className="row">
          <span className="ri ri-logo"><img src="/icons/rrx2.svg" alt="" width="30" height="15" /></span>
          <span className="row-text"><b>RRx2</b><small>Nerve’s own verifiers: RRx2, its translation validator, and RR, an exact-math language</small></span>
          <Icon name="external" size={17} />
        </a>
      </div>
      <section className="logo-credits" id="logo-credits" aria-labelledby="logo-credits-title">
        <h2 className="section" id="logo-credits-title">Logo credits</h2>
        <div className="logo-credit-list">
          {Object.entries(LANGUAGE_ARTWORK).map(([id, a]) => (
            <article key={id} className="logo-credit">
              <LanguageLogo id={id} />
              <div><h3>{a.title}</h3><p>{a.credit}</p>
                <div className="logo-credit-links">{a.links.map((l) => <a key={l.href} href={l.href} target="_blank" rel="noopener noreferrer">{l.label}<Icon name="external" size={11} /></a>)}</div>
              </div>
            </article>
          ))}
        </div>
        <p className="fineprint">The artwork identifies the languages in the comparison. Third-party marks belong to their owners; their display does not imply endorsement of Nerve.</p>
      </section>
      <p className="fineprint">Nerve is in development. Add this site to your home screen and it opens like an app.</p>
    </div>
  );
}
