import { useCallback, useEffect, useRef, useState } from 'react';

// Window geometry for the desktop metaphor: move, snap to halves, zoom, resize,
// full screen, minimize to the dock, close and reopen. Pure state + pointer maths.
const DOCK = 84;          // room kept free for the dock at the bottom
const GAP = 10;           // gap around snapped / zoomed windows
const MIN_W = 560, MIN_H = 400;
const EDGE = 6;           // how close to a screen edge counts as "dragged to the edge"
const KEY = 'cyborgpl.window';

const vw = () => window.innerWidth, vh = () => window.innerHeight;
const area = () => ({ w: vw(), h: vh() - DOCK });

export function defaultRect() {
  const a = area();
  const w = Math.min(1240, a.w - 80), h = Math.min(860, a.h - 56);
  return { x: Math.round((a.w - w) / 2), y: Math.max(20, Math.round((a.h - h) / 2)), w, h };
}

function clampRect(r) {
  const a = area();
  const w = Math.max(MIN_W, Math.min(r.w, a.w)), h = Math.max(MIN_H, Math.min(r.h, a.h));
  return { w, h, x: Math.min(Math.max(r.x, -(w - 140)), a.w - 140), y: Math.min(Math.max(r.y, 0), a.h - 44) };
}

export function geometry(mode, rect) {
  const a = area();
  switch (mode) {
    case 'zoomed': return { x: GAP, y: GAP, w: a.w - 2 * GAP, h: a.h - 2 * GAP };
    case 'left': return { x: GAP, y: GAP, w: Math.round(a.w / 2 - GAP * 1.5), h: a.h - 2 * GAP };
    case 'right': return { x: Math.round(a.w / 2 + GAP / 2), y: GAP, w: Math.round(a.w / 2 - GAP * 1.5), h: a.h - 2 * GAP };
    case 'fullscreen': return { x: 0, y: 0, w: vw(), h: vh() };
    default: return rect;
  }
}

function load() {
  try {
    const s = JSON.parse(localStorage.getItem(KEY));
    if (s && s.rect) return { rect: clampRect(s.rect), mode: s.mode === 'zoomed' || s.mode === 'left' || s.mode === 'right' ? s.mode : 'normal' };
  } catch {}
  return { rect: defaultRect(), mode: 'normal' };
}

export function useWindow() {
  const init = useRef(load()).current;
  const [rect, setRect] = useState(init.rect);        // the "normal" frame, kept while snapped
  const [mode, setMode] = useState(init.mode);        // normal | zoomed | left | right | fullscreen
  const [state, setState] = useState('open');         // open | minimized | closed
  const [preview, setPreview] = useState(null);       // snap target shown while dragging
  const [dragging, setDragging] = useState(false);
  const [active, setActive] = useState(true);
  const [dockPoint, setDockPoint] = useState({ x: 0, y: 0 });
  const beforeFs = useRef('normal');
  const [, force] = useState(0);

  useEffect(() => {
    try { localStorage.setItem(KEY, JSON.stringify({ rect, mode: mode === 'fullscreen' ? beforeFs.current : mode })); } catch {}
  }, [rect, mode]);

  // Keep the window on screen when the browser is resized.
  useEffect(() => {
    const onResize = () => { setRect((r) => clampRect(r)); force((n) => n + 1); };
    // On a phone the window IS the screen, so it can't stay closed or minimized. Listen on the
    // same breakpoint the CSS uses, which fires even when no resize event does.
    const phone = matchMedia('(max-width: 860px)');
    const onPhone = () => { if (phone.matches) setState('open'); };
    addEventListener('resize', onResize);
    phone.addEventListener('change', onPhone);
    return () => { removeEventListener('resize', onResize); phone.removeEventListener('change', onPhone); };
  }, []);

  // Real full screen follows the browser: leaving it with Esc puts the window back.
  useEffect(() => {
    const onFs = () => {
      const fs = document.fullscreenElement || document.webkitFullscreenElement;
      if (!fs) setMode((m) => (m === 'fullscreen' ? beforeFs.current : m));
    };
    document.addEventListener('fullscreenchange', onFs);
    document.addEventListener('webkitfullscreenchange', onFs);
    return () => { document.removeEventListener('fullscreenchange', onFs); document.removeEventListener('webkitfullscreenchange', onFs); };
  }, []);

  const frame = geometry(mode, rect);

  // ---- title bar: drag to move, drag to an edge to snap, double-click to zoom
  const onTitlePointerDown = useCallback((e) => {
    if (e.button !== 0 || e.target.closest('input, a, button, label') || mode === 'fullscreen') return;
    setActive(true);
    const px = e.clientX, py = e.clientY;
    let start = geometry(mode, rect);
    if (mode !== 'normal') {
      // Pulling a snapped or zoomed window off restores its own size, still under the cursor.
      const fx = (px - start.x) / start.w;
      start = clampRect({ ...rect, x: Math.round(px - rect.w * fx), y: py - 18 });
    }
    let moved = false, snap = null, last = start;
    const el = e.currentTarget; el.setPointerCapture(e.pointerId);
    const move = (ev) => {
      const dx = ev.clientX - px, dy = ev.clientY - py;
      if (!moved && Math.abs(dx) + Math.abs(dy) < 4) return;
      if (!moved) { moved = true; setDragging(true); setMode('normal'); }
      last = clampRect({ ...start, x: start.x + dx, y: start.y + dy });
      setRect(last);
      snap = ev.clientX <= EDGE ? 'left' : ev.clientX >= vw() - EDGE ? 'right' : ev.clientY <= EDGE ? 'zoomed' : null;
      setPreview(snap);
    };
    const up = () => {
      el.removeEventListener('pointermove', move); el.removeEventListener('pointerup', up); el.removeEventListener('pointercancel', up);
      setDragging(false); setPreview(null);
      if (moved && snap) setMode(snap);
    };
    el.addEventListener('pointermove', move); el.addEventListener('pointerup', up); el.addEventListener('pointercancel', up);
  }, [mode, rect]);

  const onTitleDoubleClick = useCallback((e) => {
    if (e.target.closest('input, a, button, label') || mode === 'fullscreen') return;
    setMode((m) => (m === 'zoomed' ? 'normal' : 'zoomed'));
  }, [mode]);

  // ---- resize from any edge or corner
  const onResizePointerDown = useCallback((dir) => (e) => {
    if (e.button !== 0 || mode === 'fullscreen') return;
    e.preventDefault(); e.stopPropagation(); setActive(true);
    const start = geometry(mode, rect);
    if (mode !== 'normal') { setMode('normal'); setRect(start); }
    const px = e.clientX, py = e.clientY;
    const el = e.currentTarget; el.setPointerCapture(e.pointerId); setDragging(true);
    const move = (ev) => {
      const dx = ev.clientX - px, dy = ev.clientY - py; let { x, y, w, h } = start;
      if (dir.includes('e')) w = start.w + dx;
      if (dir.includes('s')) h = start.h + dy;
      if (dir.includes('w')) { w = start.w - dx; x = start.x + dx; }
      if (dir.includes('n')) { h = start.h - dy; y = start.y + dy; }
      if (w < MIN_W) { if (dir.includes('w')) x -= MIN_W - w; w = MIN_W; }
      if (h < MIN_H) { if (dir.includes('n')) y -= MIN_H - h; h = MIN_H; }
      if (y < 0) { h += y; y = 0; }
      // Only a vertical resize is limited by the dock; a sideways one leaves the height alone.
      if (dir.includes('s')) h = Math.min(h, area().h - y);
      setRect({ x, y, w, h });
    };
    const up = () => { el.removeEventListener('pointermove', move); el.removeEventListener('pointerup', up); el.removeEventListener('pointercancel', up); setDragging(false); };
    el.addEventListener('pointermove', move); el.addEventListener('pointerup', up); el.addEventListener('pointercancel', up);
  }, [mode, rect]);

  // ---- the three buttons
  // Leave full screen: the browser's, and the window's own mode, back to what it was before.
  const leaveFullscreen = () => {
    const fs = document.fullscreenElement || document.webkitFullscreenElement;
    if (fs) try { const p = (document.exitFullscreen || document.webkitExitFullscreen).call(document); if (p && p.catch) p.catch(() => {}); } catch {}
    setMode(beforeFs.current);
  };
  // As on macOS, closing a full-screen window leaves full screen too.
  const close = () => { if (mode === 'fullscreen') leaveFullscreen(); setState('closed'); };
  const minimize = (dockEl) => {
    if (mode === 'fullscreen') return;   // macOS disables minimize in full screen
    const f = geometry(mode, rect), d = dockEl?.getBoundingClientRect();
    if (d) setDockPoint({ x: d.left + d.width / 2 - (f.x + f.w / 2), y: d.top + d.height / 2 - (f.y + f.h / 2) });
    setState('minimized');
  };
  const toggleFullscreen = () => {
    const root = document.documentElement;
    const fs = document.fullscreenElement || document.webkitFullscreenElement;
    if (mode === 'fullscreen') { leaveFullscreen(); return; }
    beforeFs.current = mode;
    setMode('fullscreen');
    const req = root.requestFullscreen || root.webkitRequestFullscreen;
    if (req) try { const p = req.call(root); if (p && p.catch) p.catch(() => {}); } catch {}
  };
  const reopen = () => {
    if (state === 'closed') setRect((r) => clampRect(r));
    setState('open'); setActive(true);
  };

  return { frame, mode, state, preview, dragging, active, setActive, dockPoint,
    previewFrame: preview ? geometry(preview, rect) : null,
    onTitlePointerDown, onTitleDoubleClick, onResizePointerDown, close, minimize, toggleFullscreen, reopen };
}
