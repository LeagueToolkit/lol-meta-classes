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
lost too. The cost compounds, which is why the rule has no exceptions.

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
# of mCoefficient, the `b` of bIsVisible, the `ar` of arDamage - the same set
# xguesser carried as its prefix list. Emitting them would put "M" and "B" in
# the wordlist, where they match everything and mean nothing. Longer runs are
# real words that camelCase merely hid, and those are the ones worth recovering.
PREFIX_MAX = 2


def is_pascal(name):
    return bool(RE_PASCAL.match(name or ""))


def why_not_pascal(name):
    """The specific reason `name` fails, phrased for someone about to retype it.

    A bare "not PascalCase" leaves the caller guessing which part offends, and
    the two failures have very different fixes: wrong case is a rename, a
    separator means the name cannot be used at all."""
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
    if name[0].islower():
        fixed = to_pascal(name)
        return f"starts lowercase; write it {fixed!r}"
    return "not PascalCase"


def check_name(name):
    """Raise unless `name` satisfies the rule. Returns the name, so it can wrap
    a value in place."""
    if not is_pascal(name):
        raise ValueError(f"{name!r}: {why_not_pascal(name)}")
    return name


def to_pascal(name):
    """`name` in PascalCase, or None if case alone cannot get there.

    Only the first letter is touched. That keeps the hash fixed - FNV runs over
    the lowercased name - so a normalized table resolves exactly what it did
    before, and the change is confined to what the name displays as.

    None means the name holds a separator or some other non-alphanumeric byte.
    That is a genuinely different name, not a differently-cased one, and the
    caller has to decide about it rather than have it silently rewritten."""
    if not name or not RE_ALNUM.match(name):
        return None
    return name[0].upper() + name[1:]


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
