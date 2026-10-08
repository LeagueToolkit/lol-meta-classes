# UI audio events

Reversing doc for `ui-audio-events`. Its `batches.jsonl` note points here.

Six classes born in `16.19.8207193`, plus 11 pointer fields added to three UI sound
tables in the same build. An event object replaces a plain event-name string on a
button, a slider or a combo box. The batch names both interfaces and 17 fields. The
four concrete classes are unnamed.

```
IUiAudioEvent             ed2da5b0  interface
  IUiSfxEvent             012759fb  interface
    df305970                          EventName      string
      57b38653                        AudioBehavior  link -> a35db897
  5d9795fb                            EventName      string
                                      VoBehavior     link -> a35db897
a35db897                              LineCooldown         f32
                                      BehaviorCooldown     f32
                                      MaxPlays             u32
                                      PercentChanceToPlay  u8 (100)
```

## What the client does

Read from the 16.21 PBE Windows client. Not tested in game.

- `df305970` posts `EventName` with no limits.
- `57b38653` and `5d9795fb` apply the linked `a35db897` object before they post. A null
  link is no limit.
- `BehaviorCooldown`: the event is skipped if fewer seconds than this have passed since
  any event played through the same `a35db897` object.
- `LineCooldown`: the event is skipped if fewer seconds than this have passed since the
  same event name played.
- `MaxPlays`: the event is skipped once the same event name has played this many times.
  `0` is no limit.
- `PercentChanceToPlay`: the event plays if this value is greater than a random integer
  below 100.
- The play count and the last play time are stored per event-name hash.
- `5d9795fb` replaces `{champion}` in `EventName` before the limits apply. An
  `EventName` without that token does not play.

## Evidence

| hash | name | what fixes it | tier |
|---|---|---|---|
| ed2da5b0 | IUiAudioEvent | interface flag set; registers between `IClientBlock` and `IUiBlock`; root of the family | 3 |
| 012759fb | IUiSfxEvent | interface flag set; registers between `IUiElementIGet` and `IUiTextureDataProvider`; shares the `IUi<X>Event` template with the root | 3 |
| 5d147744, 8b848362 | RollOver, RollOut | index pair, see below | 3 |
| 01fcf9b4, 1036ae7e | Press, Release | index pair | 3 |
| 00fdb425, 66b8c713 | PressInactive, ReleaseInactive | index pair | 3 |
| a52ae29d, 5977e147 | PressSelected, ReleaseSelected | index pair | 3 |
| fa73afb1, 5a2649b0, 976d3aeb | DragStart, DragEnd, BarClicked | index pair | 3 |
| 9087c853, 1d445284 | AudioBehavior, VoBehavior | two links to the same class share the word `Behavior`; one sits on the SFX class, one on the class that fills `{champion}` | 3 |
| d14bc802 | BehaviorCooldown | the cooldown that is stored on the behaviour object | 4 |
| ff102002 | LineCooldown | the cooldown that is stored per event name | 4 |
| a4c80814 | MaxPlays | compared against the play count of the event name | 4 |
| 7aafcde5 | PercentChanceToPlay | u8 with default 100, compared against a random integer below 100. **Weakest**: found in a run with 7.84 expected chance hits | 4 |

**The index pair.** Each sound table stores its old string fields as one array and its
new pointer fields as a second array. The client reads both with the same index and
calls the pointer only when the string is empty. Every pair lines up:

| class | index | string | pointer |
|---|---|---|---|
| UiElementGroupButtonSoundEvents | 0 | RollOverEvent | RollOver |
| | 1 | RollOutEvent | RollOut |
| | 2 | MouseDownEvent | Press |
| | 3 | MouseDownOnInactive | PressInactive |
| | 4 | MouseUpEvent | Release |
| | 5 | MouseUpOnInactive | ReleaseInactive |
| | 6 | MouseDownSelected | PressSelected |
| | 7 | MouseUpSelected | ReleaseSelected |
| UiElementGroupSliderSoundEvents | 0 | OnDragStartEvent | DragStart |
| | 1 | OnDragEndEvent | DragEnd |
| | 2 | OnBarClickedEvent | BarClicked |
| UiComboBoxSoundEvents | 0 | OnSelectionEvent | Select |

`Select` was already named. The eight button names also fill a grid of
`{Press, Release} x {"", Inactive, Selected}` plus `RollOver`/`RollOut`.

## Runs

Hand-built word lists, recombined outside the repo. Targets are the family's unresolved
class and field hashes.

| run | probes | targets | expected chance hits | landed |
|---|---|---|---|---|
| sibling stems with prefixes and suffixes | 740,544 | 19 | 0.0033 | 11 pointer fields, `IUiAudioEvent` |
| 153 words, depth 1 to 3 | 3,605,139 | 11 | 0.0092 | `AudioBehavior`, `VoBehavior`, `BehaviorCooldown` |
| 136 words, depth 1 to 3, 19 affix patterns | 48,147,672 | 8 | 0.0897 | `IUiSfxEvent` |
| 200 words, depth 0 to 2, 24 prefixes x 26 suffixes | 25,085,424 | 7 | 0.0409 | `LineCooldown`, `MaxPlays` |
| 92 words, depth 0 to 3 prefix x depth 0 to 2 suffix | 6,736,455,465 | 5 | 7.84 | `PercentChanceToPlay`; 8 other hits discarded |

## Ruled out

- 154 words, depth 0 to 3, crossed with 41 x 41 two-word suffixes: 6,179,582,935 probes
  against the four concrete classes, 5.76 expected chance hits, 3 returned, none
  readable. Discarded.
- The run with 7.84 expected chance hits returned no readable name for any of the four
  classes.
- `UiSfxEvent`, `UiVoEvent`, `UiAudioEvent`, `UiAudioBehavior`, `UiAudioEventBehavior`,
  `UiSfxEventWithBehavior` and `UiChampionVoEvent` do not match.
- 130 words, depth 0 to 3 prefix x depth 0 to 1 suffix: 290,038,061 probes against
  `a35db897` and its predecessor `dc3815a8`, 0.135 expected chance hits, 0 returned.
- No name of this family is a string in the 16.21 PBE Windows client.

## What is left

| hash | what is known |
|---|---|
| df305970 | concrete `IUiSfxEvent`, holds `EventName` |
| 57b38653 | child of `df305970`, adds `AudioBehavior` |
| 5d9795fb | child of `IUiAudioEvent`, `EventName` with `{champion}` and `VoBehavior` |
| a35db897 | the behaviour object, target of `AudioBehavior` and `VoBehavior` |
| dc3815a8 | lived `16.18.8159717` to `16.18.8175716` with `LineCooldown`, `BehaviorCooldown` and `MaxPlays`; no class referenced it. `a35db897` appears in the next build with the same three fields plus `PercentChanceToPlay` |

The four live classes register in that order (`a35db897`, `5d9795fb`, `df305970`, `57b38653`)
directly before `AmbienceEvent`.

No UI table has a pointer typed for `5d9795fb`. Every new pointer is an `IUiSfxEvent`.

## Not shipped

No bin in retail 16.20 or PBE 16.21 has an object of these classes or sets one of the
17 fields.

## Status

All 19 rows `status=pending`, `pr=-`. The tier 3 names are fit to submit. The four tier
4 names rest on one hash each plus the client's use of the field.
