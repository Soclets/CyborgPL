// What the topbar offers. A PROPOSAL until the owner names the items (2026-10-19): the shape
// follows x.ai's bar (owner's screenshot), the words are website claude's placeholders.
export type NavLink = { label: string; to: string; note?: string };
export type NavItem = { label: string; to?: string; menu?: NavLink[] };

export const NAV: NavItem[] = [
  { label: "Language", menu: [
    { label: "Overview", to: "/", note: "What CyborgPL is" },
    { label: "Precog", to: "/precog", note: "Proving every check at compile time" },
    { label: "Features", to: "/features", note: "The five pillars" },
  ] },
  { label: "Learn", menu: [
    { label: "Docs", to: "/docs", note: "The language, topic by topic" },
    { label: "Examples", to: "/examples", note: "Every program, with what it prints" },
    { label: "Good practice", to: "/good-practice", note: "How CyborgPL is meant to be written" },
  ] },
  { label: "Tools", menu: [
    { label: "KAYO\\n", to: "/kayo", note: "The compiler" },
    { label: "CyJet", to: "/cyjet", note: "The package manager" },
    { label: "CyDT and CyCT", to: "/download", note: "The dev kit and the contribute kit" },
  ] },
  { label: "Project", menu: [
    { label: "Decisions", to: "/decisions", note: "Every settled choice, dated" },
    { label: "Licence", to: "/licence", note: "Apache-2.0 and CC BY 4.0" },
    { label: "Credits", to: "/credits", note: "Who and what built it" },
  ] },
  { label: "Benchmarks", to: "/benchmarks" },
  { label: "News", to: "/news" },
];

export const SECONDARY: NavLink = { label: "Docs", to: "/docs" };
export const PRIMARY: NavLink & { menu: NavLink[] } = {
  label: "Get CyborgPL", to: "/download",
  menu: [
    { label: "CyDT 1", to: "/download#cydt", note: "The dev kit: write and build CyborgPL" },
    { label: "CyCT 1", to: "/download#cyct", note: "The contribute kit: work on CyborgPL itself" },
  ],
};
