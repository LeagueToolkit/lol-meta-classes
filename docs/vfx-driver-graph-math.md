# vfx-driver-graph-math

The 16.x math nodes under `IVfxBaseDriver`, added after the first
`vfx-driver-graph` batch.

**39 live unnamed -> 13.** 26 class names and 1 field name landed. No shipped
string attests any of them. Shipped data (two HoL_26 systems) instantiates only
`0x1d04cfa7` and `0x7cc5a312`, and names nothing.

## What fixes each name

Every class also passes the base check: the kind in each name matches its
`IVfx<Kind>Driver` base, and the `From` kind matches the `Input` type.

### Kind folding back to `Vfx` (tier 3)

Folding `<Kind>From<Kind>Driver` or `<Kind>LogicDriver` out of the hash
leaves exactly `fnv("Vfx")`. These are the templates of the named
`VfxFloatFromVector4Driver` and `VfxVector3LogicDriver`.

| hash | name |
|---|---|
| `799a50ac` | VfxFloatFromVector2Driver |
| `01a95dbf` | VfxFloatFromVector3Driver |
| `dab8397c` | VfxVector2FromVector3Driver |
| `0fd9b311` | VfxVector2FromVector4Driver |
| `e3a77546` | VfxVector3FromVector2Driver |
| `76c70374` | VfxVector3FromVector4Driver |
| `14daebe5` | VfxVector4FromVector2Driver |
| `9c5c4342` | VfxVector4FromVector3Driver |
| `93fdd326` | VfxVector2LogicDriver |

The `From` row covers both the `Select` classes and the `Fill` (extend)
classes, so the pasted family table's Select/Extend split is one template.

### Shared stems, one word fills a row (tier 3)

Kind-digit relations prove that each row shares a stem ending in
`<Kind>Driver`. The stem was then reached from `Vfx`.

| hashes | name | stem found by |
|---|---|---|
| `dfe3528c` `c2685905` `1d708462` `4b5aa9eb` | VfxCondition{Float,Vector2,Vector3,Vector4}Driver | 2-word MITM, noise 0.01 |
| `d6738324` `168d2f0d` `95182f0a` `ff2348d3` | VfxDivideByFactor{Float,Vector2,Vector3,Vector4}Driver | anchored on `VfxDivide`, noise 0.015 |
| `997d54ab` `64707da8` `a995ecc5` | VfxDivideByComponent{Vector2,Vector3,Vector4}Driver | 4-word focused vocab, noise 0.21 |
| `399295b9` `65e1b9a2` `3624c20b` `791d4f88` | VfxBreakVector{2,3,4}ToFloatsDriver, VfxBreakVector4ToVector2sDriver | `Floats`/`Vector2s` swap stem, then predicted siblings hashed exact |
| `414d1503` `c5e53afa` | VfxUniformRangedRandomDriver, VfxSquareRootRangedRandomDriver | `RangedRandomDriver` tail predicted, sibling hit at noise 0.034 |

- `Break...To...` reads backwards: these nodes build a vector from their
  parts. The lattice fixes the name anyway, since 4 hashes land on 1 template.
- Word boundaries in `DivideBy`, `ByComponent`, `ByFactor` and `SquareRoot`
  follow the repo PascalCase rule over table words. They are not attested as
  compounds.

### Single hash (tier 4)

| hash | name | what fixes it |
|---|---|---|
| `b1ea6248` | Fill | field on the three `From` extend classes (`F32` or `Vec2` beside `Input`); single dictionary word, chance about 1e-6 |

## Negatives

| run | expected noise | result |
|---|---|---|
| remaining 19 targets, `Vfx`[Kind][W]+[W][Kind]`Driver` | 1.57 | 3 junk |
| 15 targets, `Vfx`+W1+W2+[Kind]`Driver` | 0.47 | 1 junk |
| broadcast stem, `Vfx`+W1+W2+glue+`{Vector2,Vec2,Float2,}2Driver` | 0.31 | dry |
| broadcast/empty, 30k corpus word + small word | 0.19 | dry |
| curve/factor/material/random rows, 4-word single target | 188 | about 188 junk, 0 lattice |
| kind lattice `Vfx`+A+Kind+B+`Driver`, A,B <= 2 words, pivots Float, Vector2, Color | raw about 25k each | 0 confirmed |
| same lattice with a 30k corpus word on one side | raw 750 | 0 confirmed (controls recovered) |

## What is left

| hash | what is known |
|---|---|
| `9a2d73f2` `def9bfd5` `7c387678` | broadcast Float -> Vec{2,3,4}; proven shared stem `0x94de5689` before `Vector{n}Driver` |
| `1d04cfa7` `3eb74cbe` `2d42ea41` `44852d75` `7cc5a312` | curve leaves (15.22-16.1); `1d04cfa7` field `graph` renamed `Float` at 15.23 |
| `2959e51d` `5ff9a600` `7a39d82b` `fc39b68c` | factor-sampled curves, all 16.19 |
| `88406627` | Vec3, no fields, 16.18 |

The curve leaves have `IVfxMaterialDriver` twins (15.14) that share their stem:

| stem state | `+Driver` | `+MaterialDriver` |
|---|---|---|
| `0x6700ffbd` | `1d04cfa7` (ValueFloat) | `fbef6376` (ValueFloat) |
| `0x036d3b0c` | `7cc5a312` (ValueColor) | `c5b349bf` (ValueColor) |

Cracking either side names both.

## Status

All 27 rows `pending`. Every row except `Fill` is tier 3 and fit to submit.
Flag `Fill` as single-hash in any upstream PR.
