# Contextual actions, 16.21

Record of the batch `contextual-actions-16-21`. Its `batches.jsonl` note points here. It names classes and fields
of the contextual action (CAC) system: the `ContextualCondition*` and `ICharacterSubcondition` classes, the
`ContextualAction*` actions and `ContextualRule` slots, and the announcer / champion music / theme music event
types. Every class was read in the 16.21 Windows client, build `8255794`.

| part | live classes | unnamed classes before | after | unnamed fields before | after |
|---|---:|---:|---:|---:|---:|
| conditions, actions, rules, audio event types | 133 | 16 | 3 | 27 | 7 |

No string attests any name; casing follows named siblings.

## Pairs and sets (tier 3)

| hash | name | what fixes it |
|---|---|---|
| 9e9e1f6d, cae2695f, 66dc7e9b | ContextualConditionUserFactCount, ContextualConditionUserFactSequence, ContextualActionUserFactAction | the action writes `{time, fact hash, unit}` into a per-unit ring; the two conditions count facts in a time window and test their order |
| eb60b2e8, 24392a01, b293a6a9, ec1b2b3c, 913543b4 | UserFact, UserFacts, CheckChronologicalOrder, FactName, SetUserFactAction | the fields of the three classes above and the `ContextualRule` slot that holds the action |
| 28556403, ea4213a0 | ContextualActionNotifyMixAction, NotifyMixAction | the action that requests an `AudioDynamicMixState` and the `ContextualRule` slot that holds it |
| 7463e786, e46a1cdb | ContextualActionKillCountSource, ContextualActionObjectiveCountSource | the embed types of `ContextualConditionKillComparison` and `ContextualConditionObjectiveCountComparison`; `SourceId` picks own player, own team, enemy team or a constant |
| b95ee9e0, 55848081, 6caa1bcc | CompareToSource, KillBaseSource, KillCompareToSource | `CompareToSource` is the second source next to the named `BaseSource`; the kill comparison carries the same pair with a `Kill` prefix |
| a29e7869, 58a4ecbd | ChampionMusicEventTypeTeamwide, ThemeMusicEventTypeTeamwide | twin classes with only `TeamSucceededEvent` / `TeamFailedEvent`; one word fits both hashes; both register after the `Generic` sibling |
| 2f68e6e3, 11e8956e | BlueTeamSpectatorEvent, RedTeamSpectatorEvent | the two strings of an announcer event type that a spectator hears, for team 100 and team 200 |
| 395e4a90, fa24eccf | ExcludeChangedToZero, ExcludeDecrements | filters of `ContextualConditionBuffCounterChanged` that skip a change to 0 and a decrease |
| f2f334, 68ddd682 | ContextualConditionIsInCombat, InCombat | subcondition that runs its embedded `IsInCombatDynamicMaterialBoolDriver`; between `ContextualConditionIsAlly` and `ContextualConditionIsSelf` |
| 40bae5c7, b62df33d | ContextualConditionIsStealthed, IsStealthed | subcondition that tests the unit's stealth state against the bool; between `ContextualConditionIsSelf` and `ContextualConditionItemCanBePurchased` |
| 72387b18, d4e06278 | ContextualConditionCharacterIsLocalPlayer, IsLocal | subcondition that tests whether the unit is the local player; between `ContextualConditionCharacterIsCastingRecall` and `ContextualConditionCharacterLevel` |
| bd465a5, 65fad9fb | ContextualConditionAbilityResourceIsPips, AbilityResourceIsPips | compares the bool with the kind of resource bar that fired the "resource full" situation; before `ContextualConditionAbilityResourceSlot` |
| 67a18e0c, be662223 | ContextualConditionCharacterTimeToLive, mTargetTimeToLive | compares the owner's estimated seconds to death with the float; between `ContextualConditionCharacterSkinID` and `ContextualConditionCharacterUnitTags` |

## Singles (tier 4: reader and registration order agree)

| hash | name | what fixes it |
|---|---|---|
| 2b11fd98 | ContextualConditionKillerType | compares `KillerType` with the kind of unit that killed the owner; between `ContextualConditionKillCount` and `ContextualConditionLastBoughtItem` |
| 4bd0e2b | FilterBuffByCaster | `ContextualConditionCharacterBuffStackCount`: when set, counts only the stacks applied by the unit that `BuffCaster` selects |
| cab8f282 | WaitForShopkeeperVo | `ContextualActionPlayAudio`: queues the line behind the shopkeeper voice |

## What is left

- Classes: `0x36255113` (announcer event type with individual, team and spectator events), `0x15e1fe7e` (champion
  music event type with one event), `0x9583cf01` (champion music event type with individual, team and assisted
  events).
- Fields: `ContextualActionPlayVO.0x9449aa71` (turns off spell-attached playback), `0x2f1edf29` on the two `Multi*`
  event types (a count past the end returns the last entry), `ThemeMusicEventTypeGeneric.0x2a2e7fd9`,
  `0x15e1fe7e.0x4c546bb7`, `0x9583cf01.0x8c107881` / `0xe2030a49` (not read).

## Status

All rows `pending`. Tier 3 rows are fit to submit upstream; tier 4 rows go in a separate, flagged part of any
upstream PR.
