# www.nerve-lang.com → nerve-lang.com

A tiny Worker that answers every request on `www.nerve-lang.com` with a 301 to the
same path on `nerve-lang.com`. Kept separate so the main site stays static files
only. Deploy: `npx wrangler deploy` from this folder.
