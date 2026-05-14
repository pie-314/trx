pub fn fuzzy_match(query: &str, target: &str) -> f64 {
    if query.is_empty() {
        return 0.0;
    }

    let q: Vec<char> = query.to_lowercase().chars().collect();
    let t: Vec<char> = target.to_lowercase().chars().collect();

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

//fuzzy scoring inspired from VSCode fuzzy finder algorithm
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

        if idx == 0 {
            score += 4.0;
        }

        if idx > 0 {
            let prev = target[idx - 1];
            if prev == '-' || prev == '_' || prev == '/' || prev == '.' || prev == ' ' {
                score += 2.5;
            }
        }

        if i > 0 {
            let gap = (idx as i32 - indices[i - 1] as i32 - 1).max(0) as f64;
            score -= gap * 0.15;
        }
    }

    score / (tlen * 0.15 + 1.0)
}
