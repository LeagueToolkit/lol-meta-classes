---
name: crack-family
description: Run a naming campaign against an unnamed class family or a set of unresolved hashes in lol-meta-classes. Scope the target from the census, derive candidates from structure, prove them by suffix folding, lattices, the anchored guesser and shipped data, then land them with hashtool. Use when asked to crack, name, or work an unnamed family, unresolved class/field hashes, or to start/resume a campaign.
---

# Cracking an unnamed class family

Distilled from the `map-entity-templates`, `logic-drivers`,
`game-entity-blocks` and `gamemode-configs` campaigns and the semantic and
string passes. This skill holds the method; a campaign's doc under `docs/`
holds only its record (section 6).

Two invariants govern everything:

- **The hash match is the filter, not the evidence.** A 32-bit hash collides
  with plausible names by chance. Nothing enters a table until something
  outside the hash attests it.
- **Every run has a price**: `probes x target_states / 2^32` expected chance
  hits. Compute it before the run, record it after, read it before reading the
  hits. Quiet passes first, broad ones last, and a smaller wordlist beats a
  bigger machine.

## 1. Scope

- Census: `python scripts/guesser_families.py rank`; the standing survey with
  per-family reads is docs/unnamed-families.md.
- Prior work: `hashes/overrides/ledger.*.tsv`, `batches.tsv`, and every
  campaign doc's ruled-out section. **Never re-run a recorded negative** - the
  unlock there is new attested vocabulary, not more probes.
- Context pack: `python scripts/guesser_families.py emit --subtree --live
  --min-unnamed N` writes one pack per family under
  `hash-guesser-out/families/`. Read the whole pack before proposing anything.
- Workability comes from the census columns, not the size. `handle`, `res%`
  and `defs` decide whether structure-led proposals exist at all. A family
  with no fields anywhere supports only a guesser sweep over an enumerable
  subject (BaseParams) or string attestation (IKeyBind), and a family paired
  1:1 by introduction build with another (ISequenceActionInstance) is
  derivative - name the other one and pair by build, spend nothing here.

## 2. Fix the pattern before any probe

From the pack, before hashing anything:

- What the named siblings fix: the template (`<Verb><Noun>Block`,
  `<Feature>ViewController`, `<X>EntityTemplate`) and its verb/noun
  vocabulary. Check whether the suffix is optional among the named members.
- If bases are typed, **the base fixes the last word** (ILogicDriver: the base
  is the return type), so `--suffix` is anchored for free and the search
  collapses to one word.
- Parallel expressions of one concept set (Entity / EntityTemplate / Prefab /
  Definition / GeComponentDef rows; action / instance pairs). One side is
  often already named, and then the other side is a function of it before any
  hash is computed.
- Field types as signature: `Pointer I<T>Get` is an input, `Embed <T>TableSet`
  an output; a field holding a component is named after it. Loud defaults name
  the class themselves (`{Magnitude, ShakesPerSecond, FalloffRate}` is a
  camera shake).
- Introduction cohorts: classes arriving in one patch belong together, and a
  class dying in the patch before a similar one appears is a rename.

## 3. The passes, in cost order

Each proved stem anchors the next pass; loop until dry.

**Suffix folding.** FNV-1a's prime is invertible mod 2^32, so `unfold(h, s)`
is the state a name was in before suffix `s` was appended. Fold the family's
suffixes out of every target and compare states. Three outcomes, each proof
rather than proposal: the state equals a known name's hash (derived); it
equals an unresolved field hash on the class itself (the field is the stem);
two targets share a state (shared stem - and a stem filling n slots at once is
`(p/2^32)^n`-competitive, turning a per-hash search into a per-concept one).

**Slot filling.** Read each member's fields, defaults, holders and closure;
propose names; require the hash to land in the slot the inheritance graph
predicts. `python scripts/guesser_families.py check proposals.txt
--matches-out only.txt`. A few hundred probes is arithmetic-zero noise, but
the landing bar does not drop: this produces exactly the kind of plausible
that collides. Cross-check docs/semantic-pass.md's proposed-and-not-taken
list; re-derive those calls, do not trust the omission either way.

**Shipped strings.** Identifier-shaped forms hashed against target states,
folded stem states, and the family's own unresolved field hashes.
CommunityDragon corpora (`D:\lol\Data\hashes\lol`) are usually a clean zero
for class names.

**Anchored guesser.**
`python scripts/split_words.py hashes/hashes.*.txt hashes/overrides/*.txt >
words.txt`; the binary is
`D:\lol\.target\x86_64-pc-windows-msvc\release\hash-guesser.exe` (rebuild:
`cargo run --release -p hash-guesser --target x86_64-pc-windows-msvc -- ...`).
Always `--only` the family (the 30-100x noise discount is what makes depth
affordable) and `--prefix`/`--suffix` from proved stems. Mode order:
`identity`, `delete`, `chain` 3, `force` 2; deeper only under anchors. The
family's field hashes are targets in their own right, and a cracked field
retro-confirms the verbs of the classes that declare it. Sweeps go through
`scripts/guesser_sweep.py`; read `MANIFEST.tsv` noise before reading hits.
New vocabulary must pass the calibration gate in
docs/plans/vocabulary-expansion.md before it spends noise.

**Lattice completion.** Cross a proved stem with the family's templates and
fill rows, not cells: every candidate must land in a predicted slot, and a
filled 2x2 or a completed row is proved by the lattice, not by any single
hash.

**The shipped graph.** `D:\lol\.target\release\bin-grep.exe --class <root>
--subclasses` over the installed WADs; add `--hashes D:\lol\Data\hashes\lol`
to resolve object paths. Read what instances are wired to: the script
variables they write, the strings and named assets beside them, the features
they gate. This pass reaches names no search can (IsInWaterBoolDriver), and
it is where renames are dated: n hashes whose lifespans meet at one patch
boundary are the strongest evidence a campaign produces, because chance
cannot arrange the spans.

**Client binary strings - last resort only.** Do not reach for this until every
pass above is exhausted, and do not treat a miss as evidence of anything. The
game resolves most meta names by hash and never materializes the string, so
whole families are simply absent: `contextual-conditions` found *zero*
occurrences of the substring `Contextual` across both arches. It pays off only
where a name is also used by non-meta code.

The repo vendors no probe set and no generator, so **ask the user to supply a
string dump rather than extracting one** - and ask for the **macOS** build,
which extracts far cleaner strings than the Windows one. Provenance rule
(docs/string-attestation.md): a probe set whose provenance the repo does not
state cannot back an upstream PR, and non-retail string sources are never named
anywhere - docs, batch notes, commits, PRs - describe the method only.

## 4. Standards for landing

Evidence tiers, best first:

1. a shipped string, or shipped instances that name the class outright
2. a rename boundary: paired spans meeting at one patch
3. a lattice or predicted slot: several hashes corroborating one pattern
4. single hash plus coherent structure: allowed into the tables, but flagged
   in the record and split off (or called out) in any upstream PR

Real refutations: the in-family base check (a hit must land on a class whose
base actually is the family root) and the interface flag on `I[A-Z]` names.
Actively refuted names go to `hashes/bad/` so they cannot be re-proposed;
proposed-and-not-taken without a recorded reason does not.

Casing is invisible to the hash, so it must be attested - a shipped string,
sibling spellings, corpus counts - never invented. Repo rules apply:
PascalCase with the one-leading-lowercase-letter exception, acronyms
title-cased (`Ui`, `Tft`) unless attestation puts them in
`names.py` `ACRONYM_EXEMPT`. Word boundaries need attestation too, ranked
separator-in-shipped-path over cased table over own corpus.

## 5. Landing

```
python scripts/hashtool.py add bintypes <Name...> -b <batch-slug> -e "<one line: method and what attests it>"
python scripts/db_build.py     # regenerate db/, commit together with hashes/
```

`hashtool lint` checks the tables; a change to `hashes/` without the
regenerated `db/` fails CI.

## 6. The campaign record

One short doc, docs/<batch>.md, holding only what the repo cannot derive - do
not restate the method (this skill), the family tree, or anything the db
shows:

- counts: the family from X live unnamed to Y, names landed
- evidence table: `hash | name | what fixes it`, one line each, with the
  weakest names flagged explicitly and the tier each group meets
- negatives: every ruled-out run with its summed expected noise, so nobody
  re-treads
- what is left: open hashes with what is known about each; stem states with
  the slots waiting on them
- status: all rows `pending`/`pr`, and which tiers are fit to submit

Then update the family's row and status note in docs/unnamed-families.md.
