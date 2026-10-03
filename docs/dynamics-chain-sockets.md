# dynamics-chain-sockets - campaign record

Record for the `dynamics-chain-sockets` batch; its `batches.jsonl` note points here. Method:
the `crack-family` skill (`.claude/skills/crack-family/SKILL.md`). The target was the unnamed
remainder of three related groups: the dynamics chain pose modifier and the classes under it,
the physics simulation settings it reads, and the two socket classes. 5 classes and 39 fields
are named here. 1 class and 7 fields stay open.

Each slot was first read in the 16.18 client (what the reader of the field does), then
searched. The reader fixes the meaning. The search fixes the spelling. Two of the searches
used the repo wordlist with `hash-guesser --only`. The rest used a hand-built list of 200 to
420 physics and animation words, restricted to these hashes, because the names turned out
to use words the corpus does not hold (`Rod`, `Lateral`, `Sim`, `Freeze`, `Substeps`,
`Convergence`). Those words did not go through the calibration gate, so every run below
states its probe count and its expected chance hits.

## The names

| hash | name | on | what fixes it | tier |
| --- | --- | --- | --- | --- |
| 03ca1fe0 | DynamicsJointTreeGroupData | class | three names, one stem | 3 |
| 23d901d3 | JointTreeGroups | DynamicsChainRigPoseModifierData | predicted from the class name | 3 |
| b0f77798 | JointTrees | DynamicsJointTreeGroupData | predicted from the class name | 3 |
| c11c69b4 | RootJointName | DynamicsJointTreeData | pair in one run, reader | 3 |
| ea59c349 | ExcludeJointNames | DynamicsJointTreeData | pair in one run, reader | 3 |
| bdaa6f16 | GlobalEnvelope | DynamicsChainRigPoseModifierData | sibling of `Envelope`, reader | 3 |
| c20b2e61 | PhysicsSimLocalSettings | class, and the field that holds it | Local and Global pair | 3 |
| 665c9fe7 | PhysicsSimGlobalSettings | class | predicted from the pair | 3 |
| 97f2e0f1 | GravityScale | PhysicsSimLocalSettings | pair in one run, reader | 3 |
| bc016dbd | GravityOverride | PhysicsSimLocalSettings | pair in one run, reader | 3 |
| 13465a3b | ConstraintSubsteps | PhysicsSimGlobalSettings | three names, one stem, reader | 3 |
| f3ca8ece | ConstraintIterations | PhysicsSimGlobalSettings | same | 3 |
| a791f85d | ConstraintSolverType | PhysicsSimGlobalSettings | same | 3 |
| 8108c459 | ConvergenceThreshold | PhysicsSimGlobalSettings | single hash, reader | 4 |
| 5dbadfca | CollisionResolutionMode | PhysicsSimGlobalSettings | single hash, reader | 4 |
| cb22ec08 | AnimPoseAttraction | DynamicsChainProperties | rename boundary, shared stem | 2 |
| 61906396 | AnimPoseStiffnessStart | DynamicsChainProperties (removed) | same stem | 2 |
| b379dd33 | AnimPoseStiffnessEnd | DynamicsChainProperties (removed) | same stem | 2 |
| c127258f | EnvelopeStart | DynamicsChainProperties (removed) | fold of a string-attested name | 2 |
| f5f7f81a | EnvelopeEnd | DynamicsChainProperties (removed) | fold of a string-attested name | 2 |
| 967e8270 | RodStretchStiffness | DynamicsChainProperties | lattice of four, reader | 3 |
| d30ce0f0 | RodShearStiffness | DynamicsChainProperties | same | 3 |
| 583d67c2 | RodBendStiffness | DynamicsChainProperties | same | 3 |
| b055bd6a | RodTwistStiffness | DynamicsChainProperties | same | 3 |
| 22fa6a8c, d1ae3975 | RodStretchStiffnessStart, RodStretchStiffnessEnd | DynamicsChainProperties (removed) | fold | 3 |
| 6521e60c, 027768f5 | RodShearStiffnessStart, RodShearStiffnessEnd | DynamicsChainProperties (removed) | fold | 3 |
| db82105a, e2a1edef | RodBendStiffnessStart, RodBendStiffnessEnd | DynamicsChainProperties (removed) | fold | 3 |
| 2dd6ec92, 429f9d17 | RodTwistStiffnessStart, RodTwistStiffnessEnd | DynamicsChainProperties (removed) | fold | 3 |
| 6e48edbe | UseRodPhysics | DynamicsChainProperties | `Rod` stem of the four, reader | 3 |
| 3cc9ab8a | GenerateLateralLinks | DynamicsJointTreeGroupData | `LateralLink` pair, reader | 3 |
| 10278985 | LateralLinkMaterial | DynamicsJointTreeGroupData | `LateralLink` pair, reader | 3 |
| ddf17bcb | SocketDefinitionSingleJoint | class | sibling pattern | 3 |
| 165b1803 | SocketDefinitionWorld | class | sibling pattern | 3 |
| 5fce2dff, 5ece2c6c, 61ce3125 | FreezePositionX, FreezePositionY, FreezePositionZ | SocketDefinitionSingleJoint | 2 x 3 lattice, reader | 3 |
| dcbfa078, ddbfa20b, debfa39e | FreezeRotationX, FreezeRotationY, FreezeRotationZ | SocketDefinitionSingleJoint | 2 x 3 lattice, reader | 3 |

**The joint tree names.** The modifier holds a list of 03ca1fe0, and each of those holds a
list of `DynamicsJointTreeData` plus one `DynamicsChainProperties`. The class came from
`force` depth 3 over a 422-word list under the prefixes none, `Dynamics` and
`DynamicsChain`, with and without the suffix `Data`: 225,989,862 probes, 2 target states,
0.105 expected, one hit. The two list fields were then predicted by hand from the class name
before hashing (33 probes): the list of groups is `JointTreeGroups`, the list of trees is
`JointTrees`. In the client, the root field is the joint the walk starts from and the list is
the set of joints the walk does not simulate, which is what `RootJointName` and
`ExcludeJointNames` say. Those two came from one `force` depth 3 run over the 422-word list
against the 6 open hashes of the class and its holder: 75,329,954 probes, 0.105 expected.

**GlobalEnvelope** (bdaa6f16). An F32 on the modifier, default `1.0`. The client multiplies
it into the per-joint `Envelope` weight when it writes the simulated joints back. `Envelope`
is a shipped string. `force` depth 2 over the repo wordlist against all 26 open fields of
the family: 10,715,802 probes, 0.065 expected, three hits (this one and the two gravity
fields).

**The settings pair.** c20b2e61 is both a class and the modifier field that embeds it. Its
two fields are an F32 with default `1.0` and an optional vector with no default. The solver
takes its gravity as the first times the second, and uses the gravity of class 665c9fe7 when
the second is not set. That class has a vector field already named `Gravity`, default
`(0, -981, 0)`. `PhysicsSimLocalSettings` came from `force` depth 2 over the union list under
the prefixes none, `Dynamics` and `Physics`: 34,567,890 probes, 10 target states, 0.080
expected. `PhysicsSimGlobalSettings` was then predicted from it (13 hand probes) and landed
on a class that has existed since 16.6. `GravityScale` and `GravityOverride` are from the
run under GlobalEnvelope.

**The constraint fields.** On `PhysicsSimGlobalSettings`: a U16 with default `50` that the
solver divides the frame time by, a U16 with default `1` that bounds the passes of each
substep, and a U8 that decides whether a constraint solver is created. `force` depth 2 over
the union list against the 5 open fields of the class found `ConstraintSubsteps` and
`ConstraintIterations` (11,522,630 probes, 0.013 expected), and `force` depth 3 over the
422-word list found `ConstraintSolverType` (75,329,954 probes, 0.088 expected).

**ConvergenceThreshold and CollisionResolutionMode.** The other two fields of the class. The
first is an F32, default `1e-5`: the solver stops the passes of a substep when the largest
constraint error falls below it. The second is a U8, default `0`: it selects whether
collisions are resolved once before the constraint passes or after every pass. Each came
from a three-word run over a hand-built list (below). Each is a single hash with a reader
that fits and no second name on the same stem, so both are tier 4.

**AnimPoseAttraction** (cb22ec08). The slot was renamed twice. In 16.8 it was the pair
`StiffnessStart` / `StiffnessEnd`. From 16.9 to 16.13 it was a pair whose stem is f4844e76.
From 16.14 it is cb22ec08. Unfolding the suffix `Attraction` from cb22ec08 and the suffix
`Stiffness` from f4844e76 gives the same state, 752d91dd. So the two names share a stem and
end in those two words. The stem resolved to `AnimPose` in a `force` depth 2 run over the
union list with `--suffix Attraction`: 11,522,630 probes, 1 target, 0.003 expected. The
reader agrees: the value is the rate at which a simulated joint is pulled back to its
animated pose.

**EnvelopeStart and EnvelopeEnd.** `Envelope` is a shipped string, and the hashes of the two
removed fields fold back to it under `Start` and `End`.

**The rod names.** `DynamicsChainProperties` has four fields with default `1.0` that the
client reads only when the Bool 6e48edbe is set. With it set, the chain gets two extra
constraints per segment, and the four values become their stiffness along the segment
(967e8270), across it (d30ce0f0), in bending (583d67c2) and in twisting (b055bd6a). One run
of 19,968 hand probes (16 prefixes, 24 roots, 26 suffixes, both orders) against 17 hashes,
0.00008 expected, gave four hits, one per field, each on the field with that role. The
Start and End hashes of 16.13 follow by fold. `UseRodPhysics` came from the three-word run
below and shares the `Rod` stem.

**The lateral link names.** 3cc9ab8a is a Bool: when set, the client adds distance
constraints between joints of the same depth in neighbouring trees of a group. 10278985 is
a U8, default `2`, that selects the softness of those constraints from seven fixed values.
The Bool came from the second three-word run and the U8 from the first, and they share
`LateralLink`.

**The socket names.** `SocketDefinitionBase` has two children. ddf17bcb holds `ParentJoint`,
`PositionOffset` and `RotationOffset`. 165b1803 holds only `PositionOffset`, and the client
resolves it against the root of the character, with no joint. Both names follow
`SocketDefinition<X>`. The six Bools of the first class were added together in 16.18, at
consecutive offsets. Their hashes differ by the FNV prime in X, Y, Z order within two
triples, and the client replaces the matching axis of the position or of the rotation with
its value in the bind pose. The six names cover the six hashes one each, in offset order.

Casing follows the rule of the repo. The six freeze names are entered as `Freeze...`. No
shipped string spells any name in this batch, apart from the `Envelope` the two folds rest
on.

## The hand-list runs

| run | words | probes | targets | expected | hits |
| --- | --- | --- | --- | --- | --- |
| prefix, root, suffix grid | 16 x 24 x 26, both orders | 19,968 | 17 | 0.00008 | the four `Rod...Stiffness` |
| three words, list A | 210 | 9,305,310 | 16 | 0.035 | UseRodPhysics, LateralLinkMaterial, ConvergenceThreshold |
| three words, list B | 237 | 13,312,053 | 8 | 0.025 | GenerateLateralLinks, CollisionResolutionMode |

## Negatives

Summed expected noise of every run in this campaign, hits and misses: about 2.4. Do not
re-run these without new vocabulary.

- 03ca1fe0 under `DynamicsChain`, `Dynamics` and no prefix, with and without `Data`:
  `identity`, `delete`, `force` depth 2 (0.015), `chain` depth 3 and 4 (0.012), `force`
  depth 3 over the top 500 under each prefix (0.058 each). 0 hits. The class does not start
  with `DynamicsChain`.
- All 26 open fields: `identity` and `delete` with and without `m`, `force` depth 2 with `m`
  (0.065), `chain` depth 3 (0.006) and depth 4 (0.070). 0 hits beyond the three listed.
- The Bools 04e1a2c8, 56670932, 23382109 and the String bb1d1aac: `force` depth 3 over the
  422-word list (0.123), `force` depth 2 over the union list under `Use`, `Enable`, `Is`,
  `Apply`, `Ignore` (0.080) and under `JointTree`, `JointTreeGroup`, `PhysicsSim`,
  `AnimPose`, `Collision` (0.094), `force` depth 3 over the top 390 (0.083), and both
  three-word runs above. 0 hits.
- bb1d1aac alone: `force` depth 3 over the top 700 (0.080). 0 hits.
- Class f7c4193e and its field cff65b54: both three-word runs above. 0 hits.
- The five open fields of `JointOrientationRigPoseModifierData` (a57f0269, ae1cbd5f,
  1a30a486, 57722010, 420b233d): only the all-field runs. 0 hits.

## What is left

- **04e1a2c8**: Bool on `DynamicsJointTreeGroupData`, default false. The curve of each
  parameter runs over the length of the joint tree. False: each tree uses its own longest
  branch. True: every tree of the group uses the longest branch of the group.
- **56670932**: Bool on the same class, default true. True: the rest length of each segment
  is taken from the animated pose on every frame. False: it stays at the bind length.
- **23382109**: Bool on the same class, removed in 16.19. True: joints with no simulated
  child get a collision radius of zero.
- **bb1d1aac**: String on the modifier. The path of a file of collision spheres and
  capsules.
- **f7c4193e**: a settings class with one F32, cff65b54, default `4.0`. With `UseRodPhysics`
  set, it scales the time in the pull of `AnimPoseAttraction`.
- **9df0bf7f**: Bool on `DynamicsJointTreeData`, 16.8 only. `FlipXAxis` matches by a fold on
  the known field `FlipX`, with about 0.25 expected chance hits for that kind of match and
  no reader left to check. Not entered.
- The five `JointOrientationRigPoseModifierData` fields.

## Status

All rows are `pending`. The tier 2 and tier 3 rows are fit to submit. The two tier 4 rows
(`ConvergenceThreshold`, `CollisionResolutionMode`) go in a separate upstream PR, flagged.
No shipped instance of any of these classes exists, and no shipped string spells the names,
so an upstream PR must say so.
