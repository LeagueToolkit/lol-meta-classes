# The character outline pass

Reversing doc for `character-outline`. Its `batches.tsv` note points here.

Three names, one feature: the toon outline League Classic (Jade) draws on
characters, added whole in `16.10.7747445`.

```
SkinMeshDataProperties
  OutlineCategorySubmeshes  9a817954  list2[embed]  ->  CharacterOutlineSubmeshes
                                                          materialOverride  link IMaterialDef
                                                          category          link CharacterOutlineCategory
                                                          Submeshes         string
CharacterOutlineCategory    5d7c19de                      Material          link IMaterialDef
```

A skin lists which of its submeshes get an outline. `category` points at a
globally shared material so one edit restyles every character at once, which is
the whole reason the indirection exists; `materialOverride` swaps in a bespoke
outline material for the submeshes the shared one gets wrong. `Submeshes` is a
space-separated list, like the sibling `initialSubmeshToHide` fields on the same
class - 1128 distinct values, one of which (`"Flowers behind"`) names two.

## Method

Recombination over the corpus vocabulary, unprefixed, depth 3: 31,714,537,515
probes against one target state per run, **7.38 expected chance hits**. Each run
returned 7 to 13, so most of what came back is chance and is discarded on sight.

What separates these three from that noise is measured on the slice, not the
whole. Only 30,048,511 of those probes contain the word `Outline`, so the
expected number of chance hits *that mention outlines* is **0.0070 per run**.
Three runs each threw one: `3.4e-07` if it were luck.

The nesting hypothesis was wrong and is recorded so nobody re-runs it. The
sibling embed on this class is `SkinMeshDataProperties_MaterialOverride`, so the
obvious guess is `SkinMeshDataProperties_<X>`; depth 3 under that prefix, and a
192-word hand-built domain list at depth 4, returned noise only. The names are
standalone.

`hash-guesser` cannot run the prefixed half of this - `names.py` rejects a prefix
containing a separator, correctly, since a name carrying one cannot be recased.
That pass used a throwaway recombiner outside the repo.

## Attested in shipped data

`bin-grep` over all 455 WADs.

| claim | evidence |
|---|---|
| the concept is `CharacterOutline` | `materialOverride` is set 236 times across 16 WADs, over **21 distinct materials, every one named `*_CharacterOutline_inst`**, no exceptions - e.g. `Characters/Jade_Twitch/Skins/Skin45/Materials/Jade_Twitch_Skin45_Cape_CharacterOutline_inst` |
| `category` is one shared material | exactly **one** `CharacterOutlineCategory` instance exists in the game, `e1043940` in `data/maps/shipping/map453/map453.bin`, holding `Maps/Shipping/Map453/Materials/Jade_CharacterOutline_Default_inst` |
| it is an outline shader | that material's pass is `Shaders/SkinnedMesh/Jade_CharacterOutline`, params `Outline_Color`, `Outline_Width`, `Outline_CameraDepthOffset`, `Outline_FinalAlphaMult`, sampler `Outline_Mask`; alpha is driven off `IsDeadDynamicMaterialBoolDriver`, so the outline fades on death |
| the feature is Jade's | 10,446 `CharacterOutlineSubmeshes` instances in 66 WADs, confined to the League Classic champion roster plus Map11/12/21/30/35/453 |

The material paths are what carry the batch: they are strings Riot wrote, in the
field whose owning class the recombination named.

## Not attested by a string

Whole strings and token runs from the shipped Windows client and three shipped
macOS clients, 763,355 distinct probe forms at 0.0007 expected chance hits, hit
none of the three hashes. Riot's identifiers for them are not in either binary,
so the names rest on recombination plus the asset-path attestation above. That
is the `light-regions` standard, not the `exe-strings` one.

`Submeshes`, `category` and `materialOverride` were already resolved upstream;
this batch adds only the two classes and the container field.

## Status

All three rows `status=pending`, `pr=-`. Fit to submit: the method is stateable
in full and every name is checked against shipped data.
