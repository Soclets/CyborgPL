// The examples, whole (highlighted code, output), from /examples-full.json: fetched once, in the
// background, when the Examples grid shows; an example then opens with everything in place.
let all = null, pending = null;
export const cached = (id) => all?.[id];
export function loadExamples() {
  pending ??= fetch("/examples-full.json").then((r) => r.ok ? r.json() : Promise.reject(r)).then((j) => (all = j)).catch((e) => { pending = null; throw e; });
  return pending;
}
