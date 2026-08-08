# The game mode config pass

Reversing doc for `gamemode-configs`, `gamemode-configs-unproven` and
`guest-of-honor`. Their `batches.tsv` notes point here.

## Method

The IGameModeConfigBase family (77 classes, 54 unnamed at the start) was worked
per class: each unnamed class gets its own search, aimed at its one hash, over a
vocabulary drawn from its recursive structural neighbourhood - its own field
names, the classes its fields reach (walked to depth 3), the fields that hold
it, the entry paths and modes of its shipped instances, and the named half of
the family. Aiming every probe at one hash keeps the noise per target at
`probes / 2^32`; the runs here sat at 0.001-0.007 expected chance hits per
target, ~0.16 summed over every run of the campaign.

As with the semantic pass, the hash match is the filter, not the evidence.

## The names

**AugmentClientConfig** (54091cd5). Base is IGameModeConfigClient; the class is
a `Map[Hash -> Embed 2f798a22]` of per-tier augment button vfx plus a string.
Instances link `ClientStates/Gameplay/UX/LoL/Cherry/AugmentSelection/Particles/
Augment_Remove_Silver_IdleVFX` and siblings - the in-engine client UI. Client is
the base, Augment is the content, Config is the family suffix.

**DynamicCameraConfig** (85bf79cf) and **DynamicCameraSettings** (c565e640).
The config holds `AdditionalSettings: Map[Hash[Link]]` into the settings class
(fields CameraOffsetScale, CameraUpdater, OffsetEdgePanSpeed) plus a default
link to it, so Settings is attested by the field that holds the class. The
DynamicCamera stem is attested by shipped data twice over: every map WAD carries
`InputEventBool` binds with `LogicalName = "FreeDynamicCameraToggle"` /
`"evtFreeDynamicCameraToggle"`, and the retail game writes `DynamicCameraSpeed`
and `DynamicCameraLockMode` into PersistedSettings.json. The two names also
landed in independent runs, parent and inner class agreeing on the stem.

**The timer lattice** - five names that corroborate each other structurally.
ITimerController (acc5c631) was already in the tables; IClockDefinition and
IClockProvider fix the I-prefix pattern for these interfaces.

    692bf354 ITimerControllerDefinition          interface; linked by 9a722730.TimerDefinitions
    289b31c8 ElapsedTimerController              child of ITimerController
    62113c15 CountdownTimerController            child of ITimerController
    610a14d0 CountdownTimerControllerDefinition  child of 692bf354; 2 shipped instances
    7040ad29 ElapsedTimerControllerDefinition    child of 692bf354; meta-declared only

Elapsed/Countdown crossed with Controller/ControllerDefinition landed on four
distinct hashes in exactly the inheritance slots the pattern predicts: the
children of ITimerController take -Controller, the children of 692bf354 take
-ControllerDefinition. Chance does not produce that lattice at these noise
levels.

**AugmentSet** (27bc6378). Own fields SetName, augments, TierData; held in the
AugmentDataListConfig candidate's set list. Shipped scene paths spell the
compound exactly (`.../AugmentSetGroup/AugmentSet/...`), and the tables already
carry AugmentSetTooltipViewController and AugmentSetTraitTrackerViewController.

AugmentSet and ITimerControllerDefinition are two of the 24 the semantic pass
recorded as proposed and not taken. That list was explicitly not a refutation
record, and the call was re-derived here as its doc asks: both now carry
evidence the semantic pass did not have.

## The guest-of-honor batch

**GuestOfHonor** (8b331b12), **GuestOfHonorListData** (c7ccb2ce) and its field
**GuestOfHonorList** (0886394e). Arena's Guest of Honor system, 16.13+. The
list config is one of the 17 `Maps/Shipping/Map30/Modes/CHERRY` configs; its
GuestOfHonorList links all 43 GuestOfHonor entries in `map30.bin`, the only
instances in the game. Vocabulary attested by the `Characters/Cherry_GoH_*`
rigs, the `GuestHonor{Upcoming,Current,Past}.tex` stage icons, the
`cherry_goh_name_*` / `cherry_goh_title_*` stringtable keys and the literal
"Guest of Honor Fiddlesticks" tooltip text. CherryCameo is the direct
predecessor: it carries the same Enabled (02f3b39e) and SkinID (7b34fa25)
field hashes.

Field semantics read off the 16.15 instances:

- `name` doubles as the loc-key suffix (`cherry_goh_name_<name>`); the roster
  includes lore guests ("Locke", "Atakhan", "SahnUzal" on the Mordekaiser rig
  with SkinID 54). Guests with `Enabled = false` have bespoke
  `Cherry_GoH_<self>` rigs; most enabled ones still point at placeholder rigs.
- b0f32561, `List<U8>`: only ever {2}, {8} or {2,8} - the two vote-phase
  rounds of LoLModesRoundsListData.
- e7879fb5, `List<Link self>`: flat mutual groups ({Fiddlesticks, Locke},
  {Darius, Briar, Evelynn}, {Vayne, Vladimir, Ambessa}, Kindred -> Yone
  one-way), not a containment tree.
- 937ed2a5, U8 default 3 on the list data: never serialized in shipped data.

The three field hashes stay unnamed. Ruled out at ~0.6 summed expected chance
hits, so do not re-run: depth-3 compositions over a 205-atom pool (db
structural neighbourhood plus vote/showbiz vocabulary), depth 4 over the
strongest 44 atoms, and the guesser's identity, delete, mutate (full wordlist)
and chain-4 modes aimed at only these three hashes. The unlock is new attested
vocabulary, not more probes.

Two Character field values, verified, for the same CDragon binhashes PR as the
map keys below:

    e13cb23b Characters/Cherry_GoH_Locke
    6264bcd6 Characters/Cherry_GoH_Yone

## gamemode-configs-unproven

**AugmentTierDisplayData** (2f798a22), the embed AugmentClientConfig maps. Per
tier-variant vfx block (PickedVfxSystem, NotPickedVfxSystem, HoverVfxSystem,
IdleVfxSystem, RefreshVfxSystem, RefreshOverlayVfxSystem), shipped under map
keys `remove_silver`/`remove_gold`/`remove_prismatic` and
`kiwi_jade_silver`/`_gold`/`_prismatic`, linking `Augment_Remove_Silver_*` and
sibling vfx. Augment and Tier are attested by the neighbourhood; Display is
convention, no second method confirms it. Not fit for an upstream PR as it
stands.

## Map keys, for a CDragon binhashes PR

This repo has no binhashes table; recorded here so they are not lost. All six
verified against the shipped map keys of 54091cd5's map. The underscores are
attested by the hash; the lowercase is convention, since FNV-1a lowercases.

    fc9e5633 remove_silver
    bb5c5fa4 remove_gold
    2637bc5e remove_prismatic
    1c722370 kiwi_jade_silver
    8fabba83 kiwi_jade_gold
    b5aa0aab kiwi_jade_prismatic

11 of the map's 17 keys remain unresolved: 0b2af4b2 1dd91413 3c1b8c19 4456c501
45bad3c2 73053227 9f6e3000 aa7985b3 d1ebc9b1 d95f7bdb e704c9a0.

## Status

All rows are `status=pending`, `pr=-`. Nothing has been submitted upstream.
