// Small stroke icons, drawn for this site (no icon set pulled in).
const P = {
  home: 'M4 11 12 4l8 7v8a1 1 0 0 1-1 1h-5v-6h-4v6H5a1 1 0 0 1-1-1z',
  flag: 'M5 21V4m0 0h11l-2 4 2 4H5',
  map: 'M9 4 3 6v14l6-2 6 2 6-2V4l-6 2-6-2zm0 0v14m6-12v14',
  bolt: 'M13 3 5 14h6l-1 7 8-11h-6z',
  shield: 'M12 3 5 6v6c0 4.5 3 7.5 7 9 4-1.5 7-4.5 7-9V6z',
  key: 'M14 10a4 4 0 1 0-3.9 4.9L10 15v2H8v2H5v-3l5.1-5.1A4 4 0 0 0 14 10zm1-1h.01',
  cube: 'M12 3 4 7.5v9L12 21l8-4.5v-9zm0 0v18M4 7.5l8 4.5 8-4.5',
  alert: 'M12 4 3 20h18zm0 6v4m0 3h.01',
  branch: 'M6 4v10m0 0a3 3 0 1 0 0 6 3 3 0 0 0 0-6zm12-8a3 3 0 1 0 0-6 3 3 0 0 0 0 6zm0 0c0 5-6 4-12 8',
  file: 'M7 3h7l5 5v13H7zm7 0v5h5',
  download: 'M12 4v11m-5-5 5 5 5-5M5 20h14',
  github: 'M9 19c-4 1.5-4-2-6-2.5m12 5v-3.5c0-1 .1-1.4-.5-2 2.8-.3 5.5-1.4 5.5-6a4.6 4.6 0 0 0-1.3-3.2 4.2 4.2 0 0 0-.1-3.2s-1.1-.3-3.5 1.3a12 12 0 0 0-6.2 0C6.5 2.8 5.4 3.1 5.4 3.1a4.2 4.2 0 0 0-.1 3.2A4.6 4.6 0 0 0 4 9.5c0 4.6 2.7 5.7 5.5 6-.6.6-.6 1.2-.5 2V21',
  scale: 'M12 4v16M5 20h14M7 8h10M7 8l-3 6a3 3 0 0 0 6 0zm10 0-3 6a3 3 0 0 0 6 0z',
  search: 'M11 18a7 7 0 1 1 0-14 7 7 0 0 1 0 14zm9 2-4-4',
  sidebar: 'M4 5h16v14H4zm5 0v14',
  arrow: 'M5 12h14m-6-6 6 6-6 6',
  back: 'M19 12H5m6-6-6 6 6 6',
};
export default function Icon({ name, size = 16 }) {
  return (
    <svg className="icon" viewBox="0 0 24 24" width={size} height={size} fill="none" stroke="currentColor"
      strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d={P[name]} />
    </svg>
  );
}
