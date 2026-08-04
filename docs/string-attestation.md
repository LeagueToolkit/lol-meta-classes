# String attestation

The reversing doc for `exe-strings` and `data-section-strings`. Their
`batches.tsv` notes point here.

The guesser proposes a name and the hash fails to contradict it. This runs the
other way: the probe set is nothing but strings that shipped, so a hit is a name
Riot wrote against a hash the game registered.

## The two probe sets

**`exe-strings`** (181 names: 54 classes, 127 fields). Every string the shipped
Windows client has carried, per build, across ~35 builds from `10.9.3186057` to
`16.15.7996036`. ~25k survive the filter for identifier-shaped tokens.

**`data-section-strings`** (39 names: 18 classes, 21 fields). NUL-terminated
strings out of a binary's data sections - 87,606 of them, 59,050 distinct probe
forms. Only tier A landed: the whole string is the name.

## Noise budget

`probes x target states / 2^32`, over the sweep's unresolved target sets (2,455
classes, 4,783 fields):

| pass | probes | expected chance hits | landed |
|---|---|---|---|
| `exe-strings` | ~25k | 0.04 | 181 |
| `data-section-strings` tier A | 38,657 | 0.06 | 39 |

The sweep's *quiet* tier runs at 8-11. Calibration on the second set: its probe
forms contain 2,237 of 2,919 known class names and 4,752 of 8,879 known field
names, so those strings are where meta names live at better than half recall.

## Corroboration

**124 of the 220 were already proposed by an earlier pass**, same spelling,
before any string was hashed. **8 refute a sweep candidate outright:**

| hash | shipped string | displaced candidate |
|---|---|---|
| `684b0875` | `AudioManagerWwise` | `LevelPipBonusHoverShop` |
| `1cfeba20` | `CapWalletsReward` | `CharacterLevelIdle` |
| `9e73e21c` | `EventBusApClientGameEnd` | `DeathMinimapOptionDef` |
| `1125294b` | `LolSpellScriptBuffOnUpdateStats` | `MonarchMissionStepPercentage` |
| `d443ea96` | `RewardGroupCelebrationAssets` | `FixedDesaturateCheat` |
| `707c2c57` | `HasLevelPurchasing` | `CharSwapsData` |
| `6f478bcd` | `SendFullGameState` | `SyncedAnimationRigPoseTagDataInstance` |
| `8ecb12a5` | `TacticalDifficulty` | `ValueOnlyOverrides` |

**129 of the 181 `exe-strings` names shipped before their hash appears in any
dump we hold**, 95 of them back to `10.9.3186057` - so they cannot have been
reverse-engineered out of the hash.

Families arrive whole: 40-odd `Flat*Mod`/`Percent*Mod` stat modifiers, 17
`LolSpellScript*` hooks, 18 `EventBusAp*` telemetry objects.

63 further hits landed on hashes already named, 54 of them identically - which
independently confirms part of `semantic-pass-family`, the batch its own doc
flags as having no second method behind it.

## Casing

Names entered under the repo rule, not as the string spells them: 58 shipped
camelCase (`sendFullGameState` -> `SendFullGameState`) and 13 shipped acronyms
in capitals (`UIElement*` -> `UiElement*`, `TFT*` -> `Tft*`, `LoL*` -> `Lol*`,
`WASD*` -> `Wasd*`, `PUUID` -> `Puuid`). `scripts/names.py` enforces both.

## Held back

**Tier B (12)** - the name is a scope or path component of a longer string
(`AudioFacade` from `AudioFacade::IsValid()`). **Tier C (22)** - a token split
out of prose, where ordinary English enters the probe set: `Provider` from
`Microsoft Enhanced RSA and AES Cryptographic Provider`, `Cutoff` from `OCSP
Archive Cutoff`. Both tiers are in `hash-guesser-out/ALL.txt`, neither in
`hashes/bad/` - unproven, not refuted.

## Reproducing

Neither probe set is vendored and neither generator runs from a clean checkout,
so both live in the campaign scratch directory. The per-build string history is
a byproduct of a separate archive of shipped clients. The second set's input is
not recorded here; ask before rebuilding that batch or citing it outside this
repo.

## Status

All 220 rows `pending`, `pr=-`.

`exe-strings` is fit to submit: every name is a string the retail client
carries, which is the standard `light-regions` met.

`data-section-strings` is **not cleared for upstream**. The names are at least
as solid, but a CDragon PR has to state what attests them and this repo does not
state that probe set's provenance.
