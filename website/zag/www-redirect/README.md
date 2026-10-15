# www.zag-lang.org → zag-lang.org

A tiny Worker that answers every request on `www.zag-lang.org` with a 301 to the
same path on `zag-lang.org`. Kept separate so the main site stays static files
only. Deploy: `npx wrangler deploy` from this folder.
