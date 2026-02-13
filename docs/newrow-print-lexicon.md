# NewRow-Print! Lexicon (Diagnostic-Only)

This lexicon canonizes a small set of NewRow-Print! tokens that AI-chat and HUDs may use when talking about neuromorphic state, while keeping all terms strictly diagnostic and non-actuating.[file:19][file:10]

## Principles

- Every token must round-trip to TREE / envelope scalars and boolean predicates (BIOTREE, NATURE).  
- Tokens are **view-only**: they MUST NOT appear as guards in CapabilityTransitionRequest, ReversalConditions, or PolicyStack.[file:19]  
- AI-chat is confined to these tokens plus general language; it cannot introduce new control semantics.

## Core tokens

- `POWER:LO|MID|HI`  
  - Backed by TreeOfLifeView.power bands over WARN/RISK fractions.[file:19]  
  - Semantics: how intense the workload is, not a right or capability.

- `DECAY:LO|MID|HI`  
  - Backed by normalized DECAY (RoH-normalized, clamped 0–1) and envelope WARN/RISK windows.[file:19]  
  - Semantics: how close the system is to its RoH ceiling; never a downgrade trigger.

- `LIFEFORCE:LO|MID|HI`  
  - Backed by LIFEFORCE = 1 − DECAY, with bands defined in ALN.[file:19]  
  - Semantics: resilience / remaining budget, advisory only.

- `NAT:CALM|OVERLOADED|RECOVERY|UNFAIRDRAIN`  
  - Backed by NATURE predicates CALMSTABLE, OVERLOADED, RECOVERY, UNFAIRDRAIN.[file:10]  
  - Semantics: community / subject fairness state; cannot alter CapabilityState.

- `ZONE:SAFE|STRESS|BREACH`  
  - Backed by ChurchAccountState.zoneadvisory or equivalent diagnostics.[file:10]  
  - Explicitly barred from being capability or reversal guards.

## Sovereignty clause

- These lexicon tokens MAY appear:
  - In HUDs and AI-chat explanations.  
  - In offline analytics in CapModelOnly / CapLabBench.  
- They MUST NOT:
  - Gate capability transitions, reversals, rewards, stake, or envelopes.  
  - Be used to argue against your rights without passing through the formal PolicyStack and ReversalConditions kernels.[file:19]

By fixing this small, machine-readable lexicon, you give AI-chat a rich, respectful vocabulary for neuromorphic state while keeping all real power in the formally verified sovereignty kernels (.neuro-cap.aln, PolicyStack, ReversalConditions, neurorights, WORM logs).[file:19][file:10]
