// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

// https://astro.build/config
export default defineConfig({
  site: "https://languint.github.io/factorio-rs",
  integrations: [
    starlight({
      title: "factorio-rs",
      description:
        "Rust SDK for Factorio modding - transpile Rust to Lua mods.",
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/languint/factorio-rs",
        },
      ],
      editLink: {
        baseUrl: "https://github.com/languint/factorio-rs/edit/main/docs/",
      },
      // Prefer `link` over `slug` so a stale content cache cannot crash the
      // splash page while validating sidebar entries against the collection.
      sidebar: [
        {
          label: "Start here",
          items: [
            { label: "Introduction", link: "/intro/" },
            { label: "Installation", link: "/installation/" },
            { label: "Getting started", link: "/guides/getting-started/" },
          ],
        },
        {
          label: "Guides",
          items: [
            { label: "Stages", link: "/guides/stages/" },
            { label: "Language support", link: "/guides/language/" },
            { label: "Events and filters", link: "/guides/events/" },
            { label: "Mod settings", link: "/guides/mod-settings/" },
            { label: "Locale", link: "/guides/locale/" },
            { label: "Profiles", link: "/guides/profiles/" },
          ],
        },
        {
          label: "Reference",
          items: [
            { label: "CLI", link: "/reference/cli/" },
            { label: "Factorio.toml", link: "/reference/factorio-toml/" },
            { label: "Macros and attributes", link: "/reference/macros/" },
          ],
        },
        {
          label: "Examples",
          items: [
            { label: "hello_world", link: "/examples/hello-world/" },
            {
              label: "mandatory_spaghetti",
              link: "/examples/mandatory-spaghetti/",
            },
          ],
        },
      ],
    }),
  ],
});
