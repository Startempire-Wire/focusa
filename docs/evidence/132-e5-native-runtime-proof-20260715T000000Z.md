# 132 E5 strict native runtime matrix proof

## Result

- Work item: `focusa-w26jj.9.1.2`
- Final commit: `92ae13e8b7e11d20c9dce3781a8a9c5b16d187bc`
- GitHub Actions run: `29384699342`, attempt 2
- Run URL: `https://github.com/Startempire-Wire/focusa/actions/runs/29384699342/attempts/2`
- Final conclusion: **success**
- Evidence retention: GitHub artifacts expire 2026-10-13.

Attempt 1 was cancelled when the dedicated musl artifact build reached its timeout. GitHub exposed no completed log for that cancelled job. Attempt 2 reran only the cancelled dependency chain; the musl artifact and dependent KH runtime proof both passed. All other successful jobs were retained.

## Runtime matrix

| Profile | Timestamp UTC | Target | Durable evidence | Artifact/proof SHA-256 | CLI SHA-256 | TUI SHA-256 | Result |
|---|---|---|---|---|---|---|---|
| KH native host | `20260715T033121Z` | `x86_64-unknown-linux-musl` | artifact `8331701114`, `focusa-132-e5-runtime-kh-glibc-2.28` | `5093a83dd5b73049c1d3802d759303950a03e254d700d67e709d4dd33638d692` | `aafa7b2b16492afc73de91375eca7638dbfe858f4bc74472ceb6d68a837c082d` | `bb2c6db6e98d743b88eb0bca49d3c5050112c3e648562d995c36ddbd7396599c` | PASS |
| OVH native debug | `20260715T034052Z` | `x86_64-unknown-linux-gnu` | Sol-local canonical `focusa-ovh-build` proof | `ce752282937c1bb137a0ff6802a19392386383e9331957828974851e108242d0` | `b032189f04a21349464b9eba61fbbbc70e618ce566decbaf9c1ab51e64432da3` | `1e5be722aea3cab075088375cfec7272c96847b4e441e03b9c2fd9ea46c9f484` | PASS |
| Linux musl release | `20260715T024727Z` | `x86_64-unknown-linux-musl` | artifact `8331060830`, `focusa-132-e5-runtime-linux-musl-release` | `1430e229b6bc83e36e36285321a760a07c09be8383b86c6755d20ed4435fb984` | `c0733c32869bfb5866f5c3b66d1d4c595347eaaa7f7831d1959354f7a5218601` | `6175aca4b1bc4f14826b14f41f3a9ef97aca22e99221ba3cc858ce13f9095795` | PASS |
| macOS hosted native | `20260715T024442Z` | `aarch64-apple-darwin` | artifact `8331023909`, `focusa-132-e5-runtime-macos-latest` | `4cfb22cf565a8f70aaba775caebd0c074a50f87940f26ecc38d01e2a287ae28f` | `1ccfcb869eb134e286775940af4e6398c036fd775bd5398287997d245747ef78` | `c745fd2b5d2213e5ea3d89c484a86c2a085d53e63bb6f214b77659c5ea563b14` | PASS |
| Windows ConPTY | `20260715T024608Z` | `x86_64-pc-windows-msvc` | artifact `8331047981`, `focusa-132-e5-runtime-windows-conpty` | `7bc3dceab8261a328efd7fb3e88a4f535d3f3ed0afde13efac9091908b895229` | `6b3dd6f3eb9244ec48b533af46f37573e8bbf2378b63f3fdab7333028c295dec` | `cc8959f73e3fd85305e83f593ce9777b7d01878cc4dabde972cb63e548649831` | PASS |
| Linux hosted GNU | `20260715T024333Z` | `x86_64-unknown-linux-gnu` | artifact `8331008688`, `focusa-132-e5-runtime-ubuntu-latest` | `98f00c26892626df330ec959059ab1efd1899a2b908056a9769a2946e139414f` | `296812d8a0a3abd665ec43ed29e9964a5fdb3167f50832c5805b8e2b9644ae22` | `c87ed9d92ccd9add9679d044b8882f4ce050f94c14614adcb6643aeb08d9596f` | PASS |

The KH lane downloaded and hash-verified the exact Ubuntu-built musl bundle before executing it. Bundle artifact `8331697160` (`focusa-132-e5-linux-musl-release`) has downloaded ZIP SHA-256 `5baa391b9749df08239031d248021117c453c6e9a795509bb446447a1fac3a9f`; its payload filename embeds the producer tarball hash.

## Runtime assertions

Unix/KH/OVH/macOS proof artifacts record paths, versions, file identities, file formats, SHA-256 hashes, stdout/stderr paths, and exit codes for:

- installer preflight under CI/`TERM=dumb`/`NO_COLOR`: exit 0;
- installer preflight with reduced motion/no color: exit 0;
- plain/no-animation installer preflight: exit 0;
- updater `status` and `plan` JSON contracts: exit 0, read-only, no mutation;
- `focusa-tui --version`: exit 0;
- `focusa-tui --headless-self-test`: exit 0;
- ordinary TUI launch with redirected stdio: exit 64 with `FOCUSA_TUI_NON_TTY` recovery guidance.

Windows additionally proves:

- native ConPTY host probe: exit 0;
- installer preflight durable truth survives ANSI normalization;
- no alternate-screen entry in the plain/non-animated path;
- deliberately long-lived owned child hits the timeout regression and is terminated;
- CLI/TUI updater and non-TTY contracts match the Unix lanes.

## Release-target build gate

Run `29384699342` also passed non-publishing release builds for:

- `aarch64-apple-darwin`;
- `x86_64-apple-darwin`;
- `x86_64-unknown-linux-gnu`;
- `x86_64-unknown-linux-musl`;
- `aarch64-pc-windows-msvc`.

The Pi-extension package/typecheck/lint gate also passed.

## Authority boundary

The self-hosted Actions runner identity `github-actions` has no passwordless wirebot sudo or wirebot-owned OVH SSH access. Therefore OVH proof is not represented as a GitHub-hosted job. Sol executed the same committed harness through the canonical local `focusa-ovh-build` route, copied bounded evidence, verified its SHA-256, and removed the unique remote temporary directory. No release, deployment, production-binary overwrite, or system-permission change occurred.
