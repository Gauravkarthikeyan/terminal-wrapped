//! One-line bar charts made of block characters.

const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Scale `values` so the largest one gets a full block. Zero stays blank, so a
/// quiet hour reads as a gap rather than a tiny bar.
pub fn sparkline(values: &[usize]) -> String {
    let max = values.iter().copied().max().unwrap_or(0);
    values
        .iter()
        .map(|&v| match v {
            0 => ' ',
            // Round up so any activity shows at least the smallest bar.
            v => BARS[(v * BARS.len()).div_ceil(max) - 1],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::sparkline;

    #[test]
    fn scales_to_the_maximum() {
        assert_eq!(sparkline(&[0, 1, 4, 8]), " ▁▄█");
    }

    #[test]
    fn small_values_still_show() {
        assert_eq!(sparkline(&[1, 1000]), "▁█");
    }

    #[test]
    fn all_zero_or_empty() {
        assert_eq!(sparkline(&[0, 0]), "  ");
        assert_eq!(sparkline(&[]), "");
    }
}
