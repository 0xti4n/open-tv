# Nyx TV desktop

The desktop frontend is built with Dioxus 0.7.10. From this directory, run
`dx serve --platform desktop` for development or `dx bundle --platform desktop
--package-types deb` to produce a Debian package.

Tailwind CSS v4 source and theme tokens are in `tailwind.css`; Dioxus CLI
processes it into `assets/tailwind.css` during a build. If running Tailwind
standalone, use `npx @tailwindcss/cli -i tailwind.css -o assets/tailwind.css`.
