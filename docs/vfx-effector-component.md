# VFX effector and audio components

Reversing doc for `vfx-effector-component`. Its `batches.jsonl` note points here.

Two component families that are new in PBE `16.21`, plus the runtime classes of every
entry of `VfxComponents.Components`. The batch names 18 classes and 2 fields.

```
VfxComponentBase
  VfxEffectorComponent                 524de342   EffectorData -> VfxEffectorComponentData
  VfxAudioComponentBase                fdbf829b   interface
    VfxAudioComponent                  7a98ac76   AudioCues: list of VfxAudioCue

VfxEffectorComponentData               858d9212   interface
                                                  UpdateMode u8, DecayDuration f32, Magnitude
  VfxMovementEffectorComponentData     068008e9   radius
  VfxBurstEffectorComponentData        f9ed3b7e   radius, SpinFactor
  VfxBeamEffectorComponentData         145598b7   Width
  VfxTrapezoidEffectorComponentData    eed60e9c   OriginWidth, TargetWidth

VfxAudioCue                            86af78a6   sound, type, LifetimeBehavior, Attachment

IVfxRuntime                            2c23a84c   interface
  VfxEffectorComponentRuntime          b9b8fb5a
    VfxMovementEffectorComponentRuntime   b36c307f
    VfxBurstEffectorComponentRuntime      2c6f09a6
    VfxBeamEffectorComponentRuntime       507d7fe1
    VfxTrapezoidEffectorComponentRuntime  a7cee9c8
  VfxLightComponentRuntime             f6e84048
  IVfxAudioRuntime                     c239373a
    VfxAudioRuntime                    1ad8951d
```

The effector family replaces seven classes that left in the same patch:
`VfxEffectorDefinition`, `EnvironmentEffectorBase` and
`EnvironmentEffector{Movement,Burst,Beam,Trapezoid}`, and the field
`VfxSystemDefinitionData.effectorDefinition`. The four shape classes keep the property
hashes of their predecessors. `Magnitude` and the shape values changed from `F32` to
`VfxFloatDynamicProperty`.

## What the client does

Read from the 16.21 PBE Windows client, build `8255794`. Not tested in game.

- A component in `VfxComponents.Components` creates one runtime object when an emitter
  instance is constructed. `VfxEffectorComponent` passes the call to `EffectorData`,
  which allocates the runtime class of its shape. A null `EffectorData` is not tested.
- The emitter calls each runtime object on three events: particles spawned, a particle
  killed, and each emitter step.
- Each step with at least one live particle, the effector runtime evaluates the drivers
  of its data object on particle 0. `radius`, `Width`, `OriginWidth` and `TargetWidth`
  are clamped to a minimum of 0. `Magnitude` and `SpinFactor` are not clamped.
- The runtime then stores, on the X and Z axes: the position of the VFX system, its
  target position, the direction and the speed of its movement, and the distance that it
  has travelled.
- No code in that build reads the effector runtime, `UpdateMode` or `DecayDuration`. The
  values of `UpdateMode` are not known.
- Build `8264391` adds `UseFirstParticlePosition` to `VfxEffectorComponentData`. That
  build was not read.

The audio family was not read in the client.

## Evidence

| hash | name | what fixes it | tier |
|---|---|---|---|
| 524de342 | VfxEffectorComponent | child of `VfxComponentBase`; its only field is `EffectorData`; the stem that the 10 names below share | 3 |
| 858d9212 | VfxEffectorComponentData | target of `EffectorData`; component name + `Data` | 3 |
| 068008e9, f9ed3b7e, 145598b7, eed60e9c | Vfx{Movement,Burst,Beam,Trapezoid}EffectorComponentData | one pattern, four hits; each class has the property set of the removed `EnvironmentEffector<Shape>` class | 3 |
| b9b8fb5a | VfxEffectorComponentRuntime | `Data` replaced by `Runtime`; base of the four below | 3 |
| b36c307f, 2c6f09a6, 507d7fe1, a7cee9c8 | Vfx{Movement,Burst,Beam,Trapezoid}EffectorComponentRuntime | same replacement, four hits; each is the class that the data class of that shape allocates | 3 |
| f6e84048 | VfxLightComponentRuntime | component name + `Runtime` | 3 |
| 2c23a84c | IVfxRuntime | interface flag set; root of every runtime class | 3 |
| fdbf829b, 7a98ac76 | VfxAudioComponentBase, VfxAudioComponent | interface and its child under `VfxComponentBase` | 3 |
| c239373a, 1ad8951d | IVfxAudioRuntime, VfxAudioRuntime | interface flag set on the first; same shape as the component pair | 3 |
| 86af78a6, 584327ed | VfxAudioCue, AudioCues | the list field and its element class share `AudioCue`; the shipped element holds a Wwise event name in `sound` | 3 |
| fa48afb7 | UpdateMode | `U8` on the effector data base. **Weakest**: one hash, no reader in the client | 4 |

**Registration order.** The client registers classes in near-alphabetical order. The 19
registration functions of the classes above (the 18 new names and `VfxLightComponent`)
are in this address order:

```
IVfxAudioRuntime, IVfxRuntime,
VfxAudioComponentBase, VfxAudioComponent, VfxAudioCue, VfxAudioRuntime,
VfxBeamEffectorComponentData, VfxBeamEffectorComponentRuntime,
VfxBurstEffectorComponentData, VfxBurstEffectorComponentRuntime,
VfxEffectorComponentData, VfxEffectorComponent, VfxEffectorComponentRuntime,
VfxLightComponent, VfxLightComponentRuntime,
VfxMovementEffectorComponentData, VfxMovementEffectorComponentRuntime,
VfxTrapezoidEffectorComponentData, VfxTrapezoidEffectorComponentRuntime
```

That is the alphabetical order with two adjacent swaps: `VfxAudioComponentBase` before
`VfxAudioComponent`, and `VfxEffectorComponentData` before `VfxEffectorComponent`.

## Runs

Hand-built lists, recombined outside the repo.

| run | probes | targets | expected chance hits | landed |
|---|---|---|---|---|
| 10 prefixes x 11 stems x 23 suffixes | 2,530 | 9 | 0.0000053 | `VfxEffectorComponent`, `VfxEffectorComponentData` |
| 27 patterns x 53 shape words | 1,431 | 8 | 0.0000027 | the four shape data classes |
| 2 prefixes x 75 stems x 44 suffixes | 6,600 | 14 | 0.000022 | the five effector runtime classes, `VfxLightComponentRuntime`, `IVfxRuntime`, the four `VfxAudio` classes |
| 8 prefixes x 14 stems x 18 suffixes | 2,016 | 1 | 0.0000005 | `VfxAudioCue` |
| 23 field words | 23 | 9 | 0.00000005 | `UpdateMode` |
| 27 field words | 27 | 14 | 0.00000009 | `AudioCues` |

## Ruled out

- No name of either family is a string in the 16.21 PBE Windows client.
- `0e0d9594` on `VfxAudioCue` is `sound`, which upstream already has. It is not part of
  the batch.

## What is left

| hash | what is known |
|---|---|
| 24c6d754 | `F32` on `EnvironmentEffectorBase`, default 1.0, last seen in `8230722` |
| 608eac63 | `Bool` on `VfxEffectorDefinition`, removed with the class |

## Shipped data

- No bin in retail 16.20 or PBE 16.21 has an object of the effector family.
- PBE 16.21 has one object with `VfxAudioComponent`: `0x8faf1e44` in `Global.wad.client`,
  three emitters, one `VfxAudioCue` each.

## Status

All 20 rows `status=pending`, `pr=-`. The tier 3 names are fit to submit. `UpdateMode`
rests on one hash.
