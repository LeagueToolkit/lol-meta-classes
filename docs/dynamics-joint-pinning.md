# Dynamics joint pinning

Reversing doc for `dynamics-joint-pinning`. Its `batches.jsonl` note points here.

One value class that is new in PBE `16.21`, and the field that holds it. The batch names
1 class and 3 fields. The family is the dynamics chain pose modifier of
[dynamics-chain-sockets](dynamics-chain-sockets.md).

```
DynamicsJointTreeData                  339ecf91   48 B, 24 B before 16.21
    +0   PinningData        aa3e5b12   Embed DynamicsJointPinningData
    +24  ExcludeJointNames  ea59c349   List2 of Hash
    +40  RootJointName      c11c69b4   Hash

DynamicsJointPinningData               f9b5b50e   24 B
    +0   PinnedJointNames   0c287d6b   List2 of Hash, default empty
    +16  PinRoot            b7ab82e4   Bool, default true
```

Offsets and sizes are from the registration functions of the 16.21 PBE Windows client,
build `8255794`. The dump of build `8264391` has the same two classes.

## What the client does

Read from build `8255794`. Not tested in game.

- The modifier builds each joint tree once. It walks from `RootJointName` through every
  descendant joint and gives each joint that is not excluded one particle.
- A particle is pinned if its joint is in `PinnedJointNames`, or if its joint is the tree
  root and `PinRoot` is true. Before 16.21 the client pinned the tree root and no other
  joint.
- Each entry of `PinnedJointNames` is the FNV-1a hash of a lowercased joint name, the same
  form as `RootJointName`. An entry that the skeleton does not have matches no joint.
- A joint in `ExcludeJointNames`, or below one, has no particle, so an entry of
  `PinnedJointNames` for it has no effect.
- A pinned particle is set to the animated position of its joint on every frame. The
  simulation does not move it. Its children are simulated from that position.
- The client writes the rotation of a pinned joint and leaves its position as animated.
  Without `UseRodPhysics`, a pinned joint with exactly one simulated child is turned toward
  that child.
- With `PinRoot` false and the root not in `PinnedJointNames`, the root is simulated: the
  client integrates it under gravity and pulls it toward its own animated position at the
  rate of `AnimPoseAttraction`. The limit cone and the length restore need a parent, so
  the root has neither. The client writes both the position and the rotation of the root.
- The solver still reads a physics manager object that no code in the build creates. The
  retail 16.19 client crashed on that read when a unit with a dynamics chain spawned. The
  read is unchanged in `8255794`.

## Evidence

| hash | name | on | what fixes it | tier |
|---|---|---|---|---|
| f9b5b50e | DynamicsJointPinningData | class | one run, registration order, reader | 3 |
| aa3e5b12 | PinningData | DynamicsJointTreeData | one run, the class name without `DynamicsJoint` | 3 |
| 0c287d6b | PinnedJointNames | DynamicsJointPinningData | one run, reader, sibling `ExcludeJointNames` | 3 |
| b7ab82e4 | PinRoot | DynamicsJointPinningData | one run, reader | 3 |

The reader came first. The tree build function reads the list at +0 and the Bool at +16 of
the embedded value in one expression: `pinned = joint in list || (joint is the tree root &&
bool)`. The list of words for the run was then written from that role.

**The run.** 20 prefixes x 71 role words x 40 object words x 29 suffixes, in both orders of
role word and object word: 3,191,506 distinct names against the 4 hashes, 0.0030 expected
chance hits. Four names matched, one per hash, and each matched the slot that has that role:
the Bool that pins the root, the list of pinned joints, the class, and the field that holds
the class. All four share the stem `Pin`.

**Registration order.** The client registers classes in near-alphabetical order of their
names. The registration function of f9b5b50e is between those of
`DynamicsChainRigPoseModifierData` and `DynamicsJointTreeData`, and
`DynamicsJointPinningData` sorts between those two names.

**Field names.** `PinningData` is the class name without the prefix `DynamicsJoint`, the
same pattern as `ChainProperties` for `DynamicsChainProperties`. `PinnedJointNames` has the
form of `ExcludeJointNames` on the holder class, and both are lists of joint name hashes.

## Ruled out

- No name of the batch is a string in the 16.21 PBE Windows client.
- The two F32 fields that 16.20 added to the settings classes do not share a stem under the
  suffixes `Scale`, `Multiplier`, `Override`, `Scalar`, `Factor`, `Mult` and `Modifier`,
  and a role list of 51,289 names (0.00004 expected) matched neither.

## What is left

| hash | what is known |
|---|---|
| f057d92a | `F32` on `PhysicsSimLocalSettings`, default 1.0, since 16.20. The solver multiplies it with 635d340e |
| 635d340e | `F32` on `PhysicsSimGlobalSettings`, default 1000.0, since 16.20 |

The reader of the product was not found. The open hashes of
[dynamics-chain-sockets](dynamics-chain-sockets.md) are unchanged.

## Shipped data

No bin in PBE 16.21 has an object of `DynamicsChainRigPoseModifierData`,
`DynamicsJointTreeData` or `DynamicsChainBlendEventData`. The same search has 448 matches
for `SpringPhysicsRigPoseModifierData`.

## Status

All 4 rows `status=pending`, `pr=-`. The names are fit to submit. No shipped instance and
no shipped string has any of them, so an upstream PR must state that.
