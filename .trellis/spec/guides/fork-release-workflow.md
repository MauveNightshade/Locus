# Fork Release Workflow

## Repository Roles

- `upstream`: `r1n7aro/Locus`, the official repository.
- `origin`: `MauveNightshade/Locus`, this fork.
- `main`: an exact mirror of `upstream/main`. Do not add fork code, README wording, release metadata, or fork-specific links here.
- `mauvenightshade/main`: the long-lived fork integration branch. Put all fork behavior, fork README content, and fork-specific release metadata here.
- `release/vX.Y.Z`: a temporary or retained release snapshot created from the fork integration branch when preparing an installer release.

## Sync Official Upstream

Before any fork work, fetch the official repository and make `main` a fast-forward-only mirror:

```powershell
git fetch upstream --prune
git switch main
git merge --ff-only upstream/main
git push origin main
```

Then update the fork integration branch from the mirrored baseline:

```powershell
git switch mauvenightshade/main
git merge main
```

Do not merge `mauvenightshade/main` or a release branch back into `main`. A merge commit or divergent commit on `main` makes future upstream synchronization ambiguous.

## Fork Changes

- Keep fork-only implementation, generated runtime exports required by a release, and README links on `mauvenightshade/main`.
- README must state that the repository is a fork, name the official upstream, name the fork integration branch, and link fork releases to `https://github.com/MauveNightshade/Locus/releases`.
- Keep official documentation links when this fork has no separate documentation site.
- Do not add local `.agents/`, `.opencode/`, `.trellis/tasks/`, or `.trellis/workspace/` files to Git unless explicitly requested.

## Fork Versioning And Releases

The base version tracks the official release. Fork builds use a SemVer prerelease suffix:

```text
Official v0.5.7, first fork release: v0.5.7-mauvenightshade.1
Next fork release on same base:      v0.5.7-mauvenightshade.2
Official v0.5.8, first fork release: v0.5.8-mauvenightshade.1
```

Do not reuse the official tag (for example `v0.5.7`) for a different fork commit.

Set the identical version value in:

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`
- `docs/overview/latest-version.mdx`
- `docs/en/overview/latest-version.mdx`

Regenerate release manifests and verify them before building:

```powershell
bun run release:generate
bun run release:verify-version
bun run release:installers
```

Expected installers are under `src-tauri/target/release/bundle/nsis/`:

- `locus_<version>_x64-setup.exe`
- `locus_<version>_x64-without_embed_python_git-setup.exe`

## Commit, Tag, And Push

After reviewing generated-file changes and confirming the release checks, commit on the release branch, create an annotated tag, and push only to `origin`:

```powershell
git add <reviewed release files>
git commit -m "chore(release): vX.Y.Z-mauvenightshade.N"
git tag -a vX.Y.Z-mauvenightshade.N -m "Release vX.Y.Z-mauvenightshade.N"
git push origin release/vX.Y.Z
git push origin vX.Y.Z-mauvenightshade.N
```

Never push fork commits or tags to `upstream`. If GitHub is unreachable, keep the local commit and annotated tag intact, report the connectivity failure, and retry the two push commands after connectivity returns.

## Release Checklist

- [ ] `main` is an exact fast-forwarded copy of `upstream/main`.
- [ ] Fork changes are on `mauvenightshade/main` or a branch derived from it.
- [ ] The version is unique and has the `-mauvenightshade.N` suffix.
- [ ] Both README files point fork release links at `origin`.
- [ ] `bun run release:verify-version` passes.
- [ ] Installer paths and filenames include the fork version.
- [ ] Only reviewed files are staged; local Trellis and agent directories remain untracked.
- [ ] Only `origin` receives the branch and tag pushes.
