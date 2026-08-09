#!/bin/env python
"""The naming rule for cracked names, and the word splitter built on it.

Every name this repo adds to an override table or a ledger row is PascalCase:
an uppercase first letter, then ASCII letters and digits only. No camelCase, no
underscores, no separators. `check_name` is the enforcement point and is not
advisory - `hashtool add` refuses a name that fails it.

The reason is the wordlist, not tidiness. Cracking a hash means recombining
words taken from names we already know, so a name is only worth as much as the
words that can be recovered from it. PascalCase marks every boundary with a
capital; camelCase hides the first one. `abilityHaste` yields "Haste" and loses
"Ability", and a word lost from the wordlist is every future name built from it
lost too. The cost compounds, which is why the rule is narrow.

It has exactly one exception, and the exception exists because the reasoning
above inverts for it: a *single* lowercase letter may lead a name, as in
`mCoefficient`. One letter is never a word worth having - it matches everything
and means nothing, which is why `split_words` drops it (see PREFIX_MAX) - so the
camelCase spelling costs the wordlist nothing at all. Capitalizing it does not
recover a word, it invents one: `MCoefficient` splits as "M" + "Coefficient" and
puts "M" in the vocabulary permanently, where it will pad every force run that
reaches it. The rule would be doing the exact damage it was written to prevent.
Attestation says the same thing from the other side - `m`-prefixed members are
how the engine spells its own fields, 2001 of the 9439 names upstream serves for
binfields, so recasing would replace an attested spelling with an invented one.

Two or more leading lowercase letters stay banned, because there the original
reasoning does hold: the `Uv` of `uvMode`, the `Hp` of `hpPerLevel`, the `Is` of
`isClickable` are real words, and the capital is what recovers them.

The same argument bans an acronym in capitals - `Ui` not `UI`, `Tft` not `TFT`,
`Lol` not `LoL`, `Wasd` not `WASD`. A capital run is one word to the splitter,
so `UIElement` spends a token nothing else can reuse where `UiElement` yields
two that recur everywhere. See `acronym_runs`, and `ACRONYM_EXEMPT` for the
handful of names where attestation puts the capitals there.

Casing is free to legislate because the bin-hash is FNV-1a over the *lowercased*
name: `abilityHaste` and `AbilityHaste` are the same hash and resolve the same
thing. Separators are not - the underscore in `Obj_InfoPoint` is hashed like any
other byte, so a name carrying one cannot be normalized into the rule and has to
be rejected instead. `to_pascal` only ever changes letter case, and returns None
rather than produce a name that hashes to something else.

Upstream is under no such rule and its binfields table is largely camelCase; the
mirror is vendored as served and is not rewritten. The rule binds what *we*
author. `split_words` therefore has to cope with upstream's casing anyway, and
recovers the leading word that the original LeagueHashes splitter dropped.
"""

import re

# The rule. Anchored, ASCII-only: an uppercase letter, then letters and digits.
RE_PASCAL = re.compile(r"^[A-Z][A-Za-z0-9]*$")

# The one exception: a single lowercase letter, then an otherwise PascalCase
# name. `mCoefficient`, `bHaveHitBone`. Requiring the capital immediately after
# is what keeps this to Hungarian notation - `mfoo` has no boundary to preserve
# and is a plain lowercase name, so it falls through to the rule and gets recased.
RE_NOTATION = re.compile(r"^[a-z][A-Z][A-Za-z0-9]*$")

# Case-only normalization is possible exactly when the name is already
# alphanumeric - anything else would need a byte removed, which moves the hash.
RE_ALNUM = re.compile(r"^[A-Za-z][A-Za-z0-9]*$")

# One identifier -> its words, in order. The alternatives are tried left to
# right, so the acronym case wins before the ordinary one:
#
#   [A-Z]+(?![a-z])         a run of capitals not followed by a lowercase letter,
#     (?:[0-9]+[a-z0-9]*)?  so AIGenericCommon gives AI (not AIG - the G belongs
#                           to Generic) and ASSETS/UI2 stay whole. The optional
#                           tail starts at a digit because the lookahead already
#                           ruled out a lowercase letter there; once past the
#                           digits, lowercase continues the same word, which is
#                           what makes IX3dShadingModel open with IX3d instead
#                           of IX3 and a stray d
#   [A-Z][a-z0-9]*          the ordinary word: Detection2, Map12
#   [a-z][a-z0-9]*          a lowercase run: upstream camelCase, or a tail after
#                           an underscore
#   [0-9]+                  digits nothing claimed
#
# Anything unmatched - underscores above all - separates words by falling
# through, which is what makes Obj_InfoPoint split as Obj + InfoPoint.
RE_WORD = re.compile(
    r"[A-Z]+(?![a-z])(?:[0-9]+[a-z0-9]*)?|[A-Z][a-z0-9]*|[a-z][a-z0-9]*|[0-9]+")

# A leading lowercase run this short is Hungarian notation, not a word: the `m`
# of mCoefficient, the `b` of bHaveHitBone, the `ar` of arBase - the same set
# xguesser carried as its prefix list. Emitting them would put "M" and "B" in
# the wordlist, where they match everything and mean nothing. Longer runs are
# real words that camelCase merely hid, and those are the ones worth recovering.
#
# Two, not one, even though the rule only permits a one-letter prefix in a name
# we author: this is the splitter, and it has to read upstream's table as served.
# The 2-char runs there (`uv`, `hp`, `ar`, `is`, `on`) are the ones the rule
# would have us recase, and dropping them here costs nothing measurable because
# every one of them already reaches the wordlist capitalized from some other
# name - Uv 15, Hp 4, Is 129, On 171 across the current tables.
PREFIX_MAX = 2


def is_pascal(name):
    return bool(RE_PASCAL.match(name or ""))


def is_notation_prefixed(name):
    """A single lowercase letter in front of a PascalCase name: `mCoefficient`.

    Note that this is deliberately *not* a list of blessed letters. `m` and `b`
    are the ones the corpus happens to use today and `r` is the one the engine
    is known to use elsewhere, but the argument for the exception is about the
    length of the prefix, not its spelling: no single letter is a word worth
    putting in a wordlist, so no single letter is worth recasing a name to
    recover. Enumerating letters would only mean revisiting this the next time
    the engine picks a new one."""
    return bool(RE_NOTATION.match(name or ""))


def is_valid_name(name):
    """Whether `name` may be added to an override table or a ledger row.

    PascalCase, or the one-letter-prefix exception. Interface names need no
    special case and never did: `IFoo` is already PascalCase, and `split_words`
    hands back "I" + "Foo" rather than "IF" + "oo", so the `I` stays a word in
    its own right - it is one of the most productive words in the wordlist, at
    197 occurrences."""
    return (is_pascal(name) or is_notation_prefixed(name)) and not acronym_runs(name)


# No word is spelled in capitals past its first letter: `Ui`, not `UI`; `Tft`,
# not `TFT`; `Lol`, not `LoL`; `Wasd`, not `WASD`. Same argument as the rest of
# the rule, one level down. The splitter has to treat a capital run as a single
# word, so `UIElement` yields "UI" where `UiElement` yields "Ui" + "Element" -
# and "UI" is a token no other name can reuse, while "Ui" and "Element" both
# recur everywhere. An acronym spelled in caps is a word removed from the
# vocabulary and a junk token added in its place.
#
# A run is an acronym only after two adjustments. A leading `I` on an interface
# is not part of one (`IX3dShadingModel` opens with I + X3d), and the last
# capital of a run belongs to the next word when a lowercase letter follows it
# (`UIElement` is UI + Element, so the run to judge is "UI"). What is left has
# to be a single capital, or it is an acronym.
RE_CAPS = re.compile(r"[A-Z]{2,}")

# An acronym the house spells with interior lowercase is still an acronym, and
# `LoL` is the one that occurs. It is not a capital run, so RE_CAPS cannot see
# it: `LoLSpellScript` splits as "Lo" + "L" + "Spell", spending two junk tokens
# to say what "Lol" says in one.
BRANDS = {"LoL": "Lol"}

# Names carrying a capital run that attestation, not convention, put there.
# `SSAO` is the shader's own OMIT_SSAO define and the `ssao-casing` batch turns
# on that spelling; `GroupID` follows the upstream `productID` on its own class;
# `CC` is crowd control. Each is documented where it was cracked. They are
# exempt because the evidence outranks the rule, not because the rule is soft.
ACRONYM_EXEMPT = frozenset({
    "MapSSAO", "MapSSAORenderer", "MapSSAOSettings", "SSAO", "GroupID",
    "ParamsCC",
})


def acronym_runs(name):
    """The capital runs in `name` that are acronyms, in order. Empty if none."""
    if name in ACRONYM_EXEMPT:
        return []
    body = name[1:] if re.match(r"^I[A-Z]", name) else name
    out = [b for b in BRANDS if b in body]
    for m in RE_CAPS.finditer(body):
        run, end = m.group(0), m.end()
        if end < len(body) and body[end].islower():
            run = run[:-1]      # that last capital opens the following word
        if len(run) > 1:
            out.append(run)
    return out


def fold_acronyms(name):
    """`name` with every acronym run title-cased: `UIElement` -> `UiElement`.

    Case-only, so the hash is untouched, and idempotent. Best effort, not an
    oracle: two acronyms with no lowercase between them read as one run, so
    `EventBusAPRFC0190Scope` folds to `EventBusAprfc0190Scope` where the name
    wanted `EventBusApRfc0190Scope`. `why_invalid` prints what this would write,
    so a wrong fold is visible before it is accepted."""
    if not acronym_runs(name):
        return name
    for brand, folded in BRANDS.items():
        name = name.replace(brand, folded)
    lead, body = ("", name)
    if re.match(r"^I[A-Z]", name):
        lead, body = name[0], name[1:]

    def fold(m):
        run, end = m.group(0), m.end()
        tail = ""
        if end < len(body) and body[end].islower():
            run, tail = run[:-1], run[-1]
        return (run[0] + run[1:].lower() if len(run) > 1 else run) + tail

    return lead + RE_CAPS.sub(fold, body)


def why_invalid(name):
    """The specific reason `name` fails, phrased for someone about to retype it.

    A bare "not PascalCase" leaves the caller guessing which part offends, and
    the failures have very different fixes: wrong case is a rename, a separator
    means the name cannot be used at all."""
    if not name:
        return "empty name"
    bad = sorted({c for c in name if not (c.isascii() and c.isalnum())})
    if bad:
        chars = " ".join(repr(c) for c in bad)
        return (f"contains {chars}; PascalCase is letters and digits only, and "
                f"a separator is part of the hashed bytes so it cannot be "
                f"dropped - if the name really carries one, it is not ours to add")
    if name[0].isdigit():
        return "starts with a digit; PascalCase starts with an uppercase letter"
    runs = acronym_runs(name)
    if runs and (is_pascal(name) or is_notation_prefixed(name)):
        caps = ", ".join(repr(r) for r in runs)
        return (f"spells {caps} in capitals; no word is capitalized past its "
                f"first letter, because the splitter reads a capital run as a "
                f"single junk word - write it {fold_acronyms(name)!r}")
    if name[0].islower():
        fixed = to_pascal(name)
        run = re.match(r"^[a-z]+", name).group(0)
        if len(run) > 1 and len(run) < len(name):
            # Looks like a Hungarian prefix of the wrong length. Say so, because
            # the operator has probably just seen a one-letter prefix accepted
            # and needs to know what separates the two cases.
            return (f"starts with the {len(run)}-letter lowercase run {run!r}; "
                    f"only a single letter may lead a name, because a longer run "
                    f"is a word the capital recovers - write it {fixed!r}")
        return f"starts lowercase; write it {fixed!r}"
    return "not PascalCase"


def check_name(name):
    """Raise unless `name` satisfies the rule. Returns the name, so it can wrap
    a value in place."""
    if not is_valid_name(name):
        raise ValueError(f"{name!r}: {why_invalid(name)}")
    return name


def to_pascal(name):
    """`name` in PascalCase, or None if case alone cannot get there.

    Only letter case is touched, including any capital run. That keeps the hash
    fixed - FNV runs over the lowercased name - so a normalized table resolves
    exactly what it did before, and the change is confined to what the name
    displays as.

    None means the name holds a separator or some other non-alphanumeric byte.
    That is a genuinely different name, not a differently-cased one, and the
    caller has to decide about it rather than have it silently rewritten.

    Only ever call this on a name `is_valid_name` has already rejected. On a
    notation-prefixed name it is the wrong repair by construction - it produces
    the `MCoefficient` the exception exists to avoid."""
    if not name or not RE_ALNUM.match(name):
        return None
    return fold_acronyms(name[0].upper() + name[1:])


def split_words(name, prefix_max=PREFIX_MAX):
    """A name -> its words, each capitalized, in order.

    This is the wordlist, and the sentence the guesser mutates. Words come back
    capitalized whatever the input casing was, so upstream's `abilityHaste` and
    our `AbilityHaste` contribute the same two words and the wordlist does not
    carry both `ability` and `Ability` as if they were different.

    The leading Hungarian prefix is dropped (see PREFIX_MAX); pass
    prefix_max=0 to keep it."""
    words = RE_WORD.findall(name or "")
    if words and words[0].islower() and len(words[0]) <= prefix_max:
        words = words[1:]
    return [w[0].upper() + w[1:] for w in words]


def same_name(a, b):
    """Whether two spellings are the same name.

    Case-insensitive, because the hash is: a name that resolves a hash resolves
    it in any casing, so `abilityHaste` and `AbilityHaste` are one name written
    two ways and not two names.

    Use this for "does upstream know this name?", which is a question about the
    name. Do *not* use it for "is this override redundant?", which is a question
    about the file: an override that differs from upstream only in casing is
    doing exactly the job it was added for, and comparing it loosely would
    delete every deliberate restyle in the table."""
    return a is not None and b is not None and a.lower() == b.lower()


def stem(word):
    """`word` without its trailing digits, or None if that leaves nothing or
    changes nothing. `Detection2` hides `Detection`; `Map12` hides `Map`. Worth
    having in a wordlist, but as an addition to the real token, never a
    replacement - `Vector3` and `Float2` are names in their own right."""
    cut = word.rstrip("0123456789")
    return cut if cut and cut != word else None
