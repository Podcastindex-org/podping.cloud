# GitHub Actions CI + Release Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the manual `docker/multi-arch-builder.sh` front-end image process with GitHub Actions: CI verification on pushes/PRs and a tag-driven Docker Hub release, mirroring the podping-gossipwriter repo.

**Architecture:** The Dockerfile switches from git-cloning GitHub main inside the build to COPYing the checked-out `podping/` crate, so builds compile exactly the triggering commit. Two workflows: `ci.yml` (cargo check/clippy/build + docker smoke build on pushes/PRs) and `release.yml` (tag `vX.Y.Z` → push `podcastindexorg/podcasting20-podping.cloud:X.Y.Z` and `:latest`).

**Tech Stack:** GitHub Actions (`actions/checkout@v4`, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`, `docker/setup-buildx-action@v3`, `docker/login-action@v3`, `docker/build-push-action@v6`), Rust stable, Docker.

## Global Constraints

- Repo: `/mnt/c/Users/dave/RustroverProjects/podping.cloud`, work on branch `github-actions-release` off `main`.
- Image name: `podcastindexorg/podcasting20-podping.cloud` (exact — note the `podcasting20-` prefix).
- Platform: linux/amd64 only.
- Secrets referenced (added by user in repo settings later): `DOCKERHUB_USERNAME`, `DOCKERHUB_TOKEN`.
- All cargo invocations use `--locked`.
- Docker is NOT available in this WSL distro — do not attempt `docker build` locally. Local verification is `cargo metadata --locked` + Python YAML parse; the CI run on the eventual PR is the real proof.
- The `podping` crate's only path dependency is `dbif`, located INSIDE `podping/dbif`, and `build.rs` reads only `podping/plexo-schemas/` — so `COPY podping/ ...` captures everything the build needs.
- Commit messages end with:

  ```
  Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
  ```

---

### Task 1: Convert Dockerfile to build-from-checkout, add .dockerignore

**Files:**
- Modify: `docker/Dockerfile`
- Create: `.dockerignore` (repo root)

**Interfaces:**
- Produces: an image buildable with `docker build -f docker/Dockerfile .` from the repo root (context = repo root). Tasks 2 and 3 rely on exactly that command shape (`file: docker/Dockerfile`, `context: .`).

- [ ] **Step 1: Create the working branch**

```bash
cd /mnt/c/Users/dave/RustroverProjects/podping.cloud
git checkout main && git checkout -b github-actions-release
```

- [ ] **Step 2: Rewrite `docker/Dockerfile`**

Replace the entire file with:

```dockerfile
##: Build stage
FROM rust:latest AS builder

USER root

RUN apt-get update && apt-get install -y apt-utils sqlite3 openssl capnproto libzmq3-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/podping.cloud
COPY podping/ podping/
WORKDIR /opt/podping.cloud/podping
RUN cargo build --release --locked
RUN cp target/release/podping .


##: Bundle stage
FROM debian:bookworm-slim AS runner

USER root

RUN apt-get update && apt-get install -y apt-utils sqlite3 openssl capnproto libzmq3-dev libc6 && rm -rf /var/lib/apt/lists/*

RUN chown -R 1000:1000 /opt
RUN mkdir /data && chown -R 1000:1000 /data

USER 1000
RUN mkdir /opt/podping

WORKDIR /opt/podping
COPY --from=builder /opt/podping.cloud/podping/target/release/podping .
COPY --from=builder /opt/podping.cloud/podping/home.html .

EXPOSE 80/tcp

ENTRYPOINT ["/opt/podping/podping"]
```

(Only the build stage changes: `git clone` of GitHub main becomes `COPY podping/ podping/`, the `ARG GIT_REPO`/`ARG GIT_BRANCH` lines are dropped, and the build gains `--locked`. The bundle stage is byte-identical to the current file.)

- [ ] **Step 3: Create `.dockerignore` at the repo root**

```
podping/target
.git
```

(Without this, a local `docker build` from the repo root would send the multi-GB `podping/target` dir as build context. CI checkouts have no `target/`, but local builds must stay usable.)

- [ ] **Step 4: Verify the lockfile is consistent (this is what `--locked` will enforce in CI)**

```bash
cd /mnt/c/Users/dave/RustroverProjects/podping.cloud/podping
cargo metadata --locked --format-version 1 > /dev/null && echo LOCKFILE-OK
```

Expected: `LOCKFILE-OK`. If it errors with "the lock file ... needs to be updated", STOP — the lockfile is stale and `--locked` builds will fail in CI; report this rather than regenerating the lockfile silently.

(Do not run `cargo check` locally — libzmq is not installed in this WSL distro, so compilation fails locally for environment reasons, not code reasons.)

- [ ] **Step 5: Commit**

```bash
cd /mnt/c/Users/dave/RustroverProjects/podping.cloud
git add docker/Dockerfile .dockerignore
git commit -m "build docker image from checkout instead of cloning main

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 2: Add CI workflow

**Files:**
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: the Task 1 Dockerfile (`docker build -f docker/Dockerfile .` from repo root).
- Produces: a `CI` workflow whose green run on the PR is the acceptance proof for Task 1's Dockerfile conversion.

- [ ] **Step 1: Create `.github/workflows/ci.yml`**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install system dependencies
        run: sudo apt-get update && sudo apt-get install -y capnproto libzmq3-dev libssl-dev pkg-config
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: podping
      - name: Check
        working-directory: podping
        run: cargo check --locked
      - name: Clippy
        working-directory: podping
        run: cargo clippy --locked
      - name: Build
        working-directory: podping
        run: cargo build --locked
      - name: Docker build
        run: docker build -f docker/Dockerfile -t podping-cloud-ci .
```

- [ ] **Step 2: Verify the YAML parses**

```bash
cd /mnt/c/Users/dave/RustroverProjects/podping.cloud
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml')); print('YAML-OK')"
```

Expected: `YAML-OK`

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "add CI workflow (check/clippy/build + docker smoke build)

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 3: Add release workflow

**Files:**
- Create: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: the Task 1 Dockerfile; repo secrets `DOCKERHUB_USERNAME` / `DOCKERHUB_TOKEN` (user adds in GitHub settings — same names as the podping-gossipwriter repo).
- Produces: on tag `vX.Y.Z`, publishes `podcastindexorg/podcasting20-podping.cloud:X.Y.Z` and `:latest`.

- [ ] **Step 1: Create `.github/workflows/release.yml`**

```yaml
name: Release

on:
  push:
    tags: ["v*"]

jobs:
  docker:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Get version from tag
        id: version
        run: echo "version=${GITHUB_REF_NAME#v}" >> "$GITHUB_OUTPUT"
      - uses: docker/setup-buildx-action@v3
      - uses: docker/login-action@v3
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}
      - uses: docker/build-push-action@v6
        with:
          context: .
          file: docker/Dockerfile
          push: true
          tags: |
            podcastindexorg/podcasting20-podping.cloud:${{ steps.version.outputs.version }}
            podcastindexorg/podcasting20-podping.cloud:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

- [ ] **Step 2: Verify the YAML parses**

```bash
cd /mnt/c/Users/dave/RustroverProjects/podping.cloud
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml')); print('YAML-OK')"
```

Expected: `YAML-OK`

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "add release workflow (docker publish on v* tag)

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 4: Retire the manual build script

**Files:**
- Delete: `docker/multi-arch-builder.sh`

**Interfaces:**
- Consumes: nothing. (Verified 2026-07-21: no file in `README.md`, `docs/`, or `examples/` references `multi-arch-builder.sh`, `docker build`, or `buildx` — deletion needs no doc edits.)
- Produces: tag-driven release is the only image-publishing path.

- [ ] **Step 1: Confirm nothing references the script (fresh check at execution time)**

```bash
cd /mnt/c/Users/dave/RustroverProjects/podping.cloud
grep -rn "multi-arch-builder" --exclude-dir=.git --exclude-dir=target . || echo NO-REFERENCES
```

Expected: `NO-REFERENCES` (matches inside `docs/superpowers/` specs/plans are fine — they're historical records, leave them).

- [ ] **Step 2: Delete and commit**

```bash
git rm docker/multi-arch-builder.sh
git commit -m "remove manual image build script; releases now via GitHub Actions tag workflow

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 5: Verification and handoff (user actions)

**Files:** none — this task is the acceptance checklist.

**Interfaces:**
- Consumes: the `github-actions-release` branch from Tasks 1–4.

- [ ] **Step 1: Review the branch diff**

```bash
cd /mnt/c/Users/dave/RustroverProjects/podping.cloud
git diff main..github-actions-release --stat
```

Expected: exactly `docker/Dockerfile` (modified), `.dockerignore` (new), `.github/workflows/ci.yml` (new), `.github/workflows/release.yml` (new), `docker/multi-arch-builder.sh` (deleted).

- [ ] **Step 2 (user): Push the branch and open a PR against main**

```bash
git push -u origin github-actions-release
```

The CI run on this PR is the real verification: it proves `cargo check/clippy/build --locked` and, critically, the converted Dockerfile's docker build.

- [ ] **Step 3 (user): After merge — add secrets, then tag the release**

In GitHub → podping.cloud → Settings → Secrets and variables → Actions → **Secrets** tab, add `DOCKERHUB_USERNAME` and `DOCKERHUB_TOKEN` (read/write Docker Hub token, same as the gossipwriter repo). Then:

```bash
git checkout main && git pull
git tag v3.0.0 && git push origin v3.0.0
```

- [ ] **Step 4 (user): Verify the published image**

```bash
docker pull podcastindexorg/podcasting20-podping.cloud:3.0.0
```

Expected: pull succeeds; this is the image the gossip canary compose references.
