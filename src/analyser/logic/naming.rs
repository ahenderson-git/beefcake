pub fn sanitize_column_name(name: &str) -> String {
    let mut clean = name.trim().to_lowercase();

    // Replace non-alphanumeric with underscore
    clean = clean
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();

    // Collapse multiple underscores
    let mut result = String::new();
    let mut last_was_underscore = false;
    for c in clean.chars() {
        if c == '_' {
            if !last_was_underscore {
                result.push(c);
                last_was_underscore = true;
            }
        } else {
            result.push(c);
            last_was_underscore = false;
        }
    }

    // Trim underscores from ends
    let mut result = result.trim_matches('_').to_string();

    // Ensure it doesn't start with a number
    if !result.is_empty() && result.chars().next().unwrap().is_ascii_digit() {
        result = format!("col_{}", result);
    }

    if result.is_empty() {
        "col".to_string()
    } else {
        result
    }
}

pub fn sanitize_column_names(names: &[String]) -> Vec<String> {
    let mut cleaned_names = Vec::new();
    let mut seen = std::collections::HashMap::new();

    for name in names {
        let clean_base = sanitize_column_name(name);
        let mut clean = clean_base.clone();
        let mut count = 0;

        while seen.contains_key(&clean) {
            count += 1;
            clean = format!("{}_{}", clean_base, count);
        }

        seen.insert(clean.clone(), true);
        cleaned_names.push(clean);
    }
    cleaned_names
}
