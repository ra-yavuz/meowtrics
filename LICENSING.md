# Licensing policy

meowtrics is **MIT-licensed** software (see [`LICENSE`](LICENSE)). It is a personal, non-commercial open-source project, deliberately separate from the author's professional work at https://ramazan-yavuz.tr.

## Rules for shipped emoji / icon assets

The default install ships **zero bundled glyph assets**: the tray icon is rendered from Unicode codepoints and the user's installed emoji font (typically Noto Color Emoji on Linux). Unicode codepoints are not copyrightable; the glyphs the user sees come from their own system. This sidesteps every licensing question for the core package.

Optional companion packs ship as separate `.deb` packages. Each pack:

1. MUST have its own `LICENSE` file installed under `/usr/share/doc/meowtrics-pack-<name>/`.
2. MUST have a `pack-manifest.toml` recording `license` (SPDX id), `upstream` URL, `attribution` text, and the upstream commit/version the assets were sourced from.
3. MUST be traceable: every glyph file's source must be recorded. No "found online" assets.
4. Permitted licenses: MIT, Apache-2.0, OFL-1.1, CC-BY-4.0, CC-BY-SA-4.0, CC-BY-NC-SA-4.0 (NC packs labelled clearly, see below).
5. Forbidden: anything proprietary (Apple, Samsung, WhatsApp, Discord custom emoji), anything without a clear license, anything that is "free for personal use only on platform X".

## Non-commercial framing and NC packs

meowtrics itself is MIT, so it is technically commercial-friendly. In practice it is intended as a personal, free-of-charge project. Anyone redistributing meowtrics commercially is welcome to do so under MIT.

Packs licensed under CC-BY-NC-SA-4.0 (e.g. Mutant Standard, if shipped) are **labelled clearly** in the package name (`meowtrics-pack-<name>-nc`) and in `pack-manifest.toml`. Anyone redistributing meowtrics commercially must omit NC-labelled packs.

## Planned packs (not yet shipped)

| Pack | Upstream | License | Status |
|---|---|---|---|
| `meowtrics-pack-blobcat` | Volpeon's Blobcat (https://volpeon.ink/emojis/blobcat/) | Apache-2.0 | planned, v0.2 |
| `meowtrics-pack-neocat` | Volpeon's Neocat | CC-BY-4.0 | planned, v0.2 |
| `meowtrics-pack-blobmoji` | Original Google Android 7.1 blobs | Apache-2.0 | planned, v0.2 |
| `meowtrics-pack-openmoji` | OpenMoji | CC-BY-SA-4.0 | planned, v0.2 |
| `meowtrics-pack-fluentui` | Microsoft Fluent UI Emoji | MIT | planned, v0.2 |
| `meowtrics-pack-twemoji` | jdecked/twemoji fork | CC-BY-4.0 | planned, v0.2 |

Every pack will be reviewed and its upstream license file vetted before shipping. CI enforces that pack manifests carry a known-good SPDX identifier.

## Disclaimer

This software is provided AS IS, without warranty of any kind, express or implied. The author is not liable for any damage to hardware, data, or system. By installing and running this software you accept full responsibility. See [`README.md`](README.md) for the full disclaimer.
