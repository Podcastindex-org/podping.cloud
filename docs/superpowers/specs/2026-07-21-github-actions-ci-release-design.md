# GitHub Actions CI + release for the podping.cloud front-end

**Date:** 2026-07-21
**Status:** Approved

## Goal

Replace the manual front-end image process (`docker/multi-arch-builder.sh`, run
by hand on a workstation) with GitHub Actions, mirroring the setup already in
[podping-gossipwriter](https://github.com/Podcastindex-org/podping-gossipwriter):
CI verification on pushes/PRs, and a tag-driven Docker Hub release.

Immediate payoff: the not-yet-published `podcastindexorg/podcasting20-podping.cloud:3.0.0`
image (referenced by the gossip canary compose) is published by tagging
`v3.0.0` — no local Docker required.

## Decisions

- **Build from checkout, not git-clone.** The Dockerfile currently clones
  GitHub `main` inside the build, so every build compiles whatever main is at
  that moment. It will instead `COPY` the workflow's checked-out source, so a
  tagged release builds exactly the tagged commit, and PR builds test their
  own code.
- **CI + release**, not release-only. PRs get build verification.
- **linux/amd64 only** — matches what is published today and the production
  servers. (The "multi-arch" script only built amd64 anyway.)

## Changes

### 1. `docker/Dockerfile`

Builder stage: drop `ARG GIT_REPO` / `ARG GIT_BRANCH` / `RUN git clone`;
`COPY` the `podping/` crate from the build context instead, and build with
`--locked` against the committed `Cargo.lock`. Build context becomes the repo
root: `docker build -f docker/Dockerfile .`

Runtime stage: unchanged (debian bookworm-slim, runtime deps, user 1000,
`EXPOSE 80`, same entrypoint, copies binary + `home.html`).

Add a root `.dockerignore` (`podping/target`, `.git`) so local builds don't
send the multi-GB target dir as context.

### 2. `.github/workflows/ci.yml`

Trigger: push to `main`, all PRs. One job on `ubuntu-latest`:

1. checkout
2. `apt-get install capnproto libzmq3-dev libssl-dev pkg-config`
3. rust stable toolchain + `Swatinem/rust-cache`
4. `cargo check --locked`, `cargo clippy --locked`, `cargo build --locked`
   (run in `podping/`)
5. smoke `docker build -f docker/Dockerfile .`

No secrets needed, so fork PRs get full verification.

### 3. `.github/workflows/release.yml`

Trigger: tag `v*`. Strips the `v` prefix for the image version, logs in with
`DOCKERHUB_USERNAME` / `DOCKERHUB_TOKEN` repo secrets (same names as the
gossipwriter repo; read/write Docker Hub token), then buildx build-and-push:

- `podcastindexorg/podcasting20-podping.cloud:X.Y.Z`
- `podcastindexorg/podcasting20-podping.cloud:latest`

linux/amd64, GHA layer caching.

### 4. Retire the manual process

Delete `docker/multi-arch-builder.sh`; update any README/runbook references
to describe the tag-driven release instead.

## Behavior change

Releases build the tagged commit, not current `main`. The tag must point at
the commit intended to ship.

## Testing

- CI proves itself on the PR that introduces it (including the Docker smoke
  build of the converted Dockerfile).
- Release workflow is proven by tagging `v3.0.0`, then
  `docker pull podcastindexorg/podcasting20-podping.cloud:3.0.0` and verifying
  the container starts and serves on port 80.
