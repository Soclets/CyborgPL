import type { Route } from "./+types/home";

export const meta: Route.MetaFunction = () => [{ title: "CyborgPL" }];

// A placeholder until the owner describes the look.
export default function Home() {
  return (
    <main>
      <h1>CyborgPL</h1>
      <p>Merging with the machine.</p>
    </main>
  );
}
