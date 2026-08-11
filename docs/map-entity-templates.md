# map-entity-templates - campaign record

Record for `map-entity-templates`; its `batches.tsv` note points here. Method:
the `crack-family` skill (`.claude/skills/crack-family/SKILL.md`).
`MapPlaceableBase` went from 28 live unnamed members to 5. 109 names landed:
98 classes and 11 fields.

The family keeps parallel expressions of one concept set, and each is a naming
family: `<X>Entity`, `<X>EntityTemplate`, `<X>EntityPrefab` (15.16-15.20),
`Map<X>`, `<X>Definition`, `<X>GeComponent`, `<X>GeComponentDef` (15.22+),
`<X>GeComponentDefinition` (15.16-15.21). A template composes components that
were already named upstream, so most names are a function of the composed side
before any hash is computed.

## Runs

| run | probes | states | expected noise | yield |
|---|---|---|---|---|
| slot filling, `guesser_families.py check` over 13 proposal files | 465 | ~6,900 | <0.001 | 68 names |
| `force` depth 2 top 900, 9 suffix anchors | 1,621,800 | 731 | 0.276 | 14, all in predicted slots; two 4-slot lattices |
| `force` depth 3 over a 195-word family vocabulary | - | - | 2.49 | 5, each corroborated by an established name |
| shipped strings: CommunityDragon (3.6M forms) + both clients (248k) against stem states and field hashes | - | - | 0.006-0.10 | 11 classes, all 11 fields |

Suffix folding supplied the anchors and three derivations: `0x76bc0857`
unfolds under `Base` to `0x880c52da`; the `Superward` and `PaintedRegion`
stems equal unresolved field hashes on their own templates; the shared-stem
groups are in "What is left".

Ruled out, so nobody re-treads: `chain` depth 4 and 5, `mutate` over the full
wordlist, `delete`, and `force` depth 3 over the top 1400 aimed at the seven
open stem states (3.7, 33.3, 0.0 and 9.0 expected noise respectively - the
last two are indistinguishable from chance and were read as such).

## Attested by a shipped string

| stem | what attests it | names |
|---|---|---|
| `Superward` | `Shared/Spells/FaeLightSuperwardBuff`; every instance is `Characters/SRU_FaeLightWardPad` | `SuperwardEntity`, `SuperwardEntityTemplate`, `SuperwardDefinition`, `SuperwardGeComponent`, `SuperwardGeComponentDef`, field `Superward` |
| `PaintedRegion` | `HUDTacticalMapPaintedRegion` beside `HUDTacticalMapLaneState` / `HUDTacticalMapPassable`, and the shader `HUD/TacticalMapRegionPainting.ps` | `PaintedRegionEntity`, `PaintedRegionEntityTemplate`, `PaintedRegionGeComponent`, `PaintedRegionGeComponentDef`, `PaintedRegionData`, `PaintedRegionCell`, field `PaintedRegion` |

Both stems are field hashes on their own template, reached by unfolding, then
cracked against the string corpus - six and seven names from one string each.

Fields, same method:

| hash | name | shipped as |
|---|---|---|
| `018a801e` | `RegenTickInterval` | `sp_RegenTickInterval` |
| `3694b10d` | `HealthRegenPercent` | `sp_HealthRegenPercent`, beside the named `ManaRegenPercent` on the same class |
| `6723b8f8` | `HpMaxPenaltyRegenPercent` | `sp_HPMaxPenaltyRegenPercent` |
| `9b15d5eb` | `SSAO` | `OMIT_SSAO` |
| `c7543bfc` | `RenderParticles` | `SpawnForceRenderParticles` |
| `670b6ae3` | `Dimension` | whole string, both clients |
| `0a9e6b95` | `Taper` | whole string, beside this class's `bevel`, `height`, `Margin` |
| `c6d048fc` | `HdrEnvDiffuseScale` | `HDRENVDIFFUSESCALE` |
| `cb13aff1` | `DepthPushPull` | `DEPTHPUSHPULL` |

The `sp_` block is the spawn-point half of the map config, in the order
`TeamSpawnPointGeComponentDef` declares its fields, which is independent
confirmation of `TeamSpawnPointDefinition` and its template.

Casing: `SSAO` keeps its capitals and is in `names.py`'s `ACRONYM_EXEMPT` -
the shader's own `OMIT_SSAO`, and the three `MapSSAO*` entries already spell
it that way. `HpMaxPenaltyRegenPercent`, `HdrEnvDiffuseScale` and
`DepthPushPull` are recased under the repo rule; the tables already carry
`FlatHpPoolMod` and `HdrScale`.

## The lattices

Each row is one concept; a filled cell is a name this batch added, a cell in
*italics* was already named.

| concept | Entity | EntityTemplate | EntityPrefab | Definition | GeComponentDef |
|---|---|---|---|---|---|
| Attackable | *AttackableEntity* | `AttackableEntityTemplate` | `AttackableEntityPrefab` | *AttackableUnitDefinition* | *AttackableUnitGeComponentDef* |
| Barracks | *BarracksEntity* | `BarracksEntityTemplate` | `BarracksEntityPrefab` | *BarracksDefinition* | *BarracksGeComponentDef* |
| GameMode | *GameModeEntity* | `GameModeEntityTemplate` | `GameModeEntityPrefab` | *GameModeDefinition* | - |
| NeutralCamp | *NeutralCampEntity* | `NeutralCampEntityTemplate` | `NeutralCampEntityPrefab` | *NeutralCampDefinition* | *NeutralCampGeComponentDef* |
| DragonCamp | - | `DragonCampEntityTemplate` | `DragonCampEntityPrefab` | - | *DragonCampGeComponentDef* |
| Shop | *ShopEntity* | `ShopEntityTemplate` | `ShopEntityPrefab` | *ShopDefinition* | *ShopGeComponentDef* |
| Sfx | *SfxEntity* | `SfxEntityTemplate` | `SfxEntityPrefab` | *SfxDefinition* | *SfxGeComponentDef* |
| Vfx | *VfxEntity* | *VfxEntityTemplate* | `VfxEntityPrefab` | *VfxDefinition* | *VfxGeComponentDef* |
| Billboard | - | `BillboardEntityTemplate` | `BillboardEntityPrefab` | - | *BillboardGeComponentDef* |
| LightRegion | `LightRegionEntity` | `LightRegionEntityTemplate` | `LightRegionEntityPrefab` | `LightRegionDefinition` | *LightRegionGeComponentDef* |
| TeamSpawnPoint | `TeamSpawnPointEntity` | `TeamSpawnPointEntityTemplate` | `TeamSpawnPointEntityPrefab` | `TeamSpawnPointDefinition` | *TeamSpawnPointGeComponentDef* |
| MapCamera | *MapCamera* | `MapCameraTemplate` | `MapCameraPrefab` | `MapCameraDefinition` | - |
| RegionEntityBase | *RegionEntityBase* | `RegionEntityBaseTemplate` | `RegionEntityBasePrefab` | *RegionDefinition* | *RegionGeComponentDef* |
| Region | *RegionEntity* | *RegionEntityTemplate* | `RegionEntityPrefab` | - | - |
| CharacterMesh | *CharacterMeshEntity* | *CharacterMeshEntityTemplate* | `CharacterMeshEntityPrefab` | *CharacterMeshDefinition* | *CharacterMeshGeComponentDef* |
| Environment | *EnvironmentEntity* | *EnvironmentEntityTemplate* | `EnvironmentEntityPrefab` | *EnvironmentEntityDef* | - |
| Locator | *LocatorEntity* | *LocatorEntityTemplate* | `LocatorEntityPrefab` | *LocatorEntityDef* | - |
| Mesh | *MeshEntity* | *MeshEntityTemplate* | `MeshEntityPrefab` | - | - |
| AreaTrigger | `AreaTriggerEntity` | `AreaTriggerEntityTemplate` | `AreaTriggerEntityPrefab` | `AreaTriggerDefinition` | - |
| MonarchLocator | `MonarchLocatorEntity` | `MonarchLocatorEntityTemplate` | `MonarchLocatorEntityPrefab` | `MonarchLocatorDefinition` | - |
| Superward | `SuperwardEntity` | `SuperwardEntityTemplate` | - | `SuperwardDefinition` | `SuperwardGeComponentDef` |
| PaintedRegion | `PaintedRegionEntity` | `PaintedRegionEntityTemplate` | - | - | `PaintedRegionGeComponentDef` |
| Green | - | `GreenEntityTemplate` | `GreenEntityPrefab` | - | `GreenGeComponentDef` |
| RedGreen | - | `RedGreenEntityTemplate` | `RedGreenEntityPrefab` | - | - |
| DynamicSpotlight | - | `DynamicSpotlightEntityTemplate` | - | - | *DynamicSpotlightGeComponentDef* |
| DynamicPointLight | - | `DynamicPointLightEntityTemplate` | - | - | *DynamicPointLightGeComponentDef* |
| WorldSpaceUi | - | `WorldSpaceUiEntityTemplate` | - | - | *WorldSpaceUiGeComponentDef* |
| ModesScenario | - | `ModesScenarioEntityTemplate` | - | - | *ModesScenarioGeComponentDef* |
| proxy link | - | *GameEntityTemplateProxyLink* | `GameEntityPrefabProxyLink` | - | - |

`Red` and `Green` are the engine's own test components - `RedGeComponent` and
`GreenGeComponent` are upstream names. `GreenGeComponentDef` and
`RedGeComponentDef` are two of the 24 the semantic pass recorded as proposed
and not taken; re-derived here from the holders' names, as that doc asks.

**The component-definition rename.** In 15.21 the definition classes were
`<X>GeComponentDefinition`; from 15.22 they are `<X>GeComponentDef`. Five of
the old spellings landed together, each on a class whose successor upstream
already names: `EnvMesh`, `DragonCamp`, `Billboard`, `Red`, `Green`. Five
hashes, one rename, no way for chance to line them up.

**Runtime components.** `EnvMeshGeComponent` and `BillboardGeComponent`
complete the `<X>GeComponent` / `<X>GeComponentDef` pairing under
`GameEntityComponent`, and `MonarchRegionLocatorGeComponentDef` completes it
from the other side of the upstream-named `MonarchRegionLocatorGeComponent`.

## Everything else the pass named

| hash | name | what fixes it |
|---|---|---|
| `29c76c1f` | `EntityShapeData` | interface every `.shapeData` points at; child is the named `EntityPolygonData` |
| `2394da22` | `EntityCircleData` | its other child, `.radius` |
| `741fa51d` | `AreaTriggerShapeData` | interface `AreaTriggerDefinition.shapeData` points at |
| `4c9ed39e` `92024c11` `b1fe0ab4` | `AreaTriggerPolygonData`, `AreaTriggerBoxData`, `AreaTriggerCircleData` | its three children: vertices, Min/Max, radius - the same three-way split as the Entity set |
| `1f2e5fd0` | `MinimapTimerDefinition` | held by `NeutralCampDefinition`; its `.Group`/`.Style`/`.StyleSheet` types are the next three rows |
| `7f796784` `0c911666` `a21d6491` | `MinimapTimerGroupData`, `MinimapTimerStyleData`, `MinimapTimerStyleSheet` | one per field, each matching the field's own name |
| `5566d3a3` `8ed5df57` | `GameEntityGroup`, `GameEntityGroupHelper` | the helper's `.groups` links the group; `LocatorEntity.GroupHelper` holds the helper |
| `912bf9eb` `d2740d35` `29b5826a` | `MeshHelperDef`, `ScriptHelperDef`, `StateMachineHelperDef` | each is the type of a `<X>Helper`-named field or a field named after itself; `GroupHelper` is already upstream |
| `3aa4575e` | `GameEntityScriptDefinition` | held by `ScriptHelperDef.Scripts` |
| `881be5aa` | `SkinCharacterDefinition` | interface carrying exactly `SkinCharacterGeComponentDef`'s `CharacterRecord` + `Skin`, plus `IconHelper` |
| `dbff6738` | `IDeathGeComponentDef` | `identity` pass, `--prefix I` over the attested `DeathGeComponentDef`, 0.002 expected noise; the class is the interface `DeathGeComponentDef.definition` points at, and the meta flags it as one |
| `3ff79219` `4664ae0a` | `IShapeRenderInfo`, `IRegionBoundaryRenderInfo` | the two interfaces the two `.RenderInfo` fields point at |
| `880c52da` `76bc0857` | `RegionBoundaryInstanceParams`, `RegionBoundaryInstanceParamsBase` | `RegionBoundary.InstanceParams` names the first; the second unfolds to it exactly |
| `3f6832a1` | `MonarchMapLocator` | `MapScriptLocator` child whose only field is `List2 Embed MonarchTagPtr` |
| `64ee2fb1` | `GameEntityIconData` | the class eight `.Icon` links point at |
| `c60aca93` | `SequencerEntityTemplate` | template whose only content is `.SequencerStateMachine` |

**The two weakest.** `SequencerEntityTemplate` and `GameEntityIconData` rest
on a single hash match with no second slot to check against, and
`GameEntityIconData`'s run carried 2.49 expected noise. Both are coherent and
in-family; neither is proved the way the rest are. Anyone submitting this
upstream should split them off or say so.

## Shipped-data cross-checks

`bin-grep` over all 455 WADs, `--class MapPlaceableBase --subclasses`: 66,359
objects. Nine of the newly named classes have shipped instances, and the
instance names CommunityDragon resolves agree with the names:

- `AttackableEntityTemplate` has an instance named `Baron`.
- `NeutralCampEntityTemplate` has one named `DragonCamp`, and its only child is
  `DragonCampEntityTemplate`.
- `SuperwardEntityTemplate` has twelve distinct instances, all on Map11, all
  carrying `Characters/SRU_FaeLightWardPad/CharacterRecords/Root` and linking
  `SRU_FaeLightWardPad_Base_Uppies` / `_Alert_Uppies` through the component.
- `0x3c995caf`, still unnamed, is `MinionPath_Top`/`_Mid`/`_Bot` and
  `TopLaneHomeguardsPath` - a waypoint path beside `BezierPath`.

## What is left

Five live members of `MapPlaceableBase`, and 41 unresolved hashes across the
family and its closure. The structure of the remainder is settled even where
the names are not: suffix folding proves which hashes share a stem, so each
group below needs **one** string and the rest follow.

| stem state | slots waiting on it |
|---|---|
| `241780db` | `<X>Entity` 67c237c4, `<X>EntityTemplate` 4509678e, `<X>EntityPrefab` 61a40306, `<X>EntityDefinition` 4b998ced - the definition's one field is `MainScript` |
| `050a298b` | `<X>EntityTemplate` 59f5f97e, `<X>GeComponentDef` d3dda5e5, `<X>GeComponent` 34f2d06e - a polygon plus `DynamicLighting`, `SSAO`, `RenderParticles` and a quality byte, so a per-area render toggle |
| `b6c4d32a` | `<X>EntityTemplate` 0e4544df, `<X>GeComponentDef` fe3fdc0c, `<X>GeComponent` ef17c645 - a second sfx template, `SoundName` + `shapeData`, no `FillVolume` |
| `2d2eaa81` | `<X>GeComponentDef` fdb6b027 (no fields), `<X>GeComponent` 210fea10; the template is 9b321633 |
| `d3101cf4` | `<X>GeComponentDef` 15898c52 (0.5, 2.0, 25.0), `<X>GeComponent` a98b5b03 |
| `c7aa7d2a` | `<X>Entity` 49c3a539, `<X>EntityTemplate` 671ededf, `<X>Definition` 854a4f5b - all three live one patch only, 15.22 |
| `835c065b` | `<X>EntityTemplate` b43f710e, `<X>EntityPrefab` 1a299086 - the Red/Green test pair held by pointer rather than embedded |

Ruled out for these seven, so do not re-run them: `identity`, `delete`,
`mutate` over the full wordlist, `chain` depth 4 and 5, `force` depth 3 over
the top 1400, and the whole shipped-string corpus (3.6M forms) and both client
binaries (248k) probed against the stem states directly. Summed expected noise
~13, zero coherent hits. The unlock is new attested vocabulary, not more
probes.

Singletons still open, with what is known about each:

    3c995caf  IPath child beside BezierPath, .Segments List2 Vec3;
              instances MinionPath_Top/_Mid/_Bot, TopLaneHomeguardsPath
    5a9697c1  MapPlaceable child; .Vertices List2 Vec3 from 15.15, .Min/.Max
              before that, .type U32 from 16.5. No shipped instance.
    385c323f  base of the five <X>GeComponentDefinition classes, 15.16-15.21
    d738f7c9  mixin on GameEntity, GameEntityTemplate, GameEntityPrefab,
              LolGameEntity, MapDynamicSpotlight, MapDynamicPointLight
    1d12c4da  the IShapeRenderInfo implementation: Taper, bevel, Material,
              height, Margin
    58c2dd4f  the IRegionBoundaryRenderInfo implementation: Material
    b0e5f5b6  interface behind LolGameEntityDefinition.OptionalConfigs;
              children 82cab1b3 (lane, Position) and ddaf93fe (Group)
    313c0076  the IconHelper config: Icon link, Enabled, one bool
    7a1cab0d  one String field, texturePath, held by four .Icon fields

## Status

All 109 rows `status=pending`, `pr=-`. Nothing submitted upstream.

Fit to submit as it stands: the two shipped-string stems and the nine fields
above carry a string Riot wrote, which is the standard `light-regions` and
`exe-strings` met. The lattice names carry a slot argument, the standard
`gamemode-configs` and `viewcontroller-family` met; they should be described
the same way. `SequencerEntityTemplate` and `GameEntityIconData` are neither
and are flagged above.
