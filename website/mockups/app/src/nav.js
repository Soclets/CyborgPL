import data from './data.json';

// Sidebar structure. Every item is a route; `#/section/id` keeps each page linkable.
export const SECTIONS = [
  { items: [{ route: 'welcome', label: 'Welcome', icon: 'home' }] },
  {
    title: 'Learn',
    items: [
      { route: 'learn/getting-started', label: 'Getting started', icon: 'flag' },
      { route: 'learn/tour', label: 'Language tour', icon: 'map' },
      { route: 'learn/syntax', label: 'Syntax in five minutes', icon: 'bolt' },
    ],
  },
  {
    title: 'Language',
    items: [
      { route: 'lang/precog', label: 'Precog', icon: 'shield' },
      { route: 'lang/ownership', label: 'Ownership', icon: 'key' },
      { route: 'lang/types', label: 'Types', icon: 'cube' },
      { route: 'lang/failures', label: 'Failures', icon: 'alert' },
      { route: 'lang/concurrency', label: 'Concurrency', icon: 'branch' },
    ],
  },
  {
    title: 'Examples',
    items: data.examples.map((e) => ({ route: `examples/${e.id}`, label: e.title, icon: 'file' })),
  },
  {
    title: 'Project',
    items: [
      { route: 'download', label: 'Download', icon: 'download' },
      { route: 'source', label: 'Source', icon: 'github' },
      { route: 'licence', label: 'Licence', icon: 'scale' },
    ],
  },
];

export const ALL_ITEMS = SECTIONS.flatMap((s) => s.items);
export const findItem = (route) => ALL_ITEMS.find((i) => i.route === route);
