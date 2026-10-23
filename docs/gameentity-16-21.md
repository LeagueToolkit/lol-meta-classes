# GameEntity families, 16.21

Record of the batch `gameentity-16-21`. Its `batches.jsonl` note points here. It names classes and fields of the
GameEntity system: the `IGeComponentDef` definitions, the `GameEntityComponent` runtime components, the
`GameEntityTemplate` templates and `GameEntity` entities, and the support data they embed (state machine helpers,
neutral camp identifiers and pre-spawn data, light updaters, lanes, icons). Every class was read in the 16.21
Windows client, build `8255794`, except the five marked below.

| part | live classes | unnamed classes before | after | unnamed fields before | after |
|---|---:|---:|---:|---:|---:|
| GameEntity templates, components and support data | 174 | 39 | 11 | 85 | 47 |

31 class names and 37 field names; three classes (`IGameObjectEvents`, `GameEntitySelectorMapping`,
`PreSpawnTimerIconData`) sit just outside the counted scope. No string attests any name; casing follows named
siblings.

## Twin sets and shared roots (tier 3)

A component exists as a def (`<Root>GeComponentDef`), a runtime class (`<Root>GeComponent`) and often a template
(`<Root>EntityTemplate`) whose field is named `<Root>`. For the Hierarchy, PlanarReflector and AutoRegionTagging
sets, `back(def, "GeComponentDef") == back(runtime, "GeComponent")` holds, so each set is one hash. The other rows
are sets of classes and fields that one reader traced together.

| hashes | names | what fixes it |
|---|---|---|
| 9ef07ec6, ac98ac97, 10112a28 | HierarchyGeComponentDef, HierarchyGeComponent, Hierarchy | the prefab component that spawns, activates and deactivates the member templates as child entities |
| 2dfc7ffc, b3b32e26, 3634dccb | PrefabEntity, PrefabEntityTemplate, MemberList | the template that embeds the `Hierarchy` def and lists its member item names; it creates a `PrefabEntity` |
| d3dda5e5, 34f2d06e, 59f5f97e, 50a298b | PlanarReflectorGeComponentDef, PlanarReflectorGeComponent, PlanarReflectorEntityTemplate, PlanarReflector | the root hash of the three classes equals the template's field hash; the component registers a planar reflection region with the renderer that creates `PLANAR_REFLECTION_MAP` |
| fdb6b027, 210fea10, d6ebb885 | AutoRegionTaggingGeComponentDef, AutoRegionTaggingGeComponent, AutoTagging | on activation the component adds the nav grid region tags at the entity position to the entity |
| ca19d90e and 9 fields | FlickerMapLightUpdater; Min/MaxFlickerIntensity, FlickerIntensityVariance, Min/MaxFlickerCycleDuration, Min/MaxFlickerCyclesBeforeSleep, Min/MaxSleepDuration | a light updater next to `RotateMapLightUpdater`; the fields come as min / max pairs with matching defaults |
| 608e61e9, 31acb60e, da370296, e8f20237, 595e6ce0, b8252467, 173cf5d9, e59ce2fa | INeutralCampPreSpawnDef, PreSpawnTimerIconDef, PreSpawnTimerIconProgression, PreSpawnTimerIconData, PreSpawnAudioDef, PreSpawnBehavior, PreSpawnAudio, TimeBeforeSpawn | the neutral camp pre-spawn behaviours: the icon timeline and the audio parameter ramp before a camp spawns |
| ad399234, f2a39a, 3636114b, b34ac035 | StateMachineHelperStateData, StateMachineHelperSequenceData, SelectorMappings, GameEntitySelectorMapping | the def-side state entries of `StateMachineGeComponentDef` and their sequence overrides |
| 3f04641e, db26669a, 5a4ef4e7 | NeutralCampIdentifier, IdentifierComponents, CampIdentifier | the camp name and the per-spawn-type component lists that the client dispatches by hash at load and on the network spawn type |

## Singles (tier 4: reader and registration order agree)

| hash | name | what fixes it |
|---|---|---|
| 129e311 | IUpdatableGeComponent | components of this interface join the world tick list when their entity activates |
| 2576b4fc | IGameObjectEvents | secondary base of eight components that wrap an old-system game object |
| 9b321633 | GameplayLocatorTemplate | `<Kind>Template` style, like `MapCameraTemplate`; a locator with a team and an `AutoTagging` def |
| 82cab1b3 | SrLaneDefinition | `LaneDefinition` type; lane and position feed the turret, inhibitor and nexus object names |
| 7a1cab0d | IconReference | one `texturePath` field; every holder is an icon slot |
| 292991be | DragonSoulTypes | `DragonCampGeComponentDef.SoulTypes`; the shipped object lists the soul types |
| fields | RenderSunShadow, RenderEnvironment, SkyboxTexture, PercentageResolution, ForceOn | `PlanarReflectorGeComponentDef`: `ForceOn` runs the pass at the lower graphics setting, the render bools apply in that case |
| fields | CastsShadows, QualityLevels, ProjectedTextureMaterial | dynamic point light and spotlight defs: light type is the bool + 1; one bit per quality level |
| fields | HitRect, BarracksConfigMutatorOverrides, AiScriptOverride, RegenAreaRadius, TeamSpawnPoint, MinimapTimerDefinition | Billboard, Barracks, AttackableUnit, TeamSpawnPoint and NeutralCamp defs; each read on the create path |
| fields | Smooth, RegionGroupIndex, CellHitThreshold, InstanceVariableOverrides, StyleSets | `EntityPolygonData`, `PaintedRegionData`, `RegionBoundaryConfig`, `GameEntityScriptDefinition`, `MinimapTimerStyleSheet` |

## Not in the read build

`EsportsBannerEntityTemplate` (393d54), `EsportsBannerGeComponentDef` (eb5cafc3), `EsportsBannerGeComponent`
(bdaa899c), `AttachmentGeComponentDef` (666d2e06) and `AttachmentGeComponent` (eb9f2cd7) first appear in build
`8298620`; the client build that was read does not register them. The names rest on the twin structure alone: one
lattice gave all five, and each set reduces to one root hash (`EsportsBanner` across template, def and runtime;
`Attachment` across def and runtime). `EsportsBannerData` ships.

## Held back

- `0x4b31cd83` = `IsWard` (AttackableUnit def; no reader found), `0x7453bb15` = `IgnoreOnMinimap` (copied, no
  reader found), `0x4bcc2801` = `CastsDynamicShadows` (MeshHelperDef; no reader found).

## What is left

- Classes: `0x15898c52` / `0xa98b5b03` (a def / runtime pair that spins the character mesh with decaying speed),
  `0x20946fd8`, `0x38ad3494`, `0x82a00368` (death defs, not read in 16.21), `0x1d12c4da` (shape render info),
  `0x10a4976a`, `0x115b5460`, `0x3069f601`, `0x313c0076`, `0x3e265091`.
- 47 fields.

## Status

All rows `pending`.
