pub fn fuzzy_match(query: &str, target: &str) -> f64 {
    if query.is_empty() {
        return 0.0;
    }

    let query_lower: Vec<char> = query.to_lowercase().chars().collect();
    let target_lower: Vec<char> = target.to_lowercase().chars().collect();

    let indices = match fuzzy_get_indexes(&query_lower, &target_lower) {
        Some(i) => i,
        None => return 0.0,
    };

    if indices.len() != query_lower.len() {
        return 0.0;
    }

    calculate_score(&query_lower, &target_lower, &indices)
}
pub fn fuzzy_get_indexes(query: &[char], target: &[char]) -> Option<Vec<usize>> {
    let mut indices = Vec::with_capacity(query.len());
    let mut target_idx = 0;

    for &query_char in query {
        let found = target[target_idx..].iter().position(|&c| c == query_char);

        match found {
            Some(pos) => {
                indices.push(target_idx + pos);
                target_idx += pos + 1;
            }
            None => return None,
        }
    }

    Some(indices)
}

pub fn calculate_score(query: &[char], target: &[char], indices: &[usize]) -> f64 {
    if indices.is_empty() || query.is_empty() {
        return 0.0;
    }

    let query_len = query.len() as f64;
    let target_len = target.len() as f64;

    let match_ratio = query_len / target_len.max(1.0);

    let first_match_bonus = 1.0 - (indices[0] as f64 / target_len.max(1.0));

    let mut consecutive_count = 0;
    for i in 1..indices.len() {
        if indices[i] == indices[i - 1] + 1 {
            consecutive_count += 1;
        }
    }
    let consecutiveness = consecutive_count as f64 / query_len.max(1.0);

    let span = if indices.len() > 1 {
        (indices[indices.len() - 1] - indices[0] + 1) as f64
    } else {
        1.0
    };
    let compactness = query_len / span;

    let word_boundary_bonus = if indices[0] == 0 {
        0.8
    } else if indices[0] > 0 {
        let prev_char = target[indices[0] - 1];
        if prev_char == ' ' || prev_char == '-' || prev_char == '_' || prev_char == '/' {
            0.5
        } else {
            0.0
        }
    } else {
        0.0
    };

    let score = match_ratio * 0.3
        + first_match_bonus * 0.2
        + consecutiveness * 0.3
        + compactness * 0.1
        + word_boundary_bonus * 0.1;

    score
}
