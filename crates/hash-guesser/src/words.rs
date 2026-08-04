//! Splitting a name into words, and the shape a guessed name has to have.
//!
//! This is a port of `scripts/names.py`, and the two have to agree: the Python
//! side builds the wordlist, this side recombines it, and a boundary the
//! splitters disagree about is a word one of them can never produce. The unit
//! tests at the bottom are the same cases the Python is checked against.

/// A name's words, each capitalized, in order.
///
/// Capitalizing regardless of the input's own casing is what lets upstream's
/// camelCase tables contribute on equal terms with our PascalCase ones:
/// `abilityHaste` and `AbilityHaste` yield the same two words. The bin-hash
/// lowercases anyway, so the casing carried here is presentation for the
/// emitted name and nothing more.
///
/// `prefix_max` drops a leading lowercase run no longer than that - Hungarian
/// notation, the `m` of `mCoefficient`. Pass 0 to keep it.
pub fn split_words(name: &str, prefix_max: usize) -> Vec<String> {
    let b = name.as_bytes();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0usize;

    while i < b.len() {
        let c = b[i];
        if c.is_ascii_uppercase() {
            // How far the run of capitals goes.
            let mut end = i;
            while end < b.len() && b[end].is_ascii_uppercase() {
                end += 1;
            }
            if end - i >= 2 && end < b.len() && b[end].is_ascii_lowercase() {
                // An acronym butted against an ordinary word: the last capital
                // starts that word, it does not end the acronym. AIGenericCommon
                // is AI + Generic, never AIG + eneric.
                out.push(String::from_utf8_lossy(&b[i..end - 1]).into_owned());
                i = end - 1;
                continue;
            }
            if end - i >= 2 {
                // A run standing alone: ASSETS, or UI2 with its digits.
                let mut j = end;
                while j < b.len() && b[j].is_ascii_digit() {
                    j += 1;
                }
                if j > end {
                    // Lowercase cannot follow the capitals directly - the
                    // branch above claims that case - so it can only be reached
                    // through digits, and when it is it continues this word
                    // rather than starting one: IX3dShadingModel opens with
                    // IX3d, not IX3 then a stray d.
                    while j < b.len() && (b[j].is_ascii_lowercase() || b[j].is_ascii_digit()) {
                        j += 1;
                    }
                }
                out.push(String::from_utf8_lossy(&b[i..j]).into_owned());
                i = j;
                continue;
            }
            // The ordinary case: one capital and the lowercase/digit tail it
            // heads. Detection2, Map12.
            let mut j = i + 1;
            while j < b.len() && (b[j].is_ascii_lowercase() || b[j].is_ascii_digit()) {
                j += 1;
            }
            out.push(String::from_utf8_lossy(&b[i..j]).into_owned());
            i = j;
        } else if c.is_ascii_lowercase() {
            let mut j = i;
            while j < b.len() && (b[j].is_ascii_lowercase() || b[j].is_ascii_digit()) {
                j += 1;
            }
            out.push(String::from_utf8_lossy(&b[i..j]).into_owned());
            i = j;
        } else if c.is_ascii_digit() {
            let mut j = i;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            out.push(String::from_utf8_lossy(&b[i..j]).into_owned());
            i = j;
        } else {
            // A separator, an underscore above all. It belongs to no word, and
            // falling through here is what makes it split one.
            i += 1;
        }
    }

    if let Some(first) = out.first() {
        if first.len() <= prefix_max && first.bytes().all(|c| c.is_ascii_lowercase()) {
            out.remove(0);
        }
    }
    out.iter().map(|w| capitalize(w)).collect()
}

pub fn capitalize(word: &str) -> String {
    let mut c = word.chars();
    match c.next() {
        Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
        None => String::new(),
    }
}

/// The naming rule, matching `names.RE_PASCAL`: an uppercase letter, then ASCII
/// letters and digits.
///
/// Use `is_valid_name` to decide whether to *emit* something. This one is the
/// narrow shape; a name can satisfy the repo's rule without being PascalCase.
pub fn is_pascal(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_uppercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric())
}

/// A single lowercase letter, then an otherwise PascalCase name: `mCoefficient`.
/// Matches `names.RE_NOTATION`.
pub fn is_notation_prefixed(name: &str) -> bool {
    let b = name.as_bytes();
    b.len() >= 2
        && b[0].is_ascii_lowercase()
        && b[1].is_ascii_uppercase()
        && b[2..].iter().all(|c| c.is_ascii_alphanumeric())
}

/// Whether the repo would accept this name, matching `names.is_valid_name`.
///
/// The widening is not cosmetic here, and it is not reachable from the wordlist
/// either: every word `split_words` returns is capitalized, so a name assembled
/// purely out of words always opens with a capital. A `--prefix` does not come
/// from the wordlist - it is raw text from the operator - which is the one route
/// by which this search can emit `mCoefficient`, and it matters because `M` is
/// not a word the wordlist contains (0 occurrences across the current tables).
/// Without a lowercase prefix, the whole `m`-prefixed half of the field table is
/// unreachable by recombination rather than merely unlikely.
pub fn is_valid_name(name: &str) -> bool {
    is_pascal(name) || is_notation_prefixed(name)
}

/// Does this name claim to be an interface - `I` then another capital?
///
/// Checked on the finished name rather than on the search prefix, and that
/// distinction is the whole point. `I` is a *word* in the wordlist (it falls out
/// of splitting every `IFoo` name, and it lands at rank 26), so `force` and
/// `mutate` can assemble `I` + `Model` + `Joint` + `Content` under the empty
/// prefix and never touch the `I` prefix at all. A filter hung on the prefix
/// misses every one of those; a filter on the name cannot be routed around.
pub fn is_interface_named(name: &str) -> bool {
    let b = name.as_bytes();
    b.len() >= 2 && b[0] == b'I' && b[1].is_ascii_uppercase()
}

/// A wordlist entry usable as the *first* word of a name: it has to start with
/// a letter, or the name it heads could not be PascalCase.
pub fn is_usable_word(word: &str) -> bool {
    let mut chars = word.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(name: &str) -> Vec<String> {
        split_words(name, 2)
    }

    #[test]
    fn splits_the_same_cases_as_the_python() {
        assert_eq!(split("AbilityHaste"), ["Ability", "Haste"]);
        // The camelCase leading word the original LeagueHashes splitter lost.
        assert_eq!(split("abilityHaste"), ["Ability", "Haste"]);
        assert_eq!(split("AIGenericCommon"), ["AI", "Generic", "Common"]);
        assert_eq!(split("ASSETS"), ["ASSETS"]);
        assert_eq!(split("AFKDetection2"), ["AFK", "Detection2"]);
        assert_eq!(split("Obj_InfoPoint"), ["Obj", "Info", "Point"]);
        assert_eq!(split("Map12"), ["Map12"]);
        assert_eq!(split("UI2"), ["UI2"]);
        assert_eq!(split("Vector3"), ["Vector3"]);
        assert_eq!(split("selectionStrategyConfig"), ["Selection", "Strategy", "Config"]);
        // A lowercase tail reached through digits stays in the word it belongs
        // to. Splitting this as IX3 + d would recapitalize the d and hand back
        // a name spelled differently from the one we were given.
        assert_eq!(split("IX3dShadingModel"), ["IX3d", "Shading", "Model"]);
    }

    #[test]
    fn drops_the_hungarian_prefix_but_not_a_real_word() {
        assert_eq!(split("mCoefficient"), ["Coefficient"]);
        assert_eq!(split("arDamage"), ["Damage"]);
        assert_eq!(split("bIsVisible"), ["Is", "Visible"]);
        // Three letters is a word, not notation.
        assert_eq!(split("mapContainer"), ["Map", "Container"]);
        // A whole-name lowercase word survives on its own.
        assert_eq!(split("alpha"), ["Alpha"]);
        // ...unless it is short enough to be notation, which is the price of
        // the heuristic and why --keep-prefix exists.
        assert_eq!(split("id"), Vec::<String>::new());
        assert_eq!(split_words("id", 0), ["Id"]);
    }

    #[test]
    fn empty_and_degenerate_input() {
        assert_eq!(split(""), Vec::<String>::new());
        assert_eq!(split("___"), Vec::<String>::new());
        assert_eq!(split("42"), ["42"]);
    }

    #[test]
    fn pascal_rule_matches_names_py() {
        assert!(is_pascal("AbilityHaste"));
        assert!(is_pascal("AIGenericCommon"));
        assert!(is_pascal("Vector3"));
        assert!(!is_pascal("abilityHaste"));
        assert!(!is_pascal("Obj_InfoPoint"));
        assert!(!is_pascal("2Fast"));
        assert!(!is_pascal(""));
    }

    #[test]
    fn the_notation_exception_matches_names_py() {
        assert!(is_notation_prefixed("mCoefficient"));
        assert!(is_notation_prefixed("bHaveHitBone"));
        // A capital has to follow immediately, or there is no boundary being
        // preserved and the name is just lowercase.
        assert!(!is_notation_prefixed("mfoo"));
        assert!(!is_notation_prefixed("uvMode"));
        assert!(!is_notation_prefixed("m"));
        assert!(!is_notation_prefixed("MCoefficient"));
        // What the emit filter actually asks. `mCoefficient` is reachable only
        // through `--prefix m`, since "M" is not a word in the wordlist.
        assert!(is_valid_name("mCoefficient"));
        assert!(is_valid_name("IFoo"));
        assert!(!is_valid_name("uvMode"));
        assert!(!is_valid_name("Obj_InfoPoint"));
        // The prefix check in main.rs is spelled as this question.
        assert!(is_valid_name("mWord"));
        assert!(!is_valid_name("uvWord"));
    }

    #[test]
    fn interface_names_keep_their_i_as_a_word() {
        // An `I` prefix is not a shape to reject and not an acronym to swallow:
        // `IFoo` is PascalCase already, and the `I` has to come back as a word of
        // its own or every interface name in the corpus contributes a mangled
        // first token. It is one of the most productive words in the wordlist.
        assert!(is_pascal("IFoo"));
        assert_eq!(split("IFoo"), ["I", "Foo"]);
        assert_eq!(split("IMonarchItemData"), ["I", "Monarch", "Item", "Data"]);
        // Two capitals then a lowercase is the acronym branch, and it has to
        // hand the last capital to the word it heads rather than keep it.
        assert_eq!(split("IChromaData"), ["I", "Chroma", "Data"]);
        // ...but a genuine run with no lowercase after it stays whole, so the
        // `I` rule cannot be implemented by always splitting one leading letter.
        assert_eq!(split("IK"), ["IK"]);
        assert!(is_interface_named("IFoo"));
        assert!(!is_interface_named("Instance"));
    }
}
