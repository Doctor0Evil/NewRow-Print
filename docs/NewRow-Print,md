# Neuroprint! for NewRow-Print!

## 1. Purpose and Scope

Neuroprint! is the **diagnostic** and explanatory surface for subjective neuromorphic state in NewRow‑Print!, designed to stay strictly non‑actuating while providing richly structured, human‑legible views over governed biophysical data.[file:12][file:7]

It sits on top of:

- BiophysicalEnvelopeSpec (EEG, HR/HRV, EDA, motion, etc.) with RoH ≤ 0.30 and non‑relaxing minsafe/maxsafe bounds.[file:5][file:6]
- Tree‑of‑Life (TREE assets BLOOD, OXYGEN, WAVE, DECAY, LIFEFORCE, POWER, FEAR, PAIN, etc.).[file:4]
- NATURE/BIOTREE/GOAL predicates as qualitative tags over TREE and envelope histories.[file:2]

Neuroprint! MUST NOT:

- Change CapabilityState, ConsentState, or RoH models.
- Modify envelopes, ReversalConditions, or PolicyStack.
- Introduce new reward, econet, or incentive signals.

It MAY:

- Read canonical snapshots (envelopes, Tree‑of‑Life views, RoH, capability).
- Emit additional, read‑only diagnostics (e.g., ROW, γ‑WAVE overload state).
- Serialize views into `.evolve.jsonl` / `.donutloop.aln` for audit and explanation.[file:4][file:7]

## 2. Inputs and Core Data Flow

Neuroprint! consumes only governed inputs that already exist in the stack:

- `BiophysicalEnvelopeSnapshot` (per epoch):
  - EEG bandpower (theta/alpha/beta/gamma), alpha‑CVE, HR/HRV, EDA, motion, respiration, gaze.[file:5]
  - Axis states and flags: `axisstate` (INFO/WARN/RISK), `envelopeinviolation`, `requiresdowngrade`, `requestcapabilitydowngrade`.[file:6]
- `TreeOfLifeView`:
  - 0.0–1.0 TREE assets including WAVE, DECAY, LIFEFORCE, POWER, FEAR, PAIN.[file:4]
- RoH and governance:
  - `rohscore.value`, `rohceiling` (0.30 for CapControlledHuman), capability/juristags/neurorights.[file:6][file:7]
- Epoch timing:
  - `epochindex`, `epochdursecs` for finite‑difference and windowed statistics.[file:4]

Data flow (read‑only):

1. Sovereignty core computes envelopes, RoH, and capability transitions.
2. Tree‑of‑Life computes TREE assets and diagnostics from snapshots.
3. Neuroprint! computes additional diagnostics (e.g., ROW, γ‑WAVE status, NATURE tags) and logs them into the same canonical streams.

No write path from Neuroprint! to capability, consent, RoH, or envelopes is permitted.

## 3. Diagnostic Constructs

### 3.1 WAVE and DECAY

- **WAVE**: Composite cortical activation asset from normalized EEG bandpower (alpha, beta, gamma) and alpha‑CVE; 0.0–1.0.[file:4]
- **DECAY**: RoH proximity, `DECAY = rohscore.value / 0.30`, clamped to [0.0, 1.0].[file:4][file:6]
- **LIFEFORCE**: Remaining safety budget, `LIFEFORCE = 1.0 − DECAY`.[file:4]

These are computed by Tree‑of‑Life and consumed by Neuroprint!.

### 3.2 Rate‑of‑Wave (ROW)

**Definition**

Rate‑of‑Wave (ROW) is a read‑only scalar quantifying how quickly the RoH‑normalized safety budget (DECAY/LIFEFORCE) is being consumed during high‑WAVE epochs.[file:33][file:2]

Given epochs of duration \(\Delta t\) seconds:

- Let `DECAY(t)` and `DECAY(t−1)` be consecutive DECAY values from Tree‑of‑Life.
- Let `WAVE(t)` be the concurrent WAVE value.
- Define a WAVE‑state predicate:
  - `WAVE_STATE(t) = true` if `WAVE(t) ≥ θ_wave`, with \(θ_{\text{wave}}\) chosen from envelope‑calibrated ranges (e.g., 0.5–0.7).[file:5][file:2]

Then, when:

- `WAVE_STATE(t)` is true, and
- RoH monotonicity and ceiling hold (`RoH_after ≥ RoH_before`, `RoH_after ≤ 0.30`),[file:6]

Neuroprint! defines:

\[
\text{ROW}(t) = \frac{\text{DECAY}(t) - \text{DECAY}(t-1)}{\Delta t}
\]

Properties:

- Signed: positive values indicate increasing DECAY (safety budget consumption), negative values indicate recovery.
- Read‑only: computed from logs, stored as diagnostics only; NEVER used to actuate or gate capabilities.

### 3.3 γ‑WAVE Envelope Diagnostics

Neuroprint! integrates γ‑band overload as an envelope‑aligned diagnostic:

- New axes in BiophysicalEnvelopeSpec (defined in their own shard, e.g., `gamma-wave-overload-envelope-v1`):[file:5]
  - `EEGBANDPOWER-GAMMA-FRONTAL`
  - `EEGBANDPOWER-GAMMA-POSTERIOR`
  - Optional `GAMMA-COHERENCE` / alpha–gamma coupling.
- Each axis specifies:
  - `minsafe`, `maxsafe`: 95–99th percentile ranges from rest vs high‑load EEG/MEG studies.[web:15][web:30]
  - `minwarn`, `maxwarn`: tighter inner bounds for early warnings.
  - `maxdeltapersec`: conservative slope limits based on fast‑band ramps into arousals/overloads.[web:13][web:16]

Envelope semantics:

- γ‑WAVE axes contribute to cognitiveload/sleeparousal RoH categories with fixed non‑negative weights.[file:5][file:6]
- WARN/RISK decisions obey:
  - Non‑relaxing minsafe/maxsafe.
  - Multi‑epoch hysteresis (`warnepochstoflag`, `riskepochstodowngrade`).[file:5]

Neuroprint! then uses:

- γ‑WAVE WARN/RISK flags.
- ROW(t) time series.
- TREE assets (POWER, FEAR, PAIN) and envelope states.

to label episodes such as:

- `NATURE::ROW_HIGH`
- `NATURE::GAMMA_OVERLOAD`
- `NATURE::OVERLOADED_ROW_SLEEP_RISK`

All such labels remain advisory and read‑only.

## 4. Linking ROW to Sleep and Cognitive Overload

Neuroprint! is the place where sleep disruption and cognitive overload hypotheses are formulated and tested, not enforced.

### 4.1 Sleep Disruption

Target outcomes (from actigraphy/PSG or validated proxies):

- Increased Wake After Sleep Onset (WASO) and more frequent awakenings.[web:31]
- Reduced Slow Wave Sleep (SWS) percentage and slow‑wave activity.[web:29][web:31]
- Elevated intra‑sleep beta/fast‑band power indicating hyperarousal.[web:31]

Diagnostic plan:

- Log per‑epoch ROW, WAVE, DECAY, LIFEFORCE, FEAR, PAIN, and sleep‑arousal envelope states.[file:5][file:33]
- Define high‑ROW episodes (e.g., ROW above a threshold for ≥N epochs under WAVE_STATE and elevated FEAR/PAIN).[file:33][file:2]
- Align evening high‑ROW episodes with same‑night sleep metrics (WASO, SWS%, intra‑sleep beta) and compute correlations or model fits.[web:31][file:5]

Neuroprint! MUST only surface these as explanatory statements (e.g., “high‑ROW surges clustered before bedtime were followed by elevated WASO and reduced SWS%”) and as evidence in fairness panels, never as triggers for automatic downgrades.

### 4.2 Cognitive Overload

Inputs:

- Cognitive‑load envelope WARN/RISK based on EEG (beta/γ, alpha‑CVE), HR/HRV, EDA.[file:5][file:6]
- Behavioral/ERP metrics where available (N‑back error, P300 latency).[web:21]
- TREE assets POWER, FEAR, PAIN.[file:4]

Diagnostic plan:

- Label high‑ROW episodes that co‑occur with cognitive‑load WARN/RISK, high POWER, high FEAR/PAIN, and prolonged P300 / elevated error.[file:33][file:2]
- Treat these as candidate overload events and log them as NATURE predicates, e.g., `NATURE::COGNITIVE_OVERLOAD_ROW`.[file:2]
- Use them only in reports and fairness panels (e.g., “subject experienced X overload episodes per hour with high ROW and envelope RISK”), not in PolicyStack or CapabilityTransitionRequest.

## 5. Governance and Invariants

Neuroprint! is governed by the same invariants as the rest of NewRow‑Print!:

- RoH ceiling: RoHafter ≥ RoHbefore and RoHafter ≤ 0.30 in CapControlledHuman; Neuroprint! MUST NOT circumvent or reinterpret these rules.[file:6][file:7]
- Capability/consent separation: All capability transitions and neuromorph evolution are governed by ALN kernels (`.rohmodel.aln`, `.stake.aln`, ReversalConditions, PolicyStack). Neuroprint! never introduces new transition logic.[file:3][file:7]
- Non‑actuation: No hardware control, no stimulation, no direct control over external systems. Logging and explanation only.[file:7][file:12]

Explicit constraints:

- ROW and γ‑WAVE:
  - MUST be declared as view‑only, non‑policy, non‑reward metrics in their ALN sections.
  - MAY influence only:
    - Logging into `.evolve.jsonl` / `.donutloop.aln`.
    - Advisory HUD/AI‑chat explanations.
    - Diagnostic and fairness analysis.
- ReversalConditions:
  - `allowneuromorphreversal = false` by default; any reversals still require `explicitreversalorder`, `nosaferalternative`, plus full PolicyStack and stake checks, regardless of ROW or γ‑WAVE diagnostics.[file:3][file:7]

## 6. Serialization and Integration

Neuroprint! outputs are serialized as structured fields inside the same canonical streams used by Tree‑of‑Life and envelopes:

- `.evolve.jsonl`:
  - Per‑epoch lines that may include:
    - `tree_of_life_view` (TREE assets).
    - `envelope_states` (per‑axis INFO/WARN/RISK).
    - `neuroprint` object, e.g.:

```json
{
  "epochindex": 1234,
  "rohscore": 0.21,
  "tree_of_life_view": { "wave": 0.72, "decay": 0.70, "lifeforce": 0.30, "power": 0.65, "fear": 0.58, "pain": 0.51 },
  "envelopes": {
    "cognitive_load": { "state": "RISK" },
    "sleep_arousal": { "state": "WARN" }
  },
  "neuroprint": {
    "row": 0.015,
    "row_state": "HIGH",
    "gamma_wave_state": "RISK",
    "nature_tags": ["ROW_HIGH", "COGNITIVE_OVERLOAD_ROW"]
  }
}
```

- `.donutloop.aln`:
  - Remains the hash‑linked ledger of decisions; Neuroprint! fields are attached only as additional context in decision records and MUST NOT alter decisions or hashes.[file:4][file:7]

Visualization and AI‑chat:

- HUDs and AI‑agents read Neuroprint! fields from these logs to explain system state, workload, and fairness (e.g., dashboards showing ROW time courses and sleep/overload correlations), but never call Neuroprint! as a control API.

---

This file defines Neuroprint! as a precise, non‑fictional, and non‑actuating diagnostic layer for NewRow‑Print!, with ROW and γ‑WAVE as key internal workload and sleep‑risk constructs that are mathematically rigorous, biophysically grounded, and tightly constrained by RoH and sovereignty governance.[file:12][file:33]
