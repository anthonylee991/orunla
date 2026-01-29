/// Calculate precision, recall, and F1 score from TP, FP, FN counts
///
/// Returns (precision, recall, f1_score) as f32 values between 0.0 and 1.0
pub fn calculate_metrics(
    true_positives: usize,
    false_positives: usize,
    false_negatives: usize,
) -> (f32, f32, f32) {
    let tp = true_positives as f32;
    let fp = false_positives as f32;
    let fn_ = false_negatives as f32;

    // Precision = TP / (TP + FP)
    let precision = if tp + fp > 0.0 { tp / (tp + fp) } else { 1.0 };

    // Recall = TP / (TP + FN)
    let recall = if tp + fn_ > 0.0 { tp / (tp + fn_) } else { 1.0 };

    // F1 = 2 * (precision * recall) / (precision + recall)
    let f1_score = if precision + recall > 0.0 {
        2.0 * (precision * recall) / (precision + recall)
    } else {
        0.0
    };

    (precision, recall, f1_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_score() {
        let (p, r, f1) = calculate_metrics(10, 0, 0);
        assert!((p - 1.0).abs() < 0.001);
        assert!((r - 1.0).abs() < 0.001);
        assert!((f1 - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_zero_matches() {
        let (p, r, f1) = calculate_metrics(0, 5, 10);
        assert!((p - 0.0).abs() < 0.001);
        assert!((r - 0.0).abs() < 0.001);
        assert!((f1 - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_partial_matches() {
        // 5 correct, 2 extra, 3 missed
        let (p, r, f1) = calculate_metrics(5, 2, 3);
        // Precision = 5/7 = 0.714
        assert!((p - 0.714).abs() < 0.01);
        // Recall = 5/8 = 0.625
        assert!((r - 0.625).abs() < 0.01);
        // F1 = 2 * 0.714 * 0.625 / (0.714 + 0.625) = 0.667
        assert!((f1 - 0.667).abs() < 0.01);
    }
}
