// What the topbar offers (owner, 2026-10-19: "Remove topbar things (including the ones on the right),
// and add "Docs", "Comparison", "News", "More". On the right, there's "Get CyborgPL" with the
// arrow thingy"). An item may still carry a menu; none does now.
export type NavLink = { label: string; to: string; note?: string };
export type NavItem = { label: string; to?: string; menu?: NavLink[] };

export const NAV: NavItem[] = [
  { label: "Docs", to: "/docs" },
  { label: "Comparison", to: "/comparison" },
  { label: "News", to: "/news" },
  { label: "More", to: "/more" },
];

export const PRIMARY: NavLink & { menu: NavLink[] } = {
  label: "Get CyborgPL", to: "/download",
  menu: [
    { label: "CyDT 1", to: "/download#cydt", note: "The dev kit: write and build CyborgPL" },
    { label: "CyCT 1", to: "/download#cyct", note: "The contribute kit: work on CyborgPL itself" },
  ],
};
