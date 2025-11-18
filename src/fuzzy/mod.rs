pub fn fuzzy_get_indexes(query: &String, target: &String) -> Vec<i32> {
    let query_vector: Vec<char> = query.chars().collect();
    let target_vector: Vec<char> = target.chars().collect();

    let mut indexq = 0;
    let mut indext = 0;

    let mut incendices: Vec<i32> = Vec::new();

    while indexq < query_vector.len() && indext < target_vector.len() {
        if query_vector[indexq] == target_vector[indext] {
            incendices.push(indext as i32);
            indexq += 1;
        } else if !target_vector.contains(&query_vector[indexq]) {
            indexq += 1;
            continue;
        }
        indext += 1;
    }
    incendices
}
pub fn score(query: String, target: &String, indexes: Vec<i32>) -> f64 {
    if indexes.is_empty() {
        return 0.0;
    }

    let first = indexes[0];
    let last = indexes[indexes.len() - 1];

    let span = last - first + 1;
    if span <= 0 {
        return 0.0;
    }

    let span = indexes[indexes.len() - 1] - indexes[0] + 1;
    let gap = span - query.len() as i32;
    let tightness = if indexes.len() == 1 {
        0.4 / (target.len() as f64)
    } else {
        query.len() as f64 / span as f64
    };

    let mut adjacency = 0;
    for i in 1..indexes.len() {
        if indexes[i] == indexes[i - 1] + 1 {
            adjacency += 1;
        }
    }
    let first = indexes[0];
    let target_chars: Vec<char> = target.chars().collect();
    let word_bonus = if first == 0 || [' ', '-', '_'].contains(&target_chars[(first - 1) as usize])
    {
        1.2
    } else {
        1.0
    };

    let match_len_bonus = indexes.len() as f64 * 1.2;
    let gap_penalty = (-0.2 * gap as f64).exp();
    let starts_with_bonus = if first == 0 { 1.5 } else { 0.0 };

    let match_ratio = indexes.len() as f64 / target.len() as f64;
    let adjacency_ratio = adjacency as f64 / indexes.len().max(1) as f64;
    let length_penalty = 1.0 / (1.0 + (target.len() as f64 - query.len() as f64).max(0.0));

    tightness * 0.4
        + word_bonus * 0.3
        + adjacency_ratio * 0.3
        + match_ratio * 0.8
        + starts_with_bonus
        + length_penalty
        + match_len_bonus
        - gap_penalty
}
