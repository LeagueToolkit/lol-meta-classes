# gamemode-configs - campaign record

Record for `gamemode-configs`, `gamemode-configs-unproven`,
`guest-of-honor`, `gamemode-config-readers`, `terrain-disturbance-rdef` and
`terrain-disturbance-unproven`; their `batches.jsonl` notes point here. Method: the
`crack-family` skill (`.claude/skills/crack-family/SKILL.md`), worked per
class - each unnamed class got its own search over its recursive structural
neighbourhood, so noise stayed at 0.001-0.007 expected chance hits per target,
~0.16 summed over the campaign. The `IGameModeConfigBase` family was 77
classes, 54 of them unnamed at the start.

## The names

**AugmentClientConfig** (54091cd5). Base is IGameModeConfigClient; the class
is a `Map[Hash -> Embed 2f798a22]` of per-tier augment button vfx plus a
string. Instances link `ClientStates/Gameplay/UX/LoL/Cherry/AugmentSelection/
Particles/Augment_Remove_Silver_IdleVFX` and siblings - the in-engine client
UI. Client is the base, Augment is the content, Config is the family suffix.

**DynamicCameraConfig** (85bf79cf) and **DynamicCameraSettings** (c565e640).
The config holds `AdditionalSettings: Map[Hash[Link]]` into the settings class
(fields CameraOffsetScale, CameraUpdater, OffsetEdgePanSpeed) plus a default
link to it, so Settings is attested by the field that holds the class. The
DynamicCamera stem is attested by shipped data twice over: every map WAD
carries `InputEventBool` binds with `LogicalName = "FreeDynamicCameraToggle"`
/ `"evtFreeDynamicCameraToggle"`, and the retail game writes
`DynamicCameraSpeed` and `DynamicCameraLockMode` into PersistedSettings.json.
The two names also landed in independent runs, parent and inner class agreeing
on the stem.

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
compound exactly (`.../AugmentSetGroup/AugmentSet/...`), and the tables
already carry AugmentSetTooltipViewController and
AugmentSetTraitTrackerViewController.

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

## gamemode-config-readers (16.18 client readers)

A second pass, worked from the client instead of the schema. Every config sits
in one global table keyed by the metaclass, so the readers were found from the
metaclass key (getter calls, rip-relative `[metaclass+8]` loads, `PostLoad`
vtable slots), never from the class hash, which appears only in the registrar.
Every name below is an exact FNV-1a hit whose decompiled reader fits it. None
is attested by a string, so treat the batch as hash cracks with a behaviour
check.

- **WasdMoveOrderData** (92309121): the directional (WASD) movement driver's
  tuning; its presence gates the control-scheme switch. Ships on SR (Map11).
  Fields come in families that confirm each other: `WallSlideSearch{Enabled,
  MaxDist,MaxBackDist,MinForwardDist}`, `AutoAttackLine{DurationSecs,MaxRange,
  DisplayEasing}` (one shared stem), `EnableUnitBlockAvoidance` /
  `UnitBlockSearchRange`, plus `MinMoveDistance`, `MinTimeBetweenMoveOrders`,
  `MovementRotation` (SR -45), `AlwaysStopOrder`, `StopMovementAngleDeg`,
  `WallSlideDirectionThresholdDeg`, `AutoAttackSpellScript` (was a
  `Link<LolSpellScript>` in 15.7-15.8). Repo casing writes `Wasd`.
- **OverrideKeyBindingsConfig** (0085f9f4) and
  **OverrideDeferredKeyBindingsConfig** (5a92b195): event -> binding-string
  map; `"[]"` unbinds. SR's deferred set unbinds 258 legacy events on a switch
  to WASD. **ControlSchemeSwapConfig** (1ca3eb78) with
  `ControlSwapCooldownSeconds`.
- **DeathRecapGreyscaleConfig** (29619231) / **DeathRecapTeammateSpectatorConfig**
  (ce762bab), held by `GreyscaleConfig` / `TeammateSpectatorConfig` on
  e348122d: each type is "DeathRecap" plus its own field name, a pair that
  confirms itself. `FadeGreyscaleDurationSecs`, `EnableAutoSpectate`.
- **MinimapDrawConfig.DrawTypes** (13-bit layer mask), **TeamColorsConfig** with
  **AccessibleColor** (colour + `colorblindColor`), **VoiceChatConfig.
  TeamVoiceChatEnabled**, **RegaliaConfig.RankedCrestLookup** (overrides the
  default `Loadouts/Regalia/Crests/Ranked/RankedCrestLookup`),
  **ItemRecommendationConfig**, **LuaDeprecationConfig**.
- **LastHitAssistConfig** (60f809c3): `LastHitAssistRadius`, `RankedEnabled`,
  `DisabledQueueIds` (SR {500,501,502}).
- 396e5d4f (class unnamed): `OrderAnchorPoint` / `ChaosAnchorPoint` (SR ships
  the two Nexus positions), `DefaultLaneColor` / `BothTeamsLaneColor`,
  `UpdateFrequencySeconds`.
- `DynamicCameraConfig.DefaultCameraSettings` and `SettingsSwitchDuration`.
- **SmartPingParticleOverrides** (0f2fb88d, 16.19): the single hit in ~9.2M
  candidates (~0.002 expected). Sits beside the existing `SmartPingData`.
  Map453 remaps `SRP_<n>` resolver keys to `JadeP_<n>` (all 11 values
  reproduce as FNV-1a of `JadeP_<n>`), which a map ResourceResolver points at
  `Maps/Shipping/Map453/Particles/Jade_Pings_*`.

Left out on purpose (hash fits, odds too weak): `MapDisplayEnabled`
(9eb14c42, ~8%), `RemoveGreyscaleAfterDeathSecs` (2ac8249e) and
`GreyscaleFadeEndSaturationValue` (9044f758) at ~4% each, `SecondsToResendOrder`
(83f73f14, ~0.6 expected).

## terrain-disturbance-rdef and terrain-disturbance-unproven

0xdea3b4a8 (16.17) is a water + sand surface-sim config; no map ships it and
no client code reads it through 16.20. Its field order mirrors the `$Globals`
of two unnamed Bootstrap shaders (16.19): the water step (`3dc2cb2da9509e57`)
and the sand step (`410dcc87aa229221`). The 28 rdef rows are the CamelCase of
those constants in the matching slot: 24 `Sand*` fields,
`SandImpulseSpacing`, `WaveSpeedWobbleMagnitude`, `WaveSpeedWobbleFrequency`,
`RippleFrequency`.

The unproven batch has neither reader nor shader name: `WindowWorldSize`
(6000, next to `TextureSize`), `ImpulseRadiusMultiplier`, `WakeRefSpeed`
(550), `WakeSpacing`, `WakeMagnitude`, `WakeForwardBias`,
`SandWindAngleDegrees` (where the shader has `SAND_WIND_DIR`),
`MaxWaterFloatCount` (16.19, = 64, the size of the new water-float shader's
`WATER_FLOAT_SAMPLE_POSITIONS`), and the dropped `WakeDirectionSmoothingTime`.
The class pair `WaterDisturbanceConfig` (595773f6, 16.15-16.16) ->
`TerrainDisturbanceConfig` fits the shader's `WaterDisturbanceInfo` and the
sand addition, but is hash-only. Still unnamed: +52, +56, +60, +88 and the
16.19 bool 58947b80.

## gamemode-config-client-readers and hud-feedback-damage-unproven

The three `IGameModeConfigClient` children from 14.x that the 15.x/16.x pass
skipped, worked the same way (metaclass key -> reader in the 16.18 client).
Exact FNV-1a hits whose reader fits; none is attested by a string.

- **TargetingRenderGameModeConfig** (7169f36b, 14.18):
  `TargetingIndicatorsEnabled` (0bbfe2a3, default true). The bool gates the
  spell-targeting indicator renderer: its `TARGETING_*` materials and the
  range, cone, line-missile, AOE and wall-cursor textures. An absent config
  reads as true. Only Map22 (TFT) ships it, set to false.
- **LoadingScreenBackgroundGameModeConfig** (60e2ec74, 14.1): the one hit in
  ~1.9k candidates. `UsingRandomLoadingScreen` (e013f720, 14.20) makes the
  client pick a random entry of `PossibleRandomLoadingScreenBackgrounds`
  (da6afd7c, 14.20); the two share a stem, and the bool is the single 4-word
  hit in ~7.3M. `LoadingScreenBackgroundOverride` is keyed by a string derived
  from the queue. No reader was found for
  `MutatorControlledLoadingScreenBackgrounds`.
- **HudFeedbackDamageConfig** (c3a44766, 14.14), shipped as
  `UX/HUD/Globals/DamageFeedback`: the damage screen flash for the local
  player. `PercentageDamageForFlash` (a86fc2ef) is the share of max health
  that recent damage must exceed, `StartFlashAlpha` (ed23ad91) the alpha floor
  added to the scaled excess, `LowHealthFlashDuration` (b124de6f) the flash
  length on the low-health path. The already named `flashDuration` covers the
  burst path.

The unproven batch holds two more names for the same class that came from a
wider search (roughly 2-3% coincidence odds each):
`LowHealthFlashThresholdPercentage` (e9398686, default 0.6, compared against
the health fraction) and `OverTimeForFlashSeconds` (95823356, shipped 5, the
window the damage is summed over). Left out: `LowHealthFlashOpacityStrength`
(ba7b16a1), which fits the hash but not the reader (the value scales both
paths). Still unnamed: 22728a51 (+16, divisor of the excess damage).

## Status

All rows are `status=pending`, `pr=-`. Nothing has been submitted upstream.
The family itself is resumable: 44 unnamed members remain, with the most
default evidence per class of any family in the census
(docs/unnamed-families.md).
