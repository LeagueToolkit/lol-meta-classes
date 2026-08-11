# clustered-lighting

`MapDynamicLighting` (0xef02c9a2, MapGraphicsFeature) and its embedded config
class: 10 unresolved hashes to 4, 6 names landed in one batch.
The class reflects the standard clustered
(froxel) light-culling scheme; the shipped shader side carries `CLUSTER_MAP`,
`CLUSTER_DATA_BUFFER`, `CLUSTER_SPATIAL_INDEX`, `EMPTY_CLUSTER_SPATIAL_INDEX`,
`WORLD_TO_CLUSTER_TRANSFORM`, `CLUSTER_MAX_CLAMP` and `USE_DYNAMIC_LIGHTING`,
which attest the vocabulary but no field spelling.

## Evidence

| hash | name | what fixes it |
|---|---|---|
| 6438040c | MaxNumClustersX | fold: the three u32s unwind to one shared stem state (0x824da33c) under trailing x/y/z; a 3.1M cluster-domain search on that state returns exactly MaxNumClusters, 0.0007 expected chance hits. Tier 3. |
| 6538059f | MaxNumClustersY | same fold, trailing y. Tier 3. |
| 66380732 | MaxNumClustersZ | same fold, trailing z. Tier 3. |
| 30c224ec | ClusteredLightingInfo | single hash; the class is the embed target of MapDynamicLighting 0x0be053e8 and holds the grid config. Tier 4. |
| 86bf2c9e | MinClusterSize | single hash; Vec3 defaulting [100, 100, 100] beside the grid dims. Tier 4. |
| ed17c5c8 | DynamicLightsEnabled | single hash; the only bool on MapDynamicLighting defaulting true. Tier 4. |

Defaults corroborate the shape: a 32x2x32 grid floored at 100 world units per
cell, the 2-slice Y axis matching a fixed top-down camera.

## Negatives

- Zero shipped instances of MapDynamicLighting across all installed WADs as of
  16.14/16.15, so no bin data can attest anything here yet.
- No string in the shipped client or the CommunityDragon corpora matches any
  of the six names; the CLUSTER_*/USE_DYNAMIC_LIGHTING macros are the closest
  vocabulary and none contains a field spelling.
- The stem search (3.1M probes, one target state) is the only recorded run;
  the tier-4 names came from small domain-restricted searches whose probe
  counts were not recorded.

## Left

| hash | on | known |
|---|---|---|
| 8a0d61c2 | ClusteredLightingInfo | u32 8192; plausibly a max light or cluster-entry count |
| 0be053e8 | MapDynamicLighting | the Embed field holding ClusteredLightingInfo |
| 02451a0b | MapDynamicLighting | bool false |
| b41ddb79 | MapDynamicLighting | bool false; the two bools are likely a debug visualisation and a gate on the DynamicLightsEnabled pattern |

Runtime was never traced; that is the open thread that could attest the tier-4
spellings and crack the leftovers.

## Status

All rows `pending`. Nothing here rests on an attested string: the axis triple
is fold-proven, the three tier-4 names are single-hash and must be called out
as such in any upstream PR.
