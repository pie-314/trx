/// Score a fuzzy query against a target string.
///
/// Scoring tiers (highest → lowest priority):
///   1. Exact match                      → 1000.0
///   2. Prefix match                     → 200–499  (penalised by excess length)
///   3. Word-boundary / separator prefix → 100–199
///   4. Consecutive substring anywhere   → based on run length
///   5. General fuzzy subsequence        → calculated score
///   0. No match                         → 0.0
pub fn fuzzy_match(query: &str, target: &str) -> f64 {
    if query.is_empty() {
        return 0.0;
    }

    let q_lower = query.to_lowercase();
    let t_lower = target.to_lowercase();

    // Tier 1 – exact match
    if q_lower == t_lower {
        return 1000.0;
    }

    // Tier 2 – prefix match (target starts with the full query)
    if t_lower.starts_with(q_lower.as_str()) {
        // Shorter targets rank higher for the same prefix; use char count for Unicode safety.
        let excess = (t_lower.chars().count().saturating_sub(q_lower.chars().count())) as f64;
        return (499.0 - excess * 3.0).max(200.0);
    }

    // Tier 3 – query appears after a word-boundary separator in the target.
    // Single-pass scan: when a separator is encountered, check whether the
    // remainder of the string starts with the query.  This avoids the
    // O(#separators × target_len) cost of repeated split/traversal.
    {
        let sep_set = ['-', '_', '/', '.', ' ', '+', '@'];
        let mut after_sep = false;
        let mut word_boundary_match = false;
        for (byte_pos, ch) in t_lower.char_indices() {
            if sep_set.contains(&ch) {
                after_sep = true;
            } else if after_sep {
                if t_lower[byte_pos..].starts_with(q_lower.as_str()) {
                    word_boundary_match = true;
                    break;
                }
                after_sep = false;
            }
        }
        if word_boundary_match {
            let excess = (t_lower.chars().count().saturating_sub(q_lower.chars().count())) as f64;
            return (199.0 - excess * 2.0).max(100.0);
        }
    }

    // Tier 4 – consecutive substring anywhere (case-insensitive).
    // Note: pos == 0 is already handled by Tier 2, so pos_bonus is omitted.
    if t_lower.contains(q_lower.as_str()) {
        let excess = (t_lower.chars().count().saturating_sub(q_lower.chars().count())) as f64;
        return (99.0 - excess * 1.5).max(30.0);
    }

    // Tier 5 – general fuzzy subsequence
    let q: Vec<char> = q_lower.chars().collect();
    let t: Vec<char> = t_lower.chars().collect();

    let Some(indices) = fuzzy_get_indexes(&q, &t) else {
        return 0.0;
    };

    calculate_score(&q, &t, &indices)
}

pub fn fuzzy_get_indexes(query: &[char], target: &[char]) -> Option<Vec<usize>> {
    let mut out = Vec::with_capacity(query.len());
    let mut ti = 0;

    for &qc in query {
        let mut found = None;

        while ti < target.len() {
            if target[ti] == qc {
                found = Some(ti);
                ti += 1;
                break;
            }
            ti += 1;
        }

        let idx = found?;
        out.push(idx);
    }

    Some(out)
}

/// Fuzzy scoring inspired by VSCode's fuzzy finder algorithm.
/// Scores stay below 30 so they don't overlap with the substring tiers above.
pub fn calculate_score(query: &[char], target: &[char], indices: &[usize]) -> f64 {
    if query.is_empty() || indices.is_empty() {
        return 0.0;
    }

    let tlen = target.len() as f64;

    let mut score = 0.0;
    let mut consecutive = 0;

    for (i, &idx) in indices.iter().enumerate() {
        score += 1.0;

        if i > 0 && indices[i - 1] + 1 == idx {
            consecutive += 1;
            score += 1.0 + (consecutive as f64) * 0.3;
        } else {
            consecutive = 0;
        }

        // Bonus for match at start of target
        if idx == 0 {
            score += 4.0;
        }

        // Bonus for match after a separator
        if idx > 0 {
            let prev = target[idx - 1];
            if prev == '-' || prev == '_' || prev == '/' || prev == '.' || prev == ' ' {
                score += 2.5;
            }
        }

        // Gap penalty
        if i > 0 {
            let gap = (idx as i32 - indices[i - 1] as i32 - 1).max(0) as f64;
            score -= gap * 0.15;
        }
    }

    // Normalise: longer targets rank lower for the same raw score
    (score / (tlen * 0.15 + 1.0)).min(29.0)
}
