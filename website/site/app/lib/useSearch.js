// Browser side of search: the code index (fetched once, on first use) and recent searches.
import { useCallback, useEffect, useState } from "react";

let codeIndex = null;
let loading = null;
export function useCodeIndex(wanted) {
  const [code, setCode] = useState(codeIndex);
  useEffect(() => {
    if (!wanted || code) return;
    loading ??= fetch("/search-index.json").then((r) => (r.ok ? r.json() : null)).catch(() => null);
    let live = true;
    loading.then((c) => {
      if (c) codeIndex = c;
      else loading = null; // try again next time
      if (live) setCode(c);
    });
    return () => { live = false; };
  }, [wanted, code]);
  return code;
}

// Kept on this phone only; the page works the same without it.
const KEY = "cyborgpl.recent";
const load = () => { try { return JSON.parse(localStorage.getItem(KEY)) ?? []; } catch { return []; } };
export function useRecent() {
  const [recent, setRecent] = useState([]);
  useEffect(() => setRecent(load()), []);
  const save = (list) => { setRecent(list); try { localStorage.setItem(KEY, JSON.stringify(list)); } catch {} };
  const remember = useCallback((q) => {
    const t = q.trim();
    if (t) save([t, ...load().filter((r) => r.toLowerCase() !== t.toLowerCase())].slice(0, 6));
  }, []);
  const clear = useCallback(() => save([]), []);
  return { recent, remember, clear };
}
