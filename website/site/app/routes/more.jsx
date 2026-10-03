import { Link } from "react-router";
import Icon from "../components/Icon.jsx";
import { REPO_URL, RRX2_URL } from "../lib/content.js";
import { seo } from "../lib/seo.js";

export const meta = () => seo({ title: "More · CyborgPL", description: "Download CyborgPL, read its source and its licence, and find RRx2, its verifiers.", path: "/more" });

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
          <span className="row-text"><b>RRx2</b><small>CyborgPL’s own verifiers: RRx2, its translation validator, and RR, an exact-math language</small></span>
          <Icon name="external" size={17} />
        </a>
      </div>
      <p className="fineprint">CyborgPL is in development. Add this site to your home screen and it opens like an app.</p>
    </div>
  );
}
