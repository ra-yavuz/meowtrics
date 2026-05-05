# Deploying a new release of meowtrics

Releases are triggered by **pushing a tag** of the form `vX.Y.Z`. CI does the rest: builds the .deb, attaches it to a GitHub Release, and auto-publishes it to the [ra-yavuz apt repository](https://github.com/ra-yavuz/apt). No manual download or commit step.

## Pre-flight checklist

1. **Bump the version in three places** so they stay in sync:
   - `Cargo.toml` &rarr; `version = "X.Y.Z"`
   - `plasmoid/metadata.json` &rarr; `"Version": "X.Y.Z"`
   - `debian/changelog` &rarr; new entry `meowtrics (X.Y.Z-1) unstable; urgency=low` with notes
2. **Regenerate `Cargo.lock`** (CI uses `--locked` and will fail if you skip this):
   ```bash
   ~/github-ra-yavuz/.claude-dev/run.sh "cd /work/meowtrics && cargo build --release"
   ```
3. **Verify lint passes locally** so CI doesn't reject the tag run:
   ```bash
   ~/github-ra-yavuz/.claude-dev/run.sh "cd /work/meowtrics && cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && shellcheck scripts/*.sh debian/postinst debian/postrm docs/get.sh"
   ```

## Ship it

```bash
git add -A
git commit -m "vX.Y.Z: <one-line summary>"
git push
git tag -a vX.Y.Z -m "vX.Y.Z: <one-line summary>"
git push origin vX.Y.Z
```

That's it. CI takes ~5 minutes:

- **lint** + **build-deb** &rarr; produces `dist/meowtrics_X.Y.Z_amd64.deb` and `dist/meowtrics.plasmoid`
- **release** (tag-only) &rarr; creates the GitHub Release, attaches both files, dispatches `package-published` to `ra-yavuz/apt`
- The apt repo's **add-package** workflow downloads the .deb, drops it into the pool, evicts the older version, commits, pushes
- The push triggers **publish** which rebuilds the apt index and deploys to GitHub Pages

Within ~10 min total, `sudo apt update && sudo apt install meowtrics` serves the new version on every machine that has the apt source set up.

## What if the tag run fails?

Watch CI at https://github.com/ra-yavuz/meowtrics/actions. Common causes:

- **`--locked` build error**: forgot step 2 (regenerate `Cargo.lock`).
- **clippy/fmt/shellcheck errors**: forgot step 3.
- **release upload `403 Resource not accessible`**: the `deb` job needs `permissions: contents: write` (already set in `ci.yml`).
- **apt dispatch silently skipped**: the `Notify ra-yavuz/apt` step requires the `APT_DISPATCH_TOKEN` secret to exist on this repo. Check `Settings &rarr; Secrets and variables &rarr; Actions`.

To retry after a fix: delete and re-push the tag.

```bash
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z
# fix the issue, commit, push
git tag -a vX.Y.Z -m "vX.Y.Z: ..."
git push origin vX.Y.Z
```

## What gets published

| Surface | URL | Updated by |
|---|---|---|
| GitHub Release | `https://github.com/ra-yavuz/meowtrics/releases/tag/vX.Y.Z` | `release` job |
| .deb in apt repo | `https://ra-yavuz.github.io/apt/pool/main/m/meowtrics/` | `add-package` &rarr; `publish` |
| Apt index | `https://ra-yavuz.github.io/apt/dists/stable/main/binary-amd64/Packages` | `publish` |
| Project Pages | `https://ra-yavuz.github.io/meowtrics/` | redeploys on any push to `main` |

`docs/messages.json` ships from the `main` branch automatically (Pages source: `main`, path `/docs`). To update the message database without a code release, just commit to main.

## How the auto-publish path works

After the GitHub Release exists, `release` job runs this step:

```bash
curl -X POST \
  -H "Authorization: token $APT_DISPATCH_TOKEN" \
  https://api.github.com/repos/ra-yavuz/apt/dispatches \
  -d '{"event_type":"package-published","client_payload":{
        "repo":"ra-yavuz/meowtrics",
        "tag":"vX.Y.Z",
        "arch":"amd64",
        "deb_url":"https://github.com/ra-yavuz/meowtrics/releases/download/vX.Y.Z/meowtrics_X.Y.Z_amd64.deb"}}'
```

`ra-yavuz/apt`'s `add-package.yml` listens for `repository_dispatch: package-published`, downloads the .deb, places it under `pool/main/m/meowtrics/`, evicts older versions of the same `(package, arch)` pair, and pushes a commit signed `ra-yavuz-bot`. The push fires `publish.yml` which signs and rebuilds the apt index. End to end &lt; 5 min once the dispatch fires.

## Same flow lives in herald, hydra-llm, inhibit-charge

The four packaged repos share the same CI shape (`lint &rarr; build-deb &rarr; release`) and the same dispatch pattern. Each has its own `DEPLOY.md` with the project-specific version-file list.
