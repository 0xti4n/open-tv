# Fred TV desktop

The desktop frontend is built with Dioxus 0.7.10. From this directory, run
`dx serve --platform desktop` for development or `dx bundle --platform desktop
--package-types deb` to produce a Debian package.

Tailwind CSS v4 source and theme tokens are in `tailwind.css`; Dioxus CLI
processes it into `assets/tailwind.css` during a build. If running Tailwind
standalone, use `npx @tailwindcss/cli -i tailwind.css -o assets/tailwind.css`.

Startup EPG checks are intentionally deferred to phase 2. Settings are
available as a placeholder route; advanced settings, modals, and tray behavior
are also reserved for that phase.
