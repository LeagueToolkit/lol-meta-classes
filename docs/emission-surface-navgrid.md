# emission-surface-navgrid

The classes around `VfxEmissionSurfaceData` and the nav grid region tags it reaches
through its second pointer: 11 class names and 2 field names. None is attested by a
string in the client. Each is an FNV-1a match held in place by structure: a shared stem,
the registration order of the reflection registrars in the 16.18 Windows client, or both.

Registration order is roughly alphabetical by class name inside one run, so the named
neighbours of an unnamed registrar bracket its name.

## Evidence

| hash | table | name | what fixes it |
|---|---|---|---|
| f8b81c77 | field | ParticleSpawnDataGenerator | solved together with 1519e8d2 as one stem under the affixes `""` and `I`: a 64-bit condition over 4-token stems from a 3,300-word vocabulary (about 1e14 stems, 6e-6 expected chance hits), one solution. The pairing mirrors `EmissionSurface` / `IVfxEmissionSurface` on the same class |
| 1519e8d2 | class | IParticleSpawnDataGenerator | the other half of that pair. Interface, registered between `FloatGraphMaterialDriver` and `IVfxEmissionSource` |
| 671b7351 | class | NavigationGridParticleSpawnDataGenerator | the only subclass of 1519e8d2. Two free tokens in front of the cracked stem, 0.03 expected chance hits, one hit. Registered between `MutatorMapVisibilityController` and `RefundAbilityPointsCheat`. The client implements it with a nav grid lookup |
| 526478f0 | class | VfxEmissionLinkedMeshData | third subclass of `IVfxEmissionSurface`, next to `VfxEmissionMeshData` and `VfxEmissionSkeletonData`. About 2,000 candidates of the shape `VfxEmission<word>Data`. Registered directly before `VfxEmissionMeshData`. The client samples a mesh that game code links to the effect instance |
| 2bfb084c | class | NavGridRegionGroupDefinition | element of `NavGridConfig.RegionGroups`. Registered directly after `NavGridConfig` |
| f6f4bb5f | class | NavGridRegionTagDefinition | element of the region group's `tags`. Registered directly after 2bfb084c |
| d82714cc | class | NavGridTerrainTagDefinition | element of `NavGridTerrainConfig.tags`, same shape as the region tag. Registered directly after `NavGridTerrainConfig` |
| f42cd443 | class | NavGridRegionTagsLink | fields `NavGridConfig` (link), `RegionTags`, `groupName`. Registered between f6f4bb5f and `NavGridTerrainConfig`. Two free tokens, 0.06 expected chance hits: the weakest of the batch |
| 2d00e4da | class | NavGridRegionInputData | 2d00e4da, 6b91544a and 41c19efe are three consecutive registrars after `NarrativeBarksList` and before `OverrideDeferredKeyBindingsConfig`. The three names sort in exactly that order |
| 6b91544a | class | NavGridRegionRenderData | game mode config holding `VfxSystems`, a list of 41c19efe, plus blur and fade settings |
| 41c19efe | class | NavGridRegionVfxData | fields `name`, `VfxMask`, `VfxSystem`, `InputData` (pointer to 2d00e4da). The client registers each `name` as a tag of the nav grid's `Vfx` region group |
| d2807c60 | class | VfxEmbeddedMaterial | subclass of `VfxMaterialContainer` with one embed field `Material`. About 250 candidates. Registered between `VfxEffectorDefinition` and 526478f0 |
| ec01928c | field | IsGameplayGroup | bool on 2bfb084c. One free token behind `is`, 2e-4 expected chance hits |

## 41c19efe against upstream

Upstream resolves 41c19efe to `VfxPrimitiveCameraSegmentSeriesBeam`. Both names hash to
it, so one is a chance match. The class has no base, is not a primitive, carries `name`,
`VfxMask`, `VfxSystem` and `InputData`, and registers between `NarrativeBarksList` and
`OverrideDeferredKeyBindingsConfig`, where a name starting with `Vfx` cannot sit.
Shipped instances are the `VfxSystems` entries of the Summoner's Rift game mode config
(`FaeLights_Order`, `FaeLights_Chaos`, `Squid_Ink_Chaos`). The override takes the name
that fits.

## Negatives

- No string in the client spells any of these names. Casing is by convention, not
  attested.
- `IsGameplayGroup` is set on the `Vfx`, `GameplayArea`, `GameplayLane` and
  `GameplayPOI` groups and clear on `GameplayTurretLines`, which the name alone does
  not explain. No reader was traced.

## Left

| hash | on | known |
|---|---|---|
| 50d6d98f | NavGridRegionGroupDefinition | bool |
| 46edf5aa | NavGridRegionRenderData | f32 |
| cd986599 | NavGridRegionRenderData | bool |
| 1c45cf5c | NavGridRegionVfxData | color, set beside `VfxMask` |
| 18273a68 | NavGridRegionVfxData | file |
| 499d3e3d | class registered after NavGridRegionVfxData | fields `ShowVfx` and one f32 |
