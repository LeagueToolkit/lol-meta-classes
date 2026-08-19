# The global audio/VO pass

Reversing doc for `global-audio`. Its `batches.tsv` note points here.

82 names off one dormant subsystem introduced across 16.7-16.14: an unnamed
800-byte root that holds a champion-music event table, an announcer-VO event
table, two audio queue holders and the only link anywhere to
`GlobalContextualActionData`; plus the `AudioContextEventType` hierarchy that
the event tables point into, and the two `IContextualAction` implementations
that fire them.

Three passes. The first took the 40-slot event tables (section "The 40 event
slots"); the second took the queue classes, the `AudioContextEventType` tree and
the contextual actions (section "Second pass"); the third read the meta's own
revision history instead of searching name-space (section "Third pass").

```
cls_7D98777D                     800 bytes, unnamed
  +8    ChampionMusicEvents      embed  ->  ChampionMusicEvents   1ee9686c
  +344  AnnouncerVoEvents        embed  ->  AnnouncerVoEvents     9963c03a
  +680  MusicQueueConfigs        embed  ->  AudioQueueConfigList  33d1cb2e
  +696  AnnouncerQueueConfigs    embed  ->  AudioQueueConfigList
  +728  objectPath               hash
  +736  093c1d30                 string
  +752  b01bdea6                 f32
  +760  4249f0a2                 embed  ->  AudioPriorityBehavior 7a55218f
  +784  GlobalContextualActionData  list2[link]  ->  GlobalContextualActionData

AudioContextEvent   f37213a8   56 bytes, abstract      (already named upstream)
  +8   objectPath  hash        +12  NameHash  hash  0d3aa34a
  |- AnnouncerVoEvent    83cccb6d  64  +56 AnnouncerVoEventType    63981b65
  '- ChampionMusicEvent  b4dee917  64  +56 ChampionMusicEventType  2e1cea4f

AudioQueueConfig   9bc919f2   72 bytes
  +16 name string   +32 QueuePriority f32 cd02e270   +36 tag
  +40 c8d0888c f32  +48 DefaultMaxQueueTimes f32 e168a4d7
  +56 1c6439cf list2[link AudioQueueConfig]

AudioPriorityBehavior   7a55218f   24 bytes
  +0 priority f32 (0.5)   +8 TargetQueue link -> AudioQueueConfig  ef5104c7
  +16 disabled bool       +20 MaxQueueTimes option<f32>  c8b6a592
```

`AnnouncerVoEvents` and `ChampionMusicEvents` are **exact twins**: the same 41
property hashes at the same offsets, differing only in what their 40 links point
at and in the element type of the `list2` tail. `GlobalContextualActionData`
derives from `ContextualActionData` and adds nothing; its property hash on
`cls_7D98777D` equals its class hash, so one string names both.

## The 40 event slots

Both twins carry 40 `link` properties at 8-byte stride, sorted alphabetically
inside blocks:

| off | name | off | name |
|---|---|---|---|
| +0 | BaronKill | +160 | PlayerDisconnect |
| +8 | BaronSolo | +168 | PlayerReconnect |
| +16 | BaronSteal | +176 | Respawn |
| +24 | DragonKill | +184 | Aced |
| +32 | DragonSolo | +192 | ChampionExecuted |
| +40 | DragonSteal | +200 | ChampionKill |
| +48 | ElderDragonKill | +208 | ChampionShutdown |
| +56 | ElderDragonSolo | +216 | FirstBlood |
| +64 | ElderDragonSteal | +224 | KillingSpree |
| +72 | GrubsKill | +232 | MultiKill |
| +80 | GrubsSolo | +240 | PentaKill |
| +88 | GrubsStolen | +248 | *unresolved* |
| +96 | RiftHeraldKill | +256 | InhibitorKill |
| +104 | RiftHeraldSolo | +264 | InhibitorRespawn |
| +112 | RiftHeraldSteal | +272 | InhibitorRespawnSoon |
| +120 | BountyEnded | +280 | TowerKill |
| +128 | BountyStarting | +288 | BaronSpawn |
| +136 | GameEnd | +296 | ElderSpawn |
| +144 | GameStart | +304 | *unresolved* |
| +152 | MinionsSpawn | +312 | RiftHeraldSpawn |
| | | +320 | *unresolved* (`list2[embed]` tail) |

`PlayerDisconnect` and `PlayerReconnect` are `exe-strings` cracks that already
covered +160 and +168; only three slots were ever open.

## Method

Every hash comes from a `MetaClass_register` / `MetaClass_addProperty` site in the
shipped Windows client, so the offsets, sizes, types and the twin relationship are
read out of the binary rather than inferred. Names are FNV-1a-32 recombination
scored against that structure.

**The load-bearing step is a 4-way constraint.** The middle term of each objective
triple was found by searching for a single suffix `S` satisfying all four of

```
fnv("baron" + S)       == 90214be4
fnv("dragon" + S)      == b4a64e19
fnv("elderdragon" + S) == 829dfd5f
fnv("riftherald" + S)  == fa045deb
```

`solo` is the only hit over 12.4M suffix candidates. Four independent 32-bit
matches from one string is not a coincidence at any candidate count worth
discussing, and it agrees with `kBaronSoloKilled` in the client's own
`ChallengesEventType` enum.

**`Grubs` was recovered before it was guessed.** Peeling `kill` off `9bbb4468`
and `solo` off `a9769d67` (FNV-1a is invertible) lands both on the *same* prefix
state `284570e8`. Meeting in the middle on that one state returns `grubs` -
voidgrubs - as the only word-shaped answer. Two confirmations, no wordlist bias.

Everything else came from curated candidate lists of a few hundred to a few
thousand strings, where the expected number of chance hits is 1e-5 or below:
`AnnouncerVoEvents` and `AudioContextEvent` fell out of a 374-string list,
`AnnouncerVoEvent`/`ChampionMusicEvent`/`ChampionMusicEvents` out of ~2k-string
lists anchored on the names already found.

### Calibration

The pass was run blind against names this repo and upstream already carried, and
recovered all of them with identical spelling: `Respawn`, `FirstBlood`,
`KillingSpree`, `ChampionKill`, `GameStart`, `GameEnd`, `MultiKill`, `BaronKill`,
`DragonKill` (from `semantic-pass-*` and `exe-strings`), plus `AudioContextEvent`,
`AnnouncerVoEventType`, `EventType` and `OnEvent` (from `audio-events`). Thirteen
prior cracks reproduced by an unrelated method is the reason to trust the rest.

### Weaker entries

- **`Aced`** and **`BountyStarting`** were each the sole word-shaped hit in their
  pass. Both sit in the alphabetical slot their spelling requires, but neither has
  a second method behind it.
- **`GrubsStolen`** breaks the `Steal` convention every other objective uses. The
  slot is forced (third of a triple whose first two are confirmed) and the string
  is word-shaped, so the irregularity is Riot's, not the search's.

### Casing

FNV folds case, so no spelling here is attested - the repo's PascalCase rule
picks it. Upstream's own entries for this table disagree with each other
(`Respawn`, `FIRSTBLOOD`, `pentakill`), which is the clearest evidence that the
casing carries no information.

## Second pass

34 further names, off three structures the first pass had only sketched: the
queue classes, the `AudioContextEventType` tree, and the `IContextualAction`
implementations that reach the events. Family `0x4ca99280` went from 15 unnamed
members and 28 unresolved fields to 4 and 5.

What opened it was `ContextualRule` (already named, and holding
`mAudioAction`, `mAnimationAction`, `mTriggerEventAction` beside two unnamed
pointers): it supplies both the `ContextualAction<X>` class template and the
word `Cac`, from its own `mOverrideCacCooldown`.

### The shape recovered

```
AudioContextEventType (4ca99280, abstract)
|- AnnouncerVoEventType   63981b65   +0 1ad40789 string  +16 PriorityBehavior
|  |- MultiAnnouncerVoEventType      906b3602   2f1edf29 bool, AnnouncerVoEventTypes
|  '- AnnouncerVoEventTypeConcrete   ab30ed5e   (abstract)
|     |- 36255113                    4 behaviors + 7 event strings
|     |- AnnouncerVoEventTypeGeneric 9e23d81a   GenericEvent
|     '- AnnouncerVoEventTypeCac     c9ed1a3e
|        '- AnnouncerVoEventTypeModularCac  4036941a   PrimaryEvent -> 36255113
'- ChampionMusicEventType 2e1cea4f
   |- ChampionMusicEventTypeCac      4870b9d4
   |- MultiChampionMusicEventType    b58b70dc   2f1edf29 bool, ChampionMusicEventTypes
   '- ChampionMusicEventTypeConcrete c87f69f4   (abstract)
      |- 9583cf01                    3 behaviors + 5 event strings
      |- ChampionMusicEventTypeGeneric  227fcad0   PriorityBehavior, GenericEvent
      |- 15e1fe7e                    PriorityBehavior, 4c546bb7 string
      '- a29e7869                    PriorityBehavior, TeamSucceeded/TeamFailedEvent

IContextualAction (3ff90079)
  |- ContextualActionPlayAudio, ...PlayAnimation, ...TriggerEvent   (already named)
  |- ContextualActionAnnouncerVoEvent    1c23f02c   TriggerAnnouncerVoEvent
  |- ContextualActionChampionMusicEvent  8482aa12   TriggerChampionMusicEvent
  '- 66dc7e9b                            ec1b2b3c hash, Subject
```

The two "full" event types are the outcome matrices. `0x36255113` carries
`Individual`/`TeamSucceeded`/`TeamFailed`/`General` `PriorityBehavior` embeds and
the `IndividualSucceeded`/`IndividualFailed`/`TeamSucceeded`/`TeamFailed`/`Generic`
`Event` strings; `0x9583cf01` carries `Individual`/`Assisted`/`Team` behaviors and
the same strings plus `IndividualAssistedEvent`.

### Evidence

Noise below is `probes x targets / 2^32` for the run that produced the row.

| hashes | names | what fixes it | tier |
|---|---|---|---|
| 9bc919f2, 33d1cb2e, 64e312ae, 6e654622 | AudioQueueConfig, AudioQueueConfigList, MusicQueueConfigs, AnnouncerQueueConfigs | one 2,414-probe curated pass, 1.2e-5, four hits closing a 2x2: the root's two `AudioQueueConfigList` embeds sit at +680/+696 in the same Music-then-Announcer order as `ChampionMusicEvents` (+8) / `AnnouncerVoEvents` (+344) | 3 |
| 64132003, a084e3ec, c5d72455, 50b8f3f6 | PriorityBehavior, Individual-, General-, TeamPriorityBehavior | 31,830 probes, 6e-5, four hits in one pass | 3 |
| 141437e7 | TeamSucceededPriorityBehavior | curated outcome pass, 32,190 probes, 2.2e-5; completes the announcer row beside TeamFailed | 3 |
| f6122be7 | TeamFailedPriorityBehavior | depth-2 corpus sweep under the proved `PriorityBehavior` suffix, 0.009 | 3 |
| eff5bca1 | AssistedPriorityBehavior | depth-2 inflected sweep under the same suffix, 0.073; paired with `IndividualAssistedEvent` on the same class | 3 |
| 8e8371bb, d4714dbb | TeamSucceededEvent, TeamFailedEvent | curated pass, 45,885 probes, 1e-4, two hits; same Succeeded/Failed pair the behaviors already fixed | 3 |
| 7c74a459, a2f5a276 | IndividualSucceededEvent, GenericEvent | curated lattice pass, 22,449 probes, 4e-5. `IndividualSucceededEvent` was predicted by the row before it was found; `GenericEvent` hit in three independent passes | 3 |
| fbb66c25, 66fa31d8 | IndividualFailedEvent, IndividualAssistedEvent | depth-2 inflected under the `Event` suffix, 0.585 alongside two junk hits; they complete the Individual row and `IndividualAssistedEvent` pairs with `AssistedPriorityBehavior` | 3 |
| de72a3a2, d384bc74 | AnnouncerVoEventTypes, ChampionMusicEventTypes | curated 5,819 probes, 2e-5, twin pair; these are the lists the two `Multi*` classes hold | 3 |
| 906b3602, b58b70dc, 7a55218f | MultiAnnouncerVoEventType, MultiChampionMusicEventType, AudioPriorityBehavior | depth-2 corpus under a 12-suffix list, 1.18. The `Multi*` pair is a twin sharing field `0x2f1edf29` and is corroborated by the two list names above; `AudioPriorityBehavior` is the class whose field name `PriorityBehavior` is separately proved | 3 |
| 9e23d81a, 227fcad0 | AnnouncerVoEventTypeGeneric, ChampionMusicEventTypeGeneric | two **independent** depth-3 runs, prefixed `AnnouncerVo` and `ChampionMusic`, 0.121 and 0.141; separate searches returned the same suffix word, and both classes carry `GenericEvent` as their only string | 3 |
| ab30ed5e, c87f69f4 | AnnouncerVoEventTypeConcrete, ChampionMusicEventTypeConcrete | two independent depth-3 runs, 0.070 and 0.140, same suffix word from separate searches | 3 |
| c9ed1a3e, 4870b9d4, 4036941a | AnnouncerVoEventTypeCac, ChampionMusicEventTypeCac, AnnouncerVoEventTypeModularCac | depth-2 inflected under the two `EventType` prefixes, 0.220 and 0.256. `Cac` is corpus vocabulary from `ContextualRule.mOverrideCacCooldown`; `ModularCac` derives from `Cac`, so the name extends its own base | 3 |
| 1c23f02c, 8482aa12 | ContextualActionAnnouncerVoEvent, ContextualActionChampionMusicEvent | curated template pass, 2,861 probes, 7e-6, twin pair matching the named `ContextualActionPlayAudio` / `PlayAnimation` / `TriggerEvent` siblings | 3 |
| ef5104c7 | TargetQueue | fold of all 15,368 known names against 122 suffixes, 1.95M probes, 0.0095, single hit; the field is the `Link -> AudioQueueConfig` on `AudioPriorityBehavior` | 4 |
| 2e1cea4f (class), 63981b65 (field) | ChampionMusicEventType, AnnouncerVoEventType | hash identity across tables: each hash is already the other table's name for the same concept, and the pointer field on `ChampionMusicEvent` / `AnnouncerVoEvent` targets exactly that class | 1 |

### Negatives

Do not re-run these. Summed expected false positives, second pass: ~10.

| run | probes x targets | noise | result |
|---|---|---|---|
| identity + delete, both tables, whole family | 0.3M x 59 | 0.002 | nothing |
| bintypes force d2, corpus wordlist, 21 classes | 20.2M | 0.099 | nothing |
| bintypes force d2, inflected wordlist, 18 classes | 314M | 1.32 | nothing |
| bintypes force d2 inflected, suffix `AnnouncerVoEventType` / `ChampionMusicEventType` | 2 x 314M | 0.37, 0.44 | junk only |
| bintypes force d3 curated-532, prefix `*EventTypeConcrete` | 2 x 151M | 0.035, 0.105 | nothing |
| bintypes force d2 inflected, prefix `*EventTypeConcrete` | 2 x 157M | 0.037, 0.110 | nothing |
| bintypes chain d4 inflected, prefix `*EventType` | 2 x 10.6M | 0.002, 0.007 | nothing |
| bintypes force d3 curated-532, unanchored, root + 859c9c2f | 302M | 0.140 | nothing |
| bintypes force d3 curated-532, prefix `AnnouncerVo` / `ChampionMusic`, on 5f38de60 / d52eac74 | 2 x 151M | 0.035 each | nothing |
| binfields force d2 inflected, suffix {Action, Event, EventAction, Name, Hash, Trigger}, with and without `m`, on the 7 `ContextualRule` hashes | 2 x 157M | 1.46 each | junk only |
| binfields force d3 curated-442, suffix {Event, EventName, Name}, on 4 strings | 86.5M | 0.242 | nothing |
| binfields force d3 curated-442, 12-suffix list, on 10 structural fields | 86.5M | 2.42 | junk only |
| binfields force d3 curated-532, suffix `Event`, on 4 strings | 151M | 0.140 | nothing |
| both tables force d2 over the corpus **refreshed with this pass's 34 names**, all 32 leftovers | 30M | 0.10 | nothing |
| curated 7,547-string semantic pass over the leftover fields and the three open event slots | 0.11M | 3e-5 | nothing |

The three open event slots also survived the first pass's ~2.1k curated pass and
a full-wordlist depth-2 force (10.1M probes, 0.007).

## Third pass

Ten more names, off a lever the first two did not use: **the meta's own revision
history**. Every dump since 13.15 is folded into `db/meta.db.json` with a `from`
build and, for dead properties, a `to`. A field that dies in one patch and a
field that is born in the next are a rename, and when the new name is known the
old one is constrained to whatever the rename rule produces. That turns a search
over name-space into a search over one or two candidates.

### The 16.9 priority rename

`AudioPriorityBehavior` is born at 7695709 (16.9) holding `priority`, `disabled`
and `TargetQueue`, and in the same build the plain `priority` f32 on the event
types is replaced by a `PriorityBehavior` embed of it. Both ends of that rename
are already named, so the rule is attested rather than assumed:

```
priority  94e4e309   ->   PriorityBehavior  64132003
```

`0x9583cf01` is the only class in the family that carried *per-scope* priority
floats beside the plain one, and it lost two of them at 7668562, the build
before. Applying the same rule backwards to its own live behaviors names both
dead floats at once:

| dead f32 | died | live embed | born | recovered |
|---|---|---|---|---|
| `1fd795aa` (default 0.6) | 7668562 | `IndividualPriorityBehavior` a084e3ec | 7695709 | `IndividualPriority` |
| `b038b134` (default 0.2) | 7668562 | `TeamPriorityBehavior` 50b8f3f6 | 7695709 | `TeamPriority` |

Both fell out of one 8.66M-probe curated pass at 0.052 expected noise. The rule
is what makes them a pair rather than two guesses, and the **negative is the
consistency check**: the rule's four other possible outputs - `GeneralPriority`,
`AssistedPriority`, `TeamSucceededPriority`, `TeamFailedPriority` - occur nowhere
in 248 dumps, exactly as they should not, because those four behaviors were
introduced fresh at 16.11 and 16.13 rather than migrated from a float.

### Why `MusicQueues` had to be renamed

`AudioQueueConfigList` has held exactly two property hashes in its life:
`9346aa18` from 7695709 to 7765552, and `queues` from 7794239 onward, same type
both times. The recovered name for the dead one is `MusicQueues`, and the rename
has a visible cause rather than being arbitrary: the class was music-only until
the root grew `AnnouncerQueueConfigs` at +696 beside `MusicQueueConfigs` at +680,
both embeds of this same list class. Once one list type served both, a member
called `MusicQueues` could not stay.

### A stem pair born in one build

16.12 (7851316) adds two floats in the same build, one on each side of the
`TargetQueue` link:

| hash | class | type | recovered |
|---|---|---|---|
| `e168a4d7` | `AudioQueueConfig` | f32 | `DefaultMaxQueueTimes` |
| `c8b6a592` | `AudioPriorityBehavior` | option<f32> | `MaxQueueTimes` |

`AudioPriorityBehavior.TargetQueue` links to `AudioQueueConfig`, so this is the
ordinary default-plus-override shape: the queue carries the default, the behavior
optionally overrides it. The two names were found in separate runs (0.052 and
1.256 noise) and share a stem neither run was told about.

### Two twin pairs

`TriggerAnnouncerVoEvent` / `TriggerChampionMusicEvent` are the sole link field
on `ContextualActionAnnouncerVoEvent` / `ContextualActionChampionMusicEvent`.
Both landed in one 8.66M-probe suffix-anchored run.

Given those, the two unnamed `ContextualRule` pointers followed from the sibling
convention on their own class - `mAudioAction`, `mAnimationAction`,
`mTriggerEventAction` - and both hit from a 20-string hand list:

```
be2133eb  TriggerAnnouncerVoEvent      ->  f007f2a9  TriggerAnnouncerVoEventAction
057edc39  TriggerChampionMusicEvent    ->  3a526bfb  TriggerChampionMusicEventAction
```

Two 32-bit matches out of twenty probes, landing on the twin pair of a rule the
previous line proved, is a pair constraint; it also confirms the two field names
it was derived from.

### The one shipped name in the batch

`20749c51` is a bool on `ContextualRule`, default `true`, alive since 6236777.
It is the only name in this batch that shipped data can speak to, and the only
one resting on a single unpaired hit.

- It is the sole coherent English phrase among 162 chance hits over a 6.6e11
  prefix-by-suffix space, and the only hit of the 0.052-noise curated pass.
- It is **shipped**: 116 authored occurrences across 38 WADs.
- Every occurrence is `false`, against a `true` default, so authors only ever
  turn it off.
- Every occurrence sits on a rule under a *reactive* situation:
  `AnimationEnded`, `SpellBuffReceive`, `SpellBuffRemove`, `SpellCast`,
  `CharacterLevelUp`. None under a deliberate-performance situation.

`ActiveVoiceOver` reads correctly against that distribution - incidental barks
marked as not-active - and it sits beside `CanStompSelf` and
`StompLowerPriority`, the stomping group an active-vs-passive flag belongs to.
The distribution corroborates the *meaning*; it does not attest the *string*, so
this stays tier 4. Note also that the family spells the concept `Vo` elsewhere
(`ContextualConditionEndedVoEventName`, `mOverrideCacCooldown`) while this name
spells it `VoiceOver`; the hash is the only thing saying so.

### Evidence

| hashes | names | what fixes it | tier |
|---|---|---|---|
| 1fd795aa, b038b134 | IndividualPriority, TeamPriority | dead-to-live rename boundary at 7668562/7695709 under a rule both of whose ends are already named, plus a four-way negative that the rule produces nothing else | 2 |
| 9346aa18 | MusicQueues | dead 7695709..7765552, replaced by `queues` at 7794239; the rename has a structural cause in the root gaining a second list of the same class | 2 |
| c8b6a592, e168a4d7 | MaxQueueTimes, DefaultMaxQueueTimes | same-build birth pair (7851316) on the two classes joined by `TargetQueue`, default-plus-override shape, found in independent runs | 2 |
| be2133eb, 057edc39 | TriggerAnnouncerVoEvent, TriggerChampionMusicEvent | twin pair, both in one run at 1.256 noise; sole link field on the two twin contextual actions | 3 |
| f007f2a9, 3a526bfb | TriggerAnnouncerVoEventAction, TriggerChampionMusicEventAction | twin pair, 2 hits from 20 hand probes, each the field name above plus the `Action` suffix its three named siblings use | 3 |
| 20749c51 | ActiveVoiceOver | single unpaired hash; shipped in 116 objects, always `false`, always under reactive situations | 4 |

### Negatives

Do not re-run these. Summed expected false positives, third pass: ~450, almost
all of it in the two MITM sweeps, which were run precisely because they are cheap
enough to afford that much noise and be read through a hard filter.

| run | probes x targets | noise | result |
|---|---|---|---|
| binfields force d3, full 3,181-word corpus, the 2 open event slots | 32.2e9 x 2 | 15.0 | 16 hits, indistinguishable from chance |
| binfields force d3 top-800, 26 cluster fields | 513M x 26 | 3.10 | 6 hits; only `IndividualPriority` / `TeamPriority` survived review |
| bintypes force d3 top-800 under 5 prefixes, 9 cluster classes | 2.56e9 x 9 | 5.37 | junk only |
| identity + delete + mutate, both tables, whole cluster | 1.80e9 x 8 | 3.35 | nothing |
| binfields force d2 full corpus under 18 event-shaped suffixes, 2 event slots | 10.1M x 36 | 0.085 | nothing |
| MITM prefix-by-suffix, 813k tokens each side, 2 event slots, alphabetical filter applied | 6.6e11 | 154 | 159 and 161 raw hits - chance; nothing word-shaped survived the bracket |
| same MITM widened to 1.08M tokens with League proper nouns added | 1.17e12 | 272 | 291 raw hits - chance |
| twin-constrained MITM (2 simultaneous 32-bit constraints), `5f38de60`/`d52eac74`, 5 infix templates x prefix and suffix | 8.8e11 pairs per template | ~0 | nothing |
| twin-constrained MITM, `36255113` paired against each of the 3 unnamed music concretes, 3 templates each | 2.8e12 pairs per template | ~0 | nothing |
| twin-constrained MITM, `ContextualAction<X>` class against `<X>Action` field | 8.8e11 pairs | ~0 | nothing |

Two of these are findings rather than failed runs.

**The two open event slots are not built from known vocabulary.** Between the
depth-3 corpus force and the two MITM sweeps, every arrangement of up to four
tokens drawn from all 3,181 corpus words plus ~300 curated audio and League terms
has been tried against both hashes - on the order of 1e12 candidates - and the
only output is chance. Whatever word is missing is not in any name this game has
ever shipped.

**The announcer and music concrete subclasses are not name twins.** The
two-constraint MITM would have found any pair differing only in an
`AnnouncerVo`/`ChampionMusic` token, under any prefix or suffix, over 2.8e12
candidate pairs, for all three possible pairings of `36255113` against the music
side. Nothing. That is consistent with their property sets, which are not twins
either: `36255113` has `GenericEvent` and a `TeamSucceeded`/`TeamFailed` behavior
split, `9583cf01` has `Assisted` and no `GenericEvent`. They are different
concepts that happen to share a shape, so the naming symmetry that carried the
rest of this family stops here.

## Not shipped - do not submit yet

`bin-grep --class <hash> --count` returns **0 objects on retail** for every class
in this batch, old and new, and for `GlobalContextualActionData`. The subsystem
is registered by the client but no data instantiates it: `ContextualRule` has
21,705 shipped instances, and not one of them sets the two pointers that reach
`ContextualActionAnnouncerVoEvent` or `ContextualActionChampionMusicEvent` -
re-checked directly on `f007f2a9` and `3a526bfb` once those were named, 0 hits
each across 456 WADs.

One exception, added in the third pass. **`20749c51 ActiveVoiceOver` is
shipped**: 116 authored occurrences across 38 WADs, on `ContextualRule`, which is
a long-lived class outside the dormant part of this family. It is the only name
here that clears CommunityDragon's bar on its own, and it can be split off and
submitted independently of the rest.

Everything else is attested by the binary, not by shipped data. CommunityDragon's
bar is that a submitted name occurs in shipped data, so the remainder of this
batch should stay `pending` here until instances appear.

## Leftovers

Nine classes and sixteen fields, after three passes.

Classes: the root `7d98777d`; the two outcome matrices `36255113` and `9583cf01`;
`15e1fe7e` and `a29e7869` under `ChampionMusicEventTypeConcrete`; the event-binding
triple `859c9c2f` (`EventType` u32 + `e7623bb7` f32) with its children `5f38de60`
and `d52eac74` (`OnEvent` link to the announcer / music event); and `66dc7e9b`
under `IContextualAction`.

Fields: root `093c1d30` (string), `b01bdea6` (f32), `4249f0a2` (embed
`AudioPriorityBehavior`); queue `c8d0888c` (f32) and `1c6439cf` (list2 of
itself); `e7623bb7`; `2f1edf29` (bool shared by both `Multi*` classes); the
strings `1ad40789`, `2f68e6e3`, `11e8956e`, `4c546bb7`; the `ContextualRule`
pointer `913543b4` and `ec1b2b3c` on the class it points at; and the three event
slots `d0cc4924` (+248), `6a4fa98c` (+304), `ba9bae76` (+320).

Alphabetical bounds for the two unresolved link slots, which is the strongest
constraint available for a follow-up pass:

| hash | slot | must sort between |
|---|---|---|
| `d0cc4924` | +248 | after `PentaKill`, **or** before `InhibitorKill` |
| `6a4fa98c` | +304 | `ElderSpawn` .. `RiftHeraldSpawn` |

`6a4fa98c` is interior to a block and so is bounded on both sides. `d0cc4924` sits
on a block boundary, and which block it belongs to is not decidable from the
offsets alone: it is either the last of the champion-combat block that ends with
`PentaKill`, or the first of the structure block that begins with `InhibitorKill`.
Only the first reading was stated after the second pass; both are open.

The bound is worth stating carefully: it holds for the 40-slot event tables,
whose blocks really are alphabetical, and **not** in general. `ContextualActionPlayAudio`
declares `mSelfEventName`, `mAllyEventName`, `mEnemyEventName`,
`mSpectatorEventName` in that order, so nothing in the second- or third-pass
classes above can be constrained this way.

Every leftover has now survived a depth-2 force over the corpus wordlist, over a
12.5k inflected wordlist, a depth-3 force over 442-, 532- and 205-word curated
audio vocabularies under every suffix and prefix this family proves, a depth-3
force over the entire 3,181-word corpus, and two meet-in-the-middle sweeps
covering ~1e12 prefix-by-suffix pairs. What is left needs new attested
vocabulary, not more probes - and, for the class names, some source other than
the announcer/music twin symmetry, which the third pass showed does not extend to
the concrete subclasses.
