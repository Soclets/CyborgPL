// What the topbar offers (owner, 2026-10-19: "Remove topbar things (including the ones on the right),
// and add "Docs", "Comparison", "News", "More". On the right, there's "Get CyborgPL" with the
// arrow thingy"). An item may carry a menu, which opens in the shared gliding panel (topbar.tsx).
export type NavLink = { label: string; to: string; note?: string };
export type NavItem = { label: string; to?: string; menu?: NavLink[] };

export const NAV: NavItem[] = [
  { label: "Docs", to: "/docs" },
  { label: "Comparison", to: "/comparison" },
  { label: "News", to: "/news" },
  // More opens a menu (owner, 2026-10-19: "More has License, Credit, Source", then "Change License
  // to Legal, and include 1 & 2 in there": trademarks and logo, and privacy; then "yea" to a dropdown).
  { label: "More", menu: [
    { label: "Legal", to: "/legal", note: "Licences, trademarks and logo, privacy" },
    { label: "Credits", to: "/credits", note: "Who and what built CyborgPL" },
    { label: "Source", to: "/source", note: "Where the code lives" },
  ] },
];

export const PRIMARY: NavLink & { menu: NavLink[] } = {
  label: "Get CyborgPL", to: "/download",
  menu: [
    { label: "CyDT 1", to: "/download#cydt", note: "The dev kit: write and build CyborgPL" },
    { label: "CyCT 1", to: "/download#cyct", note: "The contribute kit: work on CyborgPL itself" },
  ],
};
