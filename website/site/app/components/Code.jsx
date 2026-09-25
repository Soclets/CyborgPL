// Highlighted code, pre-rendered at build time by Shiki with the repo's grammar.
export default function Code({ html, className = "" }) {
  return <div className={"code " + className} dangerouslySetInnerHTML={{ __html: html }} />;
}
