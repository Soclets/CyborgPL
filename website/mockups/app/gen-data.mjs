// Pre-renders the real examples (highlighted with the CyborgPL grammar) and the logo
// into a JSON module the React app imports. Runs before `vite build`.
import { createHighlighter } from 'shiki';
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

// The language repo's root: this file sits at website/mockups/app/.
const REPO = fileURLToPath(new URL('../../../', import.meta.url));
const grammar = JSON.parse(readFileSync(REPO + 'website/grammar/cyborgpl.tmLanguage.json', 'utf8'));
// Solarized Dark (Ethan Schoonover): base03 002b36 · base02 073642 · base01 586e75 · base0 839496 · base1 93a1a1
// yellow b58900 · orange cb4b16 · red dc322f · magenta d33682 · violet 6c71c4 · blue 268bd2 · cyan 2aa198 · green 859900
const theme = { name: 'solarized-cyborg', type: 'dark', colors: { 'editor.background': '#00000000', 'editor.foreground': '#839496' }, settings: [
  { settings: { foreground: '#839496' } },
  { scope: ['comment'], settings: { foreground: '#586e75', fontStyle: 'italic' } },
  { scope: ['string'], settings: { foreground: '#2aa198' } },
  { scope: ['constant.character.escape', 'constant.character.newline'], settings: { foreground: '#dc322f' } },
  { scope: ['constant.numeric', 'keyword.other.unit', 'constant.language'], settings: { foreground: '#d33682' } },
  { scope: ['keyword.control', 'keyword.declaration', 'keyword.other', 'storage.modifier'], settings: { foreground: '#859900' } },
  { scope: ['keyword.operator.assignment.infer', 'keyword.operator.word'], settings: { foreground: '#859900' } },
  { scope: ['keyword.operator'], settings: { foreground: '#839496' } },
  { scope: ['support.type', 'entity.name.type'], settings: { foreground: '#b58900' } },
  { scope: ['entity.name.function', 'entity.name.namespace'], settings: { foreground: '#268bd2' } },
  { scope: ['variable.other.enummember', 'entity.name.tag'], settings: { foreground: '#cb4b16' } },
  { scope: ['variable.other'], settings: { foreground: '#6c71c4' } },
  { scope: ['variable.other.member', 'variable.other.member.declaration'], settings: { foreground: '#93a1a1' } },
  { scope: ['punctuation'], settings: { foreground: '#586e75' } },
]};
const hl = await createHighlighter({ themes: [theme], langs: [grammar] });

const EXAMPLES = [
  ['enums', 'Enums & match'], ['fizzbuzz', 'FizzBuzz'], ['generics', 'Generics'],
  ['ownership', 'Ownership'], ['failures', 'Failures'], ['concur', 'Concurrency'], ['hello', 'Hello, World'],
];
const examples = EXAMPLES.map(([id, title]) => {
  const src = readFileSync(`${REPO}examples/${id}.cpls`, 'utf8').replace(/\n$/, '');
  const out = readFileSync(`${REPO}examples/${id}.out`, 'utf8').replace(/\n$/, '');
  return { id, title, file: `examples/${id}.cpls`, lines: src.split('\n').length, html: hl.codeToHtml(src, { lang: 'cyborgpl', theme: 'solarized-cyborg' }), out };
});

const logo = readFileSync(REPO + 'brand/logo.svg', 'utf8');
const logoInner = logo.slice(logo.indexOf('>', logo.indexOf('<svg')) + 1, logo.lastIndexOf('</svg>'));

writeFileSync(new URL('./src/data.json', import.meta.url), JSON.stringify({ examples, logoInner }));
console.log('data:', examples.length, 'examples');
