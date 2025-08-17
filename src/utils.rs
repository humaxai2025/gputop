use unicode_width::{UnicodeWidthStr, UnicodeWidthChar};

pub fn create_sparkline(data: &[u64]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let max_value = *data.iter().max().unwrap_or(&1);
    let min_value = *data.iter().min().unwrap_or(&0);
    let range = if max_value == min_value { 1 } else { max_value - min_value };

    let spark_chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    
    data.iter()
        .map(|&value| {
            let normalized = ((value - min_value) as f64 / range as f64 * 7.0) as usize;
            spark_chars[normalized.min(7)]
        })
        .collect()
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{:.0} {}", size, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

pub fn truncate_string(s: &str, max_width: usize) -> String {
    if s.width() <= max_width {
        s.to_string()
    } else {
        let mut truncated = String::new();
        let mut current_width = 0;
        
        for ch in s.chars() {
            let ch_width = ch.width().unwrap_or(0);
            if current_width + ch_width + 3 > max_width {
                truncated.push_str("...");
                break;
            }
            truncated.push(ch);
            current_width += ch_width;
        }
        
        truncated
    }
}
