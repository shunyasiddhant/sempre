# Paper (working draft outline)

## Working title
The LLM Proposes, the Type System Disposes: Verified Semantic Parsing for Reliable Language Agents

## Claim
LLM-only agents are unreliable because nothing constrains their actions to be well-formed, permitted, or correct. SEMPRE-style typed, executable logical forms supply the missing constrain/execute/verify/remember layers. We quantify each layer, including its limits.

## Contributions (target)
1. A typed Formula IR (parser, type checker, executor, verifier) for SEMPRE logical forms, as a reusable verifier for LLM program proposals.
2. A layer-by-layer measurement of verification: syntax, type, execution, spec/denotation — what each layer can and cannot reject.
3. (Step 5) LLM+verifier vs LLM-only vs classic FloatingParser on a living SEMPRE domain (Overnight): invalid-program rate and denotation accuracy.
4. (Step 6) Macro caching of verified programs: search/latency reduction.

## Preliminary result — Experiment 01 (synthetic, NOT a paper result yet)
Noisy proposer: correct formula + syntax/type/unbound/semantic mutants, shuffled; pick first accepted candidate; 200 arithmetic tasks, seed 13.

    none                wrong-accepted 166/200
    parse               wrong-accepted 159/200
    parse+type          wrong-accepted 136/200
    parse+type+exec     wrong-accepted 136/200
    oracle (denotation) wrong-accepted   0/200

Reading: static layers prune invalid programs (166 to 136) but are blind to semantic mutants (wrong operator type-checks and executes fine). Conclusion: a specification layer (tests/constraints/denotation) is not optional — it is the verifier. This shapes Contribution 2.

## Experiments still needed (honesty checklist)
- Real proposer (LLM) and real domain, not synthetic mutants.
- Classic-parser baseline on the same domain (Step 0/3).
- Latency/cost per accepted program, per layer.
- Spurious-program analysis: right denotation, wrong derivation.

## Related work anchors (to complete via Semantic Scholar)
Berant et al. EMNLP 2013; Berant & Liang ACL 2014; Pasupat & Liang ACL 2015; Wang, Berant & Liang ACL 2015; Wang et al. ACL 2017; Zhang, Pasupat & Liang EMNLP 2017; modern text-to-SQL, tool-calling, code generation with execution feedback, RL with verifiers, library learning (DreamCoder).
