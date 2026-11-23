# Releasing SDS Site Generator

Releases use the same release-PR model as SDS Lite. Do not edit release versions
locally, push release tags, publish a draft yourself, or run `npm publish`.

## Repository setup

Before the first release:

1. Keep `main` as the default branch. If renaming it, update the branch filters
   in the check and release workflows.
2. Enable Actions, allow Actions to create pull requests, and permit the
   workflows' declared `GITHUB_TOKEN` permissions. No personal token is needed.
3. Require **Build and test** and normal review on `main`. Allow the bot to push
   `release/*` and `revert-release/*` branches, without bypassing review on `main`.
4. Configure `github-packages-production` and `github-packages-beta`
   environments. Restrict them to the intended release workflow/branches.
   Required reviewers add an optional publication approval; omit them if PR
   review should be the only routine approval.
5. Enable Pages with **GitHub Actions** as the source, and configure its
   `github-pages` environment.
6. If using a repository owner other than `cmu-sei`, update the npm scope,
   repository metadata, and installation links. The npm scope must match the
   repository owner.

The first package publication uses `GITHUB_TOKEN`; for an existing package,
grant this repository Actions access to it. Configure package visibility
separately. Public standalone downloads need a public repository; npm downloads
still require GitHub Packages authentication.

## Normal release

1. From `main`, run **Release - Create Release PR** in Actions.
2. Choose `stable` or `beta` and a base version, such as `0.2.0`.
   Stable becomes `0.2.0` on npm `latest`; beta automatically becomes the next
   `0.2.0-beta.N` on npm `beta`. The initial `0.1.0` release is supported.
3. Follow the workflow summary to the release PR. Automation synchronizes
   `package.json`, `package-lock.json`, `Cargo.toml`, and `Cargo.lock`, creates
   draft release notes, and explicitly dispatches **Check** for the release
   branch. This does not depend on bot-created PR events triggering CI.
4. Review the version changes and draft notes, wait for **Build and test**,
   and merge normally. Release PRs may change only those four version files;
   merge source, documentation, and workflow changes before preparation.
5. **Automatic - Publish Release** validates the exact merged commit and
   repeats the six-platform and browser checks. Approve an environment
   deployment only if repository settings require it.

Publication verifies checksums, attaches six native archives, the npm tarball,
and `SHA256SUMS` to the prepared draft, publishes the GitHub release/tag at the
merged commit, then publishes the exact tested npm tarball. Stable and beta use
separate npm tags and environments. No native binaries are committed.

Stable publication hands off its immutable commit to **Deploy documentation to
Pages**. Beta releases and ordinary pushes to `main` do not deploy production
documentation. Pages rechecks npm `latest` after any deployment approval, so an
older queued deployment cannot replace a newer release.

Only one release may be in preparation or incompletely published at a time.
This includes orphan drafts and release branches left by failed preparation,
even when the next attempt requests a different version. Discard that partial
preparation before starting another release.
Use a newer base version if development has already advanced past the desired
version. Latest-only hotfix branching is not currently automated; this release
flow publishes reviewed default-branch source.

## Retrying failures

Use **Re-run failed jobs** on the original publication run. Do not rerun
successful build jobs: publication resumes with their retained, tested bundle.
Already-uploaded assets and already-published npm versions must match the exact
tested bytes. Conflicting assets, tags, or npm integrity stop publication rather
than being overwritten. Distribution tags cannot move backward.

GitHub and npm cannot publish atomically. If GitHub publication succeeds but npm
fails, retry publication; do not cancel or delete the published release. If GitHub
refuses automatic tag creation because of workflow-file permission rules, create
the release/tag in the GitHub UI at the exact reviewed merge commit reported by
the run, then rerun the failed job. Never retarget an existing tag.

Build artifacts are retained for seven days. Resolve failed publication before
they expire. If they have expired, do not manufacture a replacement for an
already-published version; investigate recovery before rebuilding.

For a failed Pages handoff or deployment, run **Deploy documentation to Pages**
manually from `main` with the published stable tag. It verifies the tag, package
integrity, checksums, and current npm `latest` before deployment.

## Discarding or cancelling

Run **Recovery - Discard or Cancel Release** from `main`. Enter the exact
prepared version twice to confirm.

- **discard**: for an unmerged release or partial preparation. Closes the release
  PR if present, removes only its draft, and deletes its verified release branch.
- **cancel**: for a merged but completely unpublished release. Pauses publication
  with a `release-cancel-requested` label, stops publication jobs, and acquires
  the publication lock. After rechecking that nothing was published, records a
  permanent `release-cancelled` marker, removes the draft, and opens a reviewed
  revert PR. Merge that recovery PR before preparing a newer release.

Recovery refuses any version with a published GitHub release, tag, or npm
package. It never deletes published versions or rewrites history. Do not remove
the cancellation label: publication checks it again before writes, including
after deployment approval. Repeated recovery runs resume existing recovery PRs.
If creating or reopening a recovery PR succeeds but starting CI fails, rerun
recovery; it restarts the required checks for the existing unmerged PR.
If publication wins a race with cancellation, recovery refuses cleanup and removes
only the temporary request label so the partially published release can resume.

## Local validation

```sh
npm run test:package
```

The release tests exercise version synchronization with LF and CRLF line endings,
channel ordering, release
identity, artifact checksums, publication retries, and cancellation safeguards.
Fixtures model GitHub's distinct PR list/detail responses, paginated run history,
and races between cancellation and a run finishing.
The six-platform and browser builds remain required in GitHub Actions.
