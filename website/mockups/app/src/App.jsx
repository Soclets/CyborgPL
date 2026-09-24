import { useEffect, useMemo, useRef, useState } from 'react';
import data from './data.json';
import Icon from './Icon.jsx';
import { SECTIONS, ALL_ITEMS, findItem } from './nav.js';
import { useWindow } from './useWindow.js';

const routeFromHash = () => {
  const r = decodeURIComponent(location.hash.replace(/^#\/?/, ''));
  return findItem(r) ? r : 'welcome';
};

export default function App() {
  const [route, setRoute] = useState(routeFromHash);
  const [query, setQuery] = useState('');
  const [drawer, setDrawer] = useState(false);
  const searchRef = useRef(null);

  // The hash is the source of truth, so every page has its own link and Back works.
  useEffect(() => {
    const onHash = () => { setRoute(routeFromHash()); setDrawer(false); };
    addEventListener('hashchange', onHash);
    return () => removeEventListener('hashchange', onHash);
  }, []);

  const go = (r) => { location.hash = '/' + r; };

  // ⌘K / Ctrl+K focuses the search field; typing filters the sidebar.
  useEffect(() => {
    const onKey = (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') { e.preventDefault(); searchRef.current?.focus(); searchRef.current?.select(); }
      if (e.key === 'Escape' && document.activeElement === searchRef.current) { setQuery(''); searchRef.current.blur(); }
    };
    addEventListener('keydown', onKey);
    return () => removeEventListener('keydown', onKey);
  }, []);

  const item = findItem(route);
  const win = useWindow();
  const dockIconRef = useRef(null);
  const logoInner = { __html: data.logoInner };
  const logo = <svg viewBox="0 0 32 32" width="22" height="22" aria-hidden="true" dangerouslySetInnerHTML={logoInner} />;

  const f = win.frame;
  const winClass = ['window', win.mode, win.state, win.dragging && 'dragging', !win.active && 'inactive'].filter(Boolean).join(' ');
  const winStyle = { left: f.x, top: f.y, width: f.w, height: f.h,
    '--to-dock-x': win.dockPoint.x + 'px', '--to-dock-y': win.dockPoint.y + 'px' };

  return (
    <div className={'desktop' + (win.mode === 'fullscreen' && win.state === 'open' ? ' is-fullscreen' : '')}
         onPointerDown={(e) => { if (e.target === e.currentTarget) win.setActive(false); }}>
      {win.previewFrame && <div className="snap-preview" style={{ left: win.previewFrame.x, top: win.previewFrame.y, width: win.previewFrame.w, height: win.previewFrame.h }} />}

      <div className={winClass} style={winStyle} onPointerDown={() => win.setActive(true)}
           aria-hidden={win.state !== 'open'} inert={win.state !== 'open'}>
        <header className="titlebar" onPointerDown={win.onTitlePointerDown} onDoubleClick={win.onTitleDoubleClick}>
          <div className="lights">
            <button className="light close" aria-label="Close window" onClick={win.close}><Glyph kind="close" /></button>
            <button className="light min" aria-label="Minimize window" disabled={win.mode === 'fullscreen'} onClick={() => win.minimize(dockIconRef.current)}><Glyph kind="min" /></button>
            <button className="light full" aria-label={win.mode === 'fullscreen' ? 'Exit full screen' : 'Enter full screen'} onClick={win.toggleFullscreen}><Glyph kind={win.mode === 'fullscreen' ? 'unfull' : 'full'} /></button>
          </div>
          <button className="icon-btn drawer-btn" aria-label="Show sidebar" onClick={() => setDrawer((d) => !d)}><Icon name="sidebar" /></button>
          <div className="title">{logo}<span>CyborgPL</span><span className="crumb">— {item.label}</span></div>
          <label className="search">
            <Icon name="search" size={14} />
            <input ref={searchRef} value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Search" aria-label="Search the sidebar" />
            <kbd>⌘K</kbd>
          </label>
          <a className="tb-download" href="#/download">Download</a>
        </header>

        <div className="body">
          <Sidebar route={route} go={go} query={query} open={drawer} />
          {drawer && <div className="scrim" onClick={() => setDrawer(false)} />}
          <main className="content" key={route}>
            <Page route={route} go={go} />
          </main>
        </div>

        {['n', 's', 'e', 'w', 'ne', 'nw', 'se', 'sw'].map((d) => (
          <div key={d} className={'grip grip-' + d} onPointerDown={win.onResizePointerDown(d)} aria-hidden="true" />
        ))}
      </div>

      <nav className="dock" aria-label="Dock">
        <button ref={dockIconRef} className={'dock-app' + (win.state !== 'closed' ? ' running' : '')}
                onClick={win.reopen} aria-label={win.state === 'open' ? 'CyborgPL' : 'Open CyborgPL'}>
          <svg viewBox="0 0 32 32" width="48" height="48" aria-hidden="true" dangerouslySetInnerHTML={logoInner} />
          <span className="dock-tip">CyborgPL</span>
        </button>
      </nav>
    </div>
  );
}

function Glyph({ kind }) {
  const d = { close: 'M3.5 3.5l5 5m0-5-5 5', min: 'M3 6h6', full: 'M3.2 5V3.2H5M8.8 7v1.8H7M3.2 3.2 5.6 5.6M8.8 8.8 6.4 6.4', unfull: 'M5.6 3.2v2.4H3.2M6.4 8.8V6.4h2.4' }[kind];
  return <svg viewBox="0 0 12 12" width="12" height="12" aria-hidden="true"><path d={d} fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round" /></svg>;
}

function Sidebar({ route, go, query, open }) {
  const q = query.trim().toLowerCase();
  const sections = useMemo(() => SECTIONS
    .map((s) => ({ ...s, items: s.items.filter((i) => !q || i.label.toLowerCase().includes(q)) }))
    .filter((s) => s.items.length), [q]);
  const flat = sections.flatMap((s) => s.items);

  // ↑ / ↓ move the selection while the sidebar has focus, like a native source list.
  const onKeyDown = (e) => {
    if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return;
    e.preventDefault();
    // Step from the row that has focus, not from `route`: when keys repeat quickly the
    // next press can arrive before the app has re-rendered with the new route.
    const from = document.activeElement?.dataset?.route ?? route;
    const i = flat.findIndex((x) => x.route === from);
    const next = flat[Math.max(0, Math.min(flat.length - 1, i + (e.key === 'ArrowDown' ? 1 : -1)))];
    if (next) { document.querySelector(`[data-route="${next.route}"]`)?.focus(); go(next.route); }
  };

  return (
    <nav className={'sidebar' + (open ? ' open' : '')} aria-label="Sections" onKeyDown={onKeyDown}>
      {sections.map((s, i) => (
        <div className="group" key={s.title ?? i}>
          {s.title && <div className="group-title">{s.title}</div>}
          {s.items.map((it) => (
            <a key={it.route} data-route={it.route} href={'#/' + it.route}
               className={'row' + (it.route === route ? ' on' : '')} aria-current={it.route === route ? 'page' : undefined}>
              <Icon name={it.icon} /><span>{it.label}</span>
            </a>
          ))}
        </div>
      ))}
      {!flat.length && <div className="empty">Nothing matches “{query}”.</div>}
      <div className="side-foot"><span className="dot" />In development</div>
    </nav>
  );
}

function Page({ route, go }) {
  if (route === 'welcome') return <Welcome go={go} />;
  if (route.startsWith('examples/')) return <Example ex={data.examples.find((e) => 'examples/' + e.id === route)} go={go} />;
  return <NotWritten item={findItem(route)} />;
}

// GitHub mark: the `mark-github` icon from GitHub's Primer Octicons (MIT licence).
const GH = <svg viewBox="0 0 16 16" width="20" height="20" aria-hidden="true"><path fill="currentColor" d="M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 0 0-.67-.22-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.53-1.03-2.2-.82-2.2-.82-.44 1.1-.16 1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67.01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z"/></svg>;

function Welcome({ go }) {
  const tiles = [
    ['learn/getting-started', 'flag', 'Getting started', 'Write, build and run your first program.'],
    ['learn/tour', 'map', 'Language tour', 'Every major idea, one screen each.'],
    ['examples/enums', 'file', 'Examples', 'Real programs, with their real output.'],
    ['lang/precog', 'shield', 'Precog', 'How the checks are proven away.'],
  ];
  return (
    <div className="welcome">
      <div className="pills">
        <span className="pill"><span className="dot" />In development</span>
        <span className="pill">Precog</span>
        <span className="pill">Ownership, no GC</span>
      </div>
      <h1>Selfless dev meets <br /><span className="grad">Selfish language</span><span className="period">.</span></h1>
      <p className="lede">
        CyborgPL is a compiled language where Precog, a whole-program analyser, proves every overflow and bounds check at
        compile time. The final build carries none of them, or it doesn't build. Ownership instead of a garbage collector,
        <code>:=</code> that picks each value's best type from the whole program, failures written into signatures, all through LLVM.
      </p>
      <div className="actions">
        <a className="btn primary" href="#/download">Download <Icon name="download" size={20} /></a>
        <a className="btn ghost" href="#/learn/getting-started">Read shit</a>
        <a className="btn ghost" href="#/source">{GH} Source</a>
      </div>
      <div className="tiles">
        {tiles.map(([r, icon, t, d]) => (
          <a key={r} className="tile" href={'#/' + r}>
            <span className="tile-icon"><Icon name={icon} size={18} /></span>
            <strong>{t}</strong><span>{d}</span>
          </a>
        ))}
      </div>
    </div>
  );
}

function Example({ ex, go }) {
  const i = data.examples.indexOf(ex);
  const prev = data.examples[i - 1], next = data.examples[i + 1];
  return (
    <div className="doc">
      <div className="doc-head">
        <div>
          <div className="eyebrow">Example</div>
          <h2>{ex.title}</h2>
          <div className="meta"><code>{ex.file}</code> · {ex.lines} lines</div>
        </div>
      </div>
      <div className="well code" dangerouslySetInnerHTML={{ __html: ex.html }} />
      <div className="out-label"><span className="dot" />Output, checked by the reference interpreter</div>
      <pre className="well out">{ex.out}</pre>
      <div className="pager">
        {prev ? <a href={'#/examples/' + prev.id}><Icon name="back" size={14} /> {prev.title}</a> : <span />}
        {next && <a href={'#/examples/' + next.id}>{next.title} <Icon name="arrow" size={14} /></a>}
      </div>
    </div>
  );
}

function NotWritten({ item }) {
  return (
    <div className="doc">
      <div className="eyebrow">{item.route.split('/')[0] === 'lang' ? 'Language' : item.route.split('/')[0] === 'learn' ? 'Learn' : 'Project'}</div>
      <h2>{item.label}</h2>
      <div className="placeholder">
        <Icon name={item.icon} size={28} />
        <p>This page hasn't been written yet.</p>
        <span>Its content comes when the owner asks for a site update.</span>
      </div>
    </div>
  );
}
