# HAP-rs Demo Script

**Story:** Port `homebridge/HAP-NodeJS` (TypeScript) to Rust using GitHub Copilot + Foundry Local, showcasing what only Microsoft can do end-to-end. Route mechanical work to open-weight models on-device (cost story), frontier reasoning to Opus (routing story), adversarial spec review to an air-gapped fine-tuned model (security story), and close on WSL running the accessory paired to an iPhone.

**Runtime:** 5–7 minutes of stage time. Assumes all setup completed the day before.

**Audience:** Executive / analyst / customer / field.

---

## Table of contents

- [Demo narrative](#demo-narrative)
- [Pre-demo checklist](#pre-demo-checklist)
- [The four beats](#the-four-beats)
  - [Beat A — Cost story (Foundry Local)](#beat-a--cost-story-foundry-local)
  - [Beat B — Model routing (Opus for crypto)](#beat-b--model-routing-opus-for-crypto)
  - [Beat C — Security story (air-gapped review)](#beat-c--security-story-air-gapped-review)
  - [Beat D — WSL runtime close (iPhone pairs)](#beat-d--wsl-runtime-close-iphone-pairs)
- [Closing talk track](#closing-talk-track)
- [Q&A prep](#qa-prep)
- [Troubleshooting on stage](#troubleshooting-on-stage)
- [Setup reference (if you need to rebuild)](#setup-reference-if-you-need-to-rebuild)
- [Configuration reference](#configuration-reference)

---

## Demo narrative

Open by framing the story before touching the machine:

> "I'm taking a real, complex, spec-driven TypeScript codebase — Apple's HomeKit Accessory Protocol — and rewriting it in Rust. I'll do it inside GitHub, in Codespaces, with Copilot's coding agent, routing each subtask to the right model via BYOM. Compliance to Apple's HAP spec is tracked as GitHub Issues. And when I'm done, I'll `wsl` into my Windows box and pair an iPhone with it. **Nobody else has this full stack.**"

Why Rust matters (for the "so what" question):

- ~70% of CVEs are memory safety issues — Rust eliminates the class entirely.
- P99 latency benefits from no GC pauses.
- Microsoft is already using Rust in Windows kernel, HyperV, Office MonoRepo, Azure Overlake — this isn't experimental.

Why this demo matters:

- Cursor can't do the coding-agent + BYOM + local-runtime story.
- Codex CLI can't do the GitHub Issues + Actions + runner integration.
- Only **GitHub Copilot + Foundry Local + WSL** is a first-party stack that spans all three.

---

## Pre-demo checklist

Run the day before, and again 30 minutes before showtime.

### The day before

```powershell
# Verify the full stack
cd $env:USERPROFILE\Downloads
.\02-verify-setup.ps1 -GitHubUser AndrewMFlick
```

All 10 checks should be green. Common fixes if not:

| Check | If red, do this |
|---|---|
| 1. Foundry service on :5273 | `cd C:\ ; foundry service restart` |
| 2. Required models loaded | `foundry model load qwen2.5-coder-7b` |
| 3. COPILOT_* env vars | Open a fresh PowerShell session |
| 5. gh auth | `gh auth login` |
| 9. Runner online | Run `03-register-runner.ps1` |

### 30 minutes before demo

```powershell
# 1. Foundry Local up and model loaded
cd C:\
foundry service status
foundry model load qwen2.5-coder-7b

# 2. Quick inference test - verifies the whole stack
$body = '{"model":"qwen2.5-coder-7b-instruct-openvino-npu:4","messages":[{"role":"user","content":"hi"}],"stream":false,"max_tokens":20}'
(Invoke-RestMethod http://localhost:5273/v1/chat/completions -Method Post -Body $body -ContentType 'application/json').choices[0].message.content

# 3. Runner online
gh api repos/AndrewMFlick/AutomationDemo/actions/runners --jq '.runners[] | {name, status, labels: (.labels | map(.name))}'

# 4. WSL responsive
wsl -d Ubuntu-24.04 -- echo "wsl ok"

# 5. iPhone on same Wi-Fi and Home app open
# (physically verify)
```

### Physical setup

- Laptop plugged in (Foundry Local on NPU is power-hungry)
- Windows Focus Assist ON — no Teams pings mid-demo
- 2 PowerShell windows pre-opened: one for main demo, one for `foundry model load` swaps
- Browser open to `https://github.com/AndrewMFlick/AutomationDemo/issues`
- Browser tab: `https://github.com/AndrewMFlick/AutomationDemo/actions`
- iPhone unlocked, Home app open, Wi-Fi confirmed matching laptop
- Backup video of Beat D iPhone pairing on a second device in case live pairing fails

---

## The four beats

### Beat A — Cost story (Foundry Local)

**Duration:** ~60 seconds  
**What lands:** "Zero tokens spent. Ran on my NPU. Only Microsoft can do this."

#### On-screen steps

Open a PowerShell in the repo folder:

```powershell
cd "$env:USERPROFILE\OneDrive - Microsoft\Documents\GitHub\AutomationDemo"

# Confirm env vars visibly - proves offline mode
Get-ChildItem env: | Where-Object Name -like 'COPILOT_*'
```

Highlight three lines to the audience:
- `COPILOT_PROVIDER_BASE_URL = http://localhost:5273/v1`
- `COPILOT_OFFLINE = true`
- `COPILOT_MODEL = qwen2.5-coder-7b-instruct-openvino-npu:4`

Then run:

```powershell
copilot
```

At the interactive prompt:
```
> summarize reference/HAP-NodeJS/src/lib/Accessory.ts in 3 bullets
```

#### Talk track

> "I'm running Copilot CLI right now, but look at these environment variables. `COPILOT_OFFLINE=true` means it physically can't call GitHub-hosted models. `COPILOT_PROVIDER_BASE_URL=http://localhost:5273` means every request goes to Foundry Local — a Microsoft runtime — on my NPU. Zero dollars per token. Zero data leaves this machine. And for the ~60% of a port like this that's mechanical grep-and-summarize work, a 7B open-weight model is more than enough. Frontier models are for problems that need frontier reasoning. This isn't one of them."

#### If it hangs

Say: "First inference is a warm-up — the NPU is compiling the graph. Normal for a fresh boot. Let's queue Beat B while this catches up."

Move to Beat B. Come back when the response lands.

---

### Beat B — Model routing (Opus for crypto)

**Duration:** ~90 seconds  
**What lands:** "Frontier models when you need them. Assigned like a teammate."

#### On-screen steps

Switch to browser tab with GitHub Issues open. Point at the issue labeled `model:opus`:

```
Issue #3: SRP Pair-Setup + ChaCha20-Poly1305 session
Labels: area:crypto, area:pairing, model:opus, compliance, security-review-required
```

Back in PowerShell:

```powershell
gh issue edit 3 --add-assignee "@copilot"
```

Refresh the issues tab. Show the "Copilot is working on this" indicator.

Open the Actions tab in a new browser window to show the coding agent running.

#### Talk track

> "For the SRP pairing handshake — the byte-exact crypto — I want Opus, not qwen. Watch this. I'm assigning this issue to `@copilot` the way I'd assign it to a human engineer. It's now running as a coding agent on GitHub Actions, not on my machine. It planned the work, opened a branch, and it'll come back with a pull request. Roughly $0.30 of frontier-model tokens. I'm happy to spend it here because getting the SRP salt or the ChaCha20 nonce wrong means Apple devices silently reject the accessory. **This is where the model spend actually earns its cost.** Everything else stays local."

#### Optional visual

Show the `.github/copilot-instructions.md` file briefly:

```
## Model routing - respect issue labels
- `model:foundry-local` -> Runs on Foundry Local. Mechanical work.
- `model:codex`   -> TLV8 and well-patterned wire code.
- `model:opus`    -> SRP, ChaCha20-Poly1305, X25519/Ed25519. Byte-exact.
- `model:fine-tuned` -> Adversarial spec review, air-gapped, Foundry Local.
```

> "Repo-level instructions telling Copilot what to route where. This is the routing policy in code."

---

### Beat C — Security story (air-gapped review)

**Duration:** ~90 seconds  
**What lands:** "The HAP spec never left this machine. And CI enforced it."

#### On-screen steps

Open the PR that Copilot opened for the SRP work in Beat B (or a pre-baked one you prepared).

Add the `security-review-required` label:

```powershell
gh pr edit <pr-number> --add-label security-review-required
```

Switch to the Actions tab. Show the `local-review-attestation` job starting on your **self-hosted runner** (labeled `foundry-local`).

Highlight two log lines from the CI job:

```
Verify offline mode + local reviewer model
  COPILOT_OFFLINE=true  ✓
  hap-spec-reviewer-v1 available on http://localhost:5273/v1  ✓

Adversarial review
  HAP §5.6.6.1: SRP salt must be 16 bytes.  Implementation: ✓
  HAP §5.6.6.2: SRP-6a proof M1 = H(...).   Implementation: ✓
```

#### Talk track

> "Apple's HAP specification is under a Non-Commercial license. You legally can't send big chunks of it to a third-party inference endpoint. So the adversarial spec reviewer runs on Foundry Local — on my machine — with `COPILOT_OFFLINE=true`. And CI enforces that. This job is running on a **self-hosted runner** that I registered with the `foundry-local` label. It literally cannot execute on GitHub-hosted infrastructure. The Apple spec never left my box. The code never left my box. And the review still gated the merge. **No other AI dev platform can offer this shape today.** Not Cursor. Not Claude Code. Not Codex CLI standalone. Because the local runtime piece — Foundry Local — is a Microsoft asset that ships with Windows."

#### The regulated-customer transition

If this is a regulated-vertical audience (defense, healthcare, finance, sovereign), pivot here:

> "This isn't just cool for a HAP demo. This is **the pattern** for every regulated customer that told your account team Copilot 'wasn't an option.' Data residency, air-gapped inference, on-prem model governance — same architecture. One `winget install`, four environment variables, official Microsoft docs."

---

### Beat D — WSL runtime close (iPhone pairs)

**Duration:** ~60 seconds  
**What lands:** "Real device. Real Rust. Real WSL. Not a video."

#### On-screen steps

Switch to a WSL terminal (you should have this pre-opened):

```bash
cd ~/demo/AutomationDemo
./target/release/light-bulb
```

Expected output:
```
HomeKit accessory: hap-rs-lightbulb
Setup code: 031-45-154
Advertising _hap._tcp.local on port 51826
Waiting for controller...
```

Pick up the iPhone. Home app → "Add Accessory" → "More options..." → tap the `hap-rs-lightbulb` that appears → enter setup code `031-45-154`.

Wait for pairing to complete (~10 seconds).

Toggle the light on/off from the phone.

#### Talk track

> "Cargo built this into a Rust binary. It's running in WSL on this same Windows box — mirrored networking so the phone can see it on the LAN. mDNS advertisement, HAP-HTTP protocol, SRP pairing — all Rust that Copilot wrote, with the model routing you just watched. Home app on the phone. Add accessory. Setup code. Pairing... [pause for pairing]. Done. Watch the light."

Toggle the light. **Silence for two seconds. Let the audience process.** Then:

> "That's the demo. From TypeScript to a Rust binary paired to an Apple device — routed across GitHub Copilot's model fleet, with the security-sensitive work never leaving my machine. **This is the only stack that does this today.**"

#### If pairing fails live

Pre-record a 30-second video of successful pairing. If Beat D fails during the demo, say:

> "Wi-Fi is fighting me on stage — happens with mDNS in conference centers. Here's the recorded run from this morning."

Play video. Land the same closing line.

---

## Closing talk track

Three bullets for the recap email / stakeholder debrief:

1. **Cost:** 60%+ of PRs on this port were driven by a 7B open-weight model running on-device via Foundry Local — zero token cost, no external inference — reserving frontier-model spend for the ~30% of work (crypto + review) that actually needs it.

2. **Security:** The adversarial HAP spec reviewer is a fine-tuned open-weight model, running fully air-gapped via Foundry Local + `COPILOT_OFFLINE=true`. Apple's spec and our source never left the developer's Copilot+ PC — and CI enforces it as a merge gate.

3. **Only Microsoft:** GitHub Issues → Copilot coding agent → BYOM model routing → Foundry Local on Windows/WSL → runtime on the same box. Cursor can't do it. Codex CLI can't do it. GitHub + Foundry Local can.

---

## Q&A prep

### "How much did Copilot cost you for this port?"

> "I haven't priced it exactly, but the routing model means the mechanical majority — probably 60–70% of PRs — costs nothing per token because it ran locally. Only the ~30% that touched crypto or spec review consumed frontier tokens. Rough back-of-envelope: maybe $8–12 in frontier spend for what would've been $30–40 if I'd routed everything to Opus."

### "Is Foundry Local actually production-ready?"

> "It's in public preview and ships via `winget install Microsoft.FoundryLocal`. It's the same runtime powering some Windows AI APIs. For a developer inference target — which is what this demo shows — it's ready today. For serving production customer traffic at scale, use Azure OpenAI or Azure AI Foundry."

### "What about the Apple certification for HAP?"

> "Right — HAP-NodeJS isn't Apple MFi-certified either. This is a developer/enthusiast implementation. The demo is about the *engineering* process, not shipping certified HomeKit accessories. That said, the same routing pattern would apply to a certified port; you'd just gate compliance more tightly."

### "Can I run this without a Copilot+ PC?"

> "Yes — Foundry Local auto-picks the best variant for your hardware. NPU on Copilot+ PC, discrete GPU on gaming laptops, CPU everywhere else. CPU is slower but functional. The story doesn't change."

### "How is this different from Ollama?"

> "Ollama is great and it's supported by Copilot CLI too — same env-var pattern. Foundry Local is the Microsoft-supported equivalent: signed, ships via winget, has an OpenAI-compatible endpoint, and integrates with Windows ML for hardware acceleration. Pick the one your enterprise policy allows. The routing story works with either."

### "What about GitLab Duo or AWS Q Developer?"

> "Neither has this end-to-end pattern. GitLab Duo doesn't do coding-agent + BYOM local routing. Q Developer is cloud-only. This is a first-party GitHub + Windows story."

### "Can we do this for our team's internal codebase?"

> "Yes. The pattern is repo-agnostic. The routing policy lives in `.github/copilot-instructions.md`, the CI enforcement lives in `.github/workflows/*.yml`, and Foundry Local is per-developer. Enterprise-scale, we'd add centralized BYOK via the GitHub Copilot Enterprise admin controls."

---

## Troubleshooting on stage

### Foundry Local won't start

```powershell
cd C:\   # get out of OneDrive
Get-Process | Where-Object { $_.ProcessName -like '*foundry*' } | Stop-Process -Force
Get-NetTCPConnection -LocalPort 5273 -ErrorAction SilentlyContinue |
    ForEach-Object { Stop-Process -Id $_.OwningProcess -Force }
foundry service start
Start-Sleep -Seconds 5
foundry service status
```

### Model not loaded

```powershell
foundry model load qwen2.5-coder-7b
(Invoke-RestMethod http://localhost:5273/v1/models).data | Select-Object id
```

### Copilot returns 400

Almost always the model ID or token limits. Full ID with variant suffix required:

```powershell
$env:COPILOT_MODEL = 'qwen2.5-coder-7b-instruct-openvino-npu:4'
$env:COPILOT_PROVIDER_MAX_PROMPT_TOKENS = '3200'
$env:COPILOT_PROVIDER_MAX_OUTPUT_TOKENS = '500'
copilot --model "qwen2.5-coder-7b-instruct-openvino-npu:4"
```

### Copilot returns "transient API error, retrying"

Wait 60 seconds. First inference is NPU warmup. If it never lands:

```powershell
foundry service status                        # is service alive?
foundry model load qwen2.5-coder-7b           # is model loaded?
setx COPILOT_PROVIDER_TIMEOUT_MS "120000"     # bump the timeout
```

### Runner offline

```powershell
gh api repos/AndrewMFlick/AutomationDemo/actions/runners
# If empty or state != online:
Start-Service actions.runner.AndrewMFlick-AutomationDemo.foundry-local-*
```

### iPhone can't see the accessory

- Confirm laptop and phone on same Wi-Fi SSID (not guest network)
- WSL mirrored networking on: `Get-Content $env:USERPROFILE\.wslconfig`
- Firewall: rules `HAP-mDNS` (UDP 5353) and `HAP-HTTP` (TCP 51826) present
- Restart the light-bulb binary if it's been running >5 minutes

If any of these fail on stage, cut to the pre-recorded video.

---

## Setup reference (if you need to rebuild)

Run in order in an **elevated** PowerShell:

```powershell
cd $env:USERPROFILE\Downloads
Get-ChildItem *.ps1 | Unblock-File

# 1. Everything - Foundry, WSL, gh, Copilot CLI, repo, GitHub, labels, issues
.\setup-demo.ps1 -GitHubUser AndrewMFlick

# 2. Verify what worked
.\02-verify-setup.ps1 -GitHubUser AndrewMFlick

# 3. Runner for Beat C
.\03-register-runner.ps1 -GitHubUser AndrewMFlick

# 4. Codespace (optional - if demoing from Codespaces)
.\04-open-codespace.ps1 -GitHubUser AndrewMFlick

# 5. WSL runtime for Beat D
.\05-wsl-runtime.ps1 -GitHubUser AndrewMFlick -LaunchAccessory
```

---

## Configuration reference

### Environment variables (persistent, user scope)

Use `setx` — it's the most portable way to set persistent env vars from PowerShell.

```powershell
setx COPILOT_PROVIDER_BASE_URL          "http://localhost:5273/v1"
setx COPILOT_PROVIDER_TYPE              "openai"
setx COPILOT_MODEL                      "qwen2.5-coder-7b-instruct-openvino-npu:4"
setx COPILOT_OFFLINE                    "true"
setx COPILOT_PROVIDER_MAX_PROMPT_TOKENS "3200"
setx COPILOT_PROVIDER_MAX_OUTPUT_TOKENS "500"
setx COPILOT_PROVIDER_TIMEOUT_MS        "120000"
```

Open a new PowerShell after setting these. To load into the current session without reopening:

```powershell
$env:COPILOT_PROVIDER_BASE_URL          = "http://localhost:5273/v1"
$env:COPILOT_PROVIDER_TYPE              = "openai"
$env:COPILOT_MODEL                      = "qwen2.5-coder-7b-instruct-openvino-npu:4"
$env:COPILOT_OFFLINE                    = "true"
$env:COPILOT_PROVIDER_MAX_PROMPT_TOKENS = "3200"
$env:COPILOT_PROVIDER_MAX_OUTPUT_TOKENS = "500"
$env:COPILOT_PROVIDER_TIMEOUT_MS        = "120000"
```

### Foundry Local persistent settings

```powershell
foundry service set --port 5273              # pin port so env vars don't drift
foundry service set --autoload qwen2.5-coder-7b  # load on service start
foundry service set --model-ttl 0            # never auto-unload
```

### Repo locations

| Purpose | Path |
|---|---|
| Windows workspace | `C:\Users\anflick\OneDrive - Microsoft\Documents\GitHub\AutomationDemo` |
| Cargo build target (out of OneDrive) | `%LOCALAPPDATA%\cargo-target\AutomationDemo` |
| WSL workspace | `~/demo/AutomationDemo` |
| GitHub Actions runner | `%LOCALAPPDATA%\gh-runner-AutomationDemo` |
| Foundry Local models | `%LOCALAPPDATA%\Microsoft\FoundryLocal\` |
| Reference HAP-NodeJS clone | `<workspace>/reference/HAP-NodeJS` |

### GitHub repo

**Owner/Name:** `AndrewMFlick/AutomationDemo`  
**Coding agent:** enabled at `https://github.com/AndrewMFlick/AutomationDemo/settings/copilot`  
**Runner label:** `foundry-local`

### Model routing table

| Task | Model | Where it runs | Cost |
|---|---|---|---|
| Exploration, summarization | qwen2.5-coder-7b | Foundry Local (NPU) | $0 |
| Boilerplate Rust translation | qwen2.5-coder-7b | Foundry Local (NPU) | $0 |
| TLV8 wire encoding | Codex | GitHub-hosted | ~$0.05/PR |
| SRP + ChaCha20-Poly1305 crypto | Opus 4.8 | GitHub-hosted | ~$0.30/PR |
| Adversarial HAP spec review | hap-spec-reviewer-v1 (fine-tuned) | Foundry Local (offline) | $0 |

---

## Post-demo actions

Within 24 hours:

- [ ] Send recap email using the [Closing talk track](#closing-talk-track) three bullets
- [ ] Log opportunity/customer follow-ups in your CRM
- [ ] File any bugs against Foundry Local or Copilot CLI you hit during prep
- [ ] Update this doc with anything that broke or landed unexpectedly well
- [ ] Post a short internal Teams video (2 min) if the demo went well — this is the kind of asset that spreads to other PMMs

---

## Version history

| Date | Change |
|---|---|
| 2026-07-05 | Initial demo script for AutomationDemo repo. HAP-NodeJS → Rust port narrative. Four beats defined. |
