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
| 2. Required models loaded | `foundry model load qwen2.5-coder-1.5b-instruct-generic-gpu:4` |
| 3. COPILOT_* env vars | Open a fresh PowerShell session |
| 5. gh auth | `gh auth login` |
| 9. Runner online | Run `03-register-runner.ps1` |

### 30 minutes before demo

```powershell
# 1. Foundry Local up and model loaded
cd C:\
foundry service status
foundry model load qwen2.5-coder-1.5b-instruct-generic-gpu:4

# 2. Quick inference test - verifies the whole stack
$body = '{"model":"qwen2.5-coder-1.5b-instruct-generic-gpu:4","messages":[{"role":"user","content":"hi"}],"stream":false,"max_tokens":20}'
(Invoke-RestMethod http://localhost:5273/v1/chat/completions -Method Post -Body $body -ContentType 'application/json').choices[0].message.content

# 3. Runner online
gh api repos/AndrewMFlick/AutomationDemo/actions/runners --jq '.runners[] | {name, status, labels: (.labels | map(.name))}'

# 4. WSL responsive
wsl -d Ubuntu-24.04 -- echo "wsl ok"

# 5. iPhone on same Wi-Fi and Home app open
# (physically verify)
```

### Physical setup

- Laptop plugged in (Foundry Local on the GPU is power-hungry)
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
**What lands:** "The Copilot CLI you already know — running on a model on *my box*. Zero tokens. Only Microsoft can do this."

> **Read this once — it explains the model choice (all verified on this box).**
> The demo runs the **real `copilot` CLI against a local model**, offline. The
> setup that actually works on a Copilot+ PC with a 2 GB Intel Arc iGPU is
> specific, because three tempting options are dead ends:
>
> | Option | Result | Why |
> |---|---|---|
> | `qwen2.5-coder-7b` **NPU** (`…-npu:4`) | ❌ CLI can't load | Every OpenVINO **NPU** build is compiled to a hard **~4,224-token** window (verified: `400 "supports at most 4224 completion tokens"`). Copilot CLI's system prompt + tool schemas exceed that, so it reports `Static context is using 563% of available input tokens`. No env-var fixes it. |
> | `qwen2.5-coder-7b` **GPU** (4.8 GB) | ❌ gibberish | The 4.8 GB build spills the 2 GB Arc VRAM. |
> | any **CPU** build | ⚠️ too slow | Coherent but ~45 s per 4 K tokens; the ~18 K harness is minutes/turn. |
>
> **The build that works: `qwen2.5-coder-1.5b-instruct-generic-gpu:4`** — a
> ~1 GB coder model on the **DirectML GPU** runtime. It fits the Arc, stays
> coherent, and answers in ~40 s. Three rules make it reliable:
> 1. **`--stream off`** — Foundry Local's stream omits `finish_reason`, which
>    otherwise makes the CLI retry 5× and duplicate output.
> 2. **Trim the harness** (`--disable-builtin-mcps --no-custom-instructions
>    --available-tools=view`) and cap `MAX_PROMPT_TOKENS=7000` — the GPU holds
>    ~7–8 K prompt tokens before the WebGPU buffer overflows.
> 3. **Keep the prompt self-contained.** A 1.5 B model is unreliable at
>    orchestrating tool-calls live, so ask a direct question rather than making
>    it agentically read a 21 K-token file. (For a file summary, use the
>    `beat-a-local-summary.ps1` fallback below, which feeds a right-sized
>    excerpt straight to the endpoint.)
>
> Also: before showtime, make sure other models aren't pinned in memory — the
> demo sets `model-ttl 0`, and a full memory causes `bad allocation` errors and
> service crashes. `beat-a-local-cli.ps1 -FreeMemory` handles it.

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
- `COPILOT_MODEL = qwen2.5-coder-1.5b-instruct-generic-gpu:4`

Then run the Beat A launcher — it loads the local model, sets the offline env,
and runs the **real Copilot CLI** against it:

```powershell
.\beat-a-local-cli.ps1
```

Expected output (abridged):

```
Beat A - GitHub Copilot CLI on a LOCAL model (offline, $0.00)
COPILOT_OFFLINE           = true
COPILOT_PROVIDER_BASE_URL = http://localhost:5273/v1
COPILOT_MODEL             = qwen2.5-coder-1.5b-instruct-generic-gpu:4
Prompt: In exactly 3 bullets, explain what a HomeKit Accessory is and why a Rust port benefits from memory safety.
 - A HomeKit Accessory is a device that connects to Apple HomeKit...
 - Rust provides memory safety through ownership, borrowing, and lifetimes...
 - By using Rust, developers write safer code that avoids buffer overflows...
Answered by qwen2.5-coder-1.5b-instruct-generic-gpu:4 on the local GPU in 46.2s  |  $0.00 - zero tokens billed, fully offline
```

#### Talk track

> "This is the Copilot CLI — the exact same tool. But look at these environment variables. `COPILOT_OFFLINE=true` means it physically can't call a GitHub-hosted model. `COPILOT_PROVIDER_BASE_URL=http://localhost:5273` means every token is generated by Foundry Local — a Microsoft runtime — on the GPU in *this* laptop. Same CLI, same workflow, but the model is on my box. Zero dollars per token, zero data leaving the machine. For the ~60% of a port like this that's mechanical grep-and-summarize work, a small open-weight coder model on-device is enough. Frontier models are for problems that need frontier reasoning — which is exactly Beat B."

#### If it's slow

Say: "It's grinding entirely on-device — no cloud, no tokens. First call also warms the GPU graph. Let's queue Beat B while it finishes." Move to Beat B and come back.

#### Optional: summarize a real HAP file locally

To summarize actual reference source on-device (endpoint call, not the agent —
reliable regardless of model tool-use), run:

```powershell
.\beat-a-local-summary.ps1                       # summarizes head of Accessory.ts
.\beat-a-local-summary.ps1 -File reference/HAP-NodeJS/src/lib/util/uuid.ts
```

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

1. **Cost:** 60%+ of PRs on this port were driven by an open-weight coder model running on-device via Foundry Local — zero token cost, no external inference — reserving frontier-model spend for the ~30% of work (crypto + review) that actually needs it.

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

> "Yes — and you don't need a Copilot+ PC at all. Foundry Local auto-picks the best variant for your hardware: discrete or integrated GPU where available, CPU everywhere else. One caveat learned building this demo: the **NPU** builds are capped at a ~4K-token window, too small to host the Copilot CLI harness — so for *driving the CLI* you want the **GPU** (or CPU) build. The NPU is still great for direct endpoint inference. Either way the cost story doesn't change: it runs on-device for zero token cost."

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
foundry model load qwen2.5-coder-1.5b-instruct-generic-gpu:4
(Invoke-RestMethod http://localhost:5273/v1/models).data | Select-Object id
```

### Beat A: Copilot CLI can't load / "563% of available input tokens"

You are pointed at an **NPU** model. Every OpenVINO NPU build is capped at
**~4,224 tokens** — too small for the CLI harness. Switch to the **GPU** build:

```powershell
$env:COPILOT_MODEL = 'qwen2.5-coder-1.5b-instruct-generic-gpu:4'
foundry model load qwen2.5-coder-1.5b-instruct-generic-gpu:4
```

Then run `.\beat-a-local-cli.ps1`. If it still overflows, the harness is too big
for the 2 GB GPU window — lower the budget: `$env:COPILOT_PROVIDER_MAX_PROMPT_TOKENS='6000'`.

### Beat A: output is duplicated / "missing finish_reason"

Foundry Local's streaming omits `finish_reason`. Always pass **`--stream off`**
(the launcher script already does).

### Beat A: "bad allocation" 500 or the service crashes

Memory pressure — other models are pinned in RAM (`model-ttl 0`). Free them:

```powershell
.\beat-a-local-cli.ps1 -FreeMemory     # unloads other resident models first
```

### Beat A: model emits raw tool-call JSON / gibberish

The 1.5B model is unreliable at agentic tool-use. Keep the prompt **self-contained**
(the launcher does). For an actual file summary, use the endpoint fallback:

```powershell
.\beat-a-local-summary.ps1 -File reference/HAP-NodeJS/src/lib/util/uuid.ts
```

Sanity-check the model directly (should return a short summary in a few seconds):

```powershell
$body = '{"model":"qwen2.5-coder-1.5b-instruct-generic-gpu:4","messages":[{"role":"user","content":"Summarize what a HomeKit accessory is in 2 bullets."}],"stream":false,"max_tokens":80}'
(Invoke-RestMethod http://localhost:5273/v1/chat/completions -Method Post -Body $body -ContentType 'application/json').choices[0].message.content
```

### Copilot returns "transient API error, retrying"

Wait 60 seconds. First inference warms the GPU graph. If it never lands:

```powershell
foundry service status                                        # is service alive?
foundry model load qwen2.5-coder-1.5b-instruct-generic-gpu:4  # is model loaded?
setx COPILOT_PROVIDER_TIMEOUT_MS "120000"                     # bump the timeout
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
setx COPILOT_MODEL                      "qwen2.5-coder-1.5b-instruct-generic-gpu:4"
setx COPILOT_OFFLINE                    "true"
setx COPILOT_PROVIDER_MAX_PROMPT_TOKENS "7000"
setx COPILOT_PROVIDER_MAX_OUTPUT_TOKENS "400"
setx COPILOT_PROVIDER_TIMEOUT_MS        "120000"
```

Open a new PowerShell after setting these. To load into the current session without reopening:

```powershell
$env:COPILOT_PROVIDER_BASE_URL          = "http://localhost:5273/v1"
$env:COPILOT_PROVIDER_TYPE              = "openai"
$env:COPILOT_MODEL                      = "qwen2.5-coder-1.5b-instruct-generic-gpu:4"
$env:COPILOT_OFFLINE                    = "true"
$env:COPILOT_PROVIDER_MAX_PROMPT_TOKENS = "7000"
$env:COPILOT_PROVIDER_MAX_OUTPUT_TOKENS = "400"
$env:COPILOT_PROVIDER_TIMEOUT_MS        = "120000"
```

> **Note on the model choice.** `…-generic-gpu:4` is the DirectML GPU build —
> the one that actually runs the Copilot CLI harness on a 2 GB Arc iGPU. Do
> **not** use `…-openvino-npu:4` (4,224-token cap — CLI won't load) or the 7B
> GPU build (spills VRAM → gibberish). Always launch the CLI with `--stream off`.

### Foundry Local persistent settings

```powershell
foundry model download qwen2.5-coder-1.5b-instruct-generic-gpu:4  # pull the DirectML GPU build (~1 GB)
foundry service set --port 5273              # pin port so env vars don't drift
foundry service set --autoload qwen2.5-coder-1.5b-instruct-generic-gpu:4  # load on service start
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
| Exploration, summarization | qwen2.5-coder-1.5b (GPU) | Foundry Local (GPU) | $0 |
| Boilerplate Rust translation | qwen2.5-coder-1.5b (GPU) | Foundry Local (GPU) | $0 |
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
