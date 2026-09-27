# Plan: shader reflection attestation

Compiled shaders ship with their reflection metadata intact. That metadata names
constant-buffer structs and every member, in declaration order, with real
casing. It is attestation, not a hash guess, and this repo has already landed
two batches from it one shader at a time. Nobody has run it across the cache.

## Why this outranks the guesser

The guesser proposes a name and the hash fails to contradict it. Reflection runs
the other way, like `exe-strings`: the probe set is nothing but bytes the game
shipped, so a hit is a name Riot wrote against a hash the game registered.

Two batches already came from a single shader each. `light-regions` (16 rows):
`LightRegionInfo_SharedDataBuffer` in `LightRegions.ps` declared a struct
`LightRegionRenderData` and its members, which named a class and six unknown
fields at once because the members pack into float4s in the same order and types
as the meta class. `gameplay-texture` (5 rows) came from the `GAMEPLAY_TEXTURE`
macros and `PS_GAMEPLAY_TEXTURE.ps`. `ssao-casing` turned on the shader's own
`OMIT_SSAO` define.

The corpus is also disjoint from what has been mined. Shader-reflection names
are compared by `strcmp` at effect-build time and exist only in data - mapgeo
headers, materials bins, compiled reflection - **never as exe strings**. The
`exe-strings` pass structurally could not have seen them.

And reflection carries more than strings. Member names arrive in declaration
order with types, so a struct corroborates *structurally*, not just by a hash
collision. That is strictly stronger evidence than the method that landed 220
names.

## What actually ships (measured, retail 16.x)

| wad | contents |
|---|---|
| `ShaderCache.dx11.wad.client` (41.5 MB) | 2,178 chunks, all `assets/shaders/generated/shaders/**/*.{ps,vs}-dx11[_N]`; 22 unresolved paths |
| `Bootstrap.windows.wad.client` (51.9 MB) | the engine-fixed shaders, `LightRegions.ps` among them |
| `Shaders/Shaders.wad.client` (34.7 MB) | 141 `.tex` plus `data/shaders/shaders.bin`; **no compiled shaders** |

Windows retail is **dx11 only**. The `.glsl`, `.metal`, `.dx9` and `.dx9sm3`
variants in the `.game` hashtable ship on other platforms. Chunks are record
sequences, not bare blobs: `u32 size` (= DXBC totalSize + 1), the DXBC
container, then one trailer byte. A chunk bundles every permutation of one
shader stage - up to 92 observed - which is why prior work counted 71,774 DXBC
blobs across the cache.

## On "compile with all features enabled"

Right instinct, wrong mechanism, and the correct one is cheaper.

We cannot build a maximal permutation: Windows ships no HLSL source
(`assets/shaders/hlsl/` is a source *directory name* in the path table; the
files under it are compiled outputs), so a permutation can only be *selected*,
never constructed.

Selection is also the wrong operation. Permutations are not nested supersets -
`#ifdef` blocks add whole constant buffers and resource bindings, and feature A's
block may never co-occur with feature B's. A "fattest" permutation is a local
maximum, not a union.

**So: parse every permutation and union the reflection names.** We walk the
container anyway, so the union is free and strictly dominates any single build.
Deduplicate on a content hash of the reflection chunk before parsing - the 71,774
blobs collapse hard, since most permutations differ only in the code chunk.

## What the reflection chunk yields

Per blob, from `RDEF`:

- constant buffer names (`LightRegionInfo_SharedDataBuffer`)
- **struct type names** (`LightRegionRenderData`) - these live only in the
  shader-model-5 extended type descriptor, so the parser must handle it or the
  single highest-value field is silently absent
- member names, in declaration order, with type, offset and array extent
- resource binding names: textures, samplers, buffers
  (`BAKED_DIFFUSE_TEXTURE`, `IBL_CUBE_MAP_ARRAY`, `ClusterData`,
  `MantisSharedMapParameters`)

`ISGN`/`OSGN` give vertex semantics (`POSITION0`, `TEXCOORD6`). Useful for the
vertex-format work, not for meta names. `shaders.bin` is a dead end here: it
holds only data-driven `CustomShaderDef`s, never the fixed engine shaders,
though its feature-macro list is worth keeping for the casing pass.

## Three passes over one extraction

**A. Whole-name attestation.** Hash every extracted name against the unresolved
sets, exactly the `exe-strings` shape. Whole strings only, never a fragment or a
prose token - the tiering that `data-section-strings` used applies unchanged.

**B. Structural alignment.** For each reflected struct, align its member
sequence against unresolved class layouts in `db/meta.db.json` by order and
type. This is the `light-regions` method generalized and it is where the value
is: it names a class and its fields together, and already-known members pin the
alignment so a match is corroborated beyond the hash. `db/` carries types and
declaration order but no offsets; the format doc allows adding `offset`/`size`
additively from the raw dumps, and `light-regions` was matched from the type
sequence and float4 packing alone, so this is not a blocker.

**C. Casing and the held-back batches.** Reflection names carry real casing.
Two batches are currently marked DO NOT SUBMIT purely for want of a string:
`light-regions-unproven` (6 rows) and `map-graphics-unproven` (5 rows). A full
sweep can promote them or refute them, and either outcome is worth having.

## Noise budget

`probes x target states / 2^32` over 2,421 unresolved classes + 4,483 unresolved
fields = 6,904 states. One expected chance hit needs **622,000 distinct probes**,
so a reflection probe set of any plausible size is far inside budget: 50k probes
is 0.08 expected chance hits, comparable to `exe-strings` at 0.04. The budget is
not the constraint here, which is the whole point of an attested probe set. Print
the count anyway, per the house rule.

## Known traps

- **Shader casing is its own namespace.** A cbuffer `Priority` against the known
  bin field `priority`. Treat casing as corroborated only when sibling bin fields
  agree; names still enter under the `scripts/names.py` rule.
- **A CPU-side struct may carry members the GPU block does not.**
  `LightRegionRenderData` had exactly one such field, and it stayed unnamed.
- **Permutation suffixes are not names.** `_0`, `_100`, `_1100` are permutation
  groups in the path, not vocabulary.
- **Do not let pass B report an alignment as a hit without the type sequence
  matching.** Order alone is weak; order plus types plus a known member is not.

## Landing rules

Unchanged. A reflected name is attestation, so it clears the bar `light-regions`
and `exe-strings` met, but the per-name evidence still goes in a doc under
`docs/`, the batch note records the method, and `hashtool add` enforces casing.
Anything resting on structural alignment without a matching string goes to an
`-unproven` batch, as `light-regions-unproven` did.

## Optional, if the input can be obtained

The `.glsl` and `.metal` variants are **text**, not bytecode - full identifier
names with no permutation problem and no reflection parser. They are in the
`.game` path table but not in the Windows install. If a Mac or mobile build is
reachable, that source is a richer probe set than anything above and should be
run first. Not a dependency; the dx11 path stands alone.
