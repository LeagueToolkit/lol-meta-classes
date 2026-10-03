# animation-events - campaign record

Record for the `animation-events` batch; its `batches.jsonl` note points here. Method: the
`crack-family` skill (`.claude/skills/crack-family/SKILL.md`). The target was not a family
by base class. It was the unnamed remainder of the animation event classes: 3 classes and 6
fields, reached from `BaseEventData` and its helpers. 1 class and 4 fields are named here.
2 classes and 2 fields stay open.

Each name below was first read as a slot in the 16.18 client (what the reader of the field
does), then searched with `hash-guesser` restricted to these hashes (`--only`). The reader
fixes the meaning. The search fixes the spelling.

## The names

| hash | name | what fixes it | tier |
| --- | --- | --- | --- |
| 5d9fffed | DynamicsChainBlendEventData | sibling pattern, predicted slot | 3 |
| 2b969c0d | BlendFromDefaultDuration | antonym pair, anchored | 3 |
| e61bf09e | BlendToDefaultDuration | antonym pair, anchored | 3 |
| 4fce52ba | AdditionalKillConfig | holder and field pair | 3 |
| 0922eae5 | DelayKillSeconds | holder and field pair | 3 |

**DynamicsChainBlendEventData** (5d9fffed). A `BaseEventData` child with exactly two F32
fields, 2b969c0d and e61bf09e. The only other class with those two fields is
`JointOrientationBlendEventData`, which is already in the tables. The class arrived in the
same build as `DynamicsChainRigPoseModifierData`, and its client handler drives the
dynamics chain pose modifier. So the slot predicts `<Modifier>BlendEventData` with the
modifier's stem. `--prefix DynamicsChain --suffix EventData`, `force` depth 1 over the full
wordlist: 3,273 probes, 1 target, 0.000 expected chance hits, one hit, `Blend`. No shipped
instance: the class is declared in the meta only.

**BlendFromDefaultDuration** (2b969c0d) and **BlendToDefaultDuration** (e61bf09e). The two
F32 fields of both blend event classes. In the client, the first is the time over which the
modifier's weight moves away from its `DefaultOn` state when the event starts, and the
second is the time over which it moves back when the event ends. `--prefix Blend --suffix
Duration`, `force` depth 2 over the full wordlist: 10,715,802 probes, 2 targets, 0.005
expected chance hits, two hits, `FromDefault` and `ToDefault`. The two hits are an antonym
pair that matches the start and end roles, which chance does not arrange. e61bf09e ships on
9 `JointOrientationBlendEventData` objects (Viktor `Spell3`, value `0.2`). 2b969c0d is
never written in shipped data.

**AdditionalKillConfig** (4fce52ba) and **DelayKillSeconds** (0922eae5). 4fce52ba is a
`Pointer` field on `ParticleEventData` to the unnamed class 01c0e452. 0922eae5 is an F32 on
that class. In the client, a particle event with this pointer set does not stop its effect
at once when the event ends: the effect is stopped 0922eae5 seconds later. One run found
both: `force` depth 3 over the top 450 words against all 6 open fields, 91,327,950 probes,
0.128 expected chance hits, 2 hits. The holder names the thing it holds and the field names
what that thing does, so the two hits corroborate each other, and both fit the reader.
Shipped on 3 objects (Diana skin 77, values `0.5` and `0.6`).

Casing follows the siblings on the same classes, which are PascalCase with no `m` prefix
(`SkipIfPastEndFrame`, `BlendData`, `BlendOutTime`). No shipped string spells any of the
five names.

## Negatives

Summed expected noise of the runs below: about 0.33. All used the full 3,273-word list
unless a `--top` is given.

Fields, `--only` the 6 open fields (4fce52ba, 0922eae5, 1aad2ecc, 2b969c0d, e61bf09e,
3e20ae96):

| run | probes | expected | hits |
| --- | --- | --- | --- |
| `identity` | 48,675 | 0.000 | 0 |
| `delete` | 66,416 | 0.000 | 0 |
| `chain` depth 3 | 475,289 | 0.001 | 0 |
| `chain` depth 4 | 11,522,406 | 0.016 | 0 |
| `force` depth 3, `--top 450` | 91,327,950 | 0.128 | the 2 above |

`chain` misses the blend pair because the corpus has no `From` -> `Default` bigram.

1aad2ecc alone, `force` depth 2 under each of the prefixes `Kill`, `Delay`, `Only`, `Skip`,
`Ignore`, `Is`, `Should`, `Apply`, `Stop`, `Allow`, and with no prefix: 11 runs of
10,715,802 probes, 0.002 expected each, 0 hits.

3e20ae96 alone, `force` depth 2 with no anchor, with `--prefix Orientation`, and with
`--suffix Override`: 3 runs, 0.002 expected each, 0 hits.

Classes:

| target | run | expected | hits |
| --- | --- | --- | --- |
| 01c0e452 | `force` depth 2, `--suffix KillConfig` | 0.005 | 0 |
| 01c0e452 | `force` depth 2, `--suffix Config` | 0.005 | 0 |
| 01c0e452 | `force` depth 2, `--suffix Data` | 0.005 | 0 |
| 01c0e452 | `force` depth 3, `--top 450` | 0.043 | 0 |
| 01c0e452 | `force` depth 2 under `ParticleEvent`, `Particle`, `AnimationEvent`, `ParticleEventData`, `Kill`, `AdditionalKill` | 0.002 each | 0 |
| c6de5b9c | `force` depth 2, `--prefix JointOrientation --suffix EventData` | 0.002 | 0 |
| c6de5b9c | `force` depth 2, `--prefix JointOrientation --suffix Data` | 0.002 | 0 |
| c6de5b9c | `force` depth 3, `--top 450`, `--prefix JointOrientation` | 0.021 | 0 |

## What is left

- **01c0e452**: the class `AdditionalKillConfig` points to. Fields `DelayKillSeconds` and
  1aad2ecc. It is not `<x>KillConfig`, `<x>Config` or `<x>Data` for any two words.
- **1aad2ecc**: Bool on 01c0e452, default true. In the client: when true, the delay is
  skipped and the effect is stopped at once if the clip of the event still plays, or plays
  again, when the event ends. Never written in shipped data.
- **c6de5b9c**: the class the pointer 3e20ae96 on `JointOrientationEventData` holds. Fields
  `orientationType` and `OrientationSource`, which moved onto it from the event class. It
  replaces the direction source of the joint orientation modifier while the event is
  active. 2 shipped instances.
- **3e20ae96**: that pointer.

## Status

All five rows are `pending`. All five meet tier 3 and are fit to submit. None rests on a
shipped string, so an upstream PR must say so.
