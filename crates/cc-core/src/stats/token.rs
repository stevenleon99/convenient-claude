/// Estimate cost based on token usage and model pricing.
/// These are approximate prices for Claude models (as of 2025).
pub fn estimate_cost(input_tokens: u64, output_tokens: u64, model: &str) -> f64 {
    let (input_price, output_price) = model_pricing(model);
    let input_cost = (input_tokens as f64 / 1_000_000.0) * input_price;
    let output_cost = (output_tokens as f64 / 1_000_000.0) * output_price;
    input_cost + output_cost
}

fn model_pricing(model: &str) -> (f64, f64) {
    // Returns (input_price_per_million, output_price_per_million) in USD
    let model_lower = model.to_lowercase();
    if model_lower.contains("opus") {
        (15.0, 75.0)
    } else if model_lower.contains("sonnet") {
        (3.0, 15.0)
    } else if model_lower.contains("haiku") {
        (0.25, 1.25)
    } else {
        // Default to Sonnet pricing
        (3.0, 15.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_cost_sonnet() {
        let cost = estimate_cost(100_000, 25_000, "sonnet");
        assert!(cost > 0.0);
        // 100k input * $3/M = $0.30, 25k output * $15/M = $0.375 → $0.675
        assert!((cost - 0.675).abs() < 0.001);
    }

    #[test]
    fn test_estimate_cost_opus() {
        let cost = estimate_cost(100_000, 25_000, "opus");
        assert!(cost > 0.675); // Opus is more expensive
    }

    #[test]
    fn test_estimate_cost_unknown_model() {
        let cost = estimate_cost(100_000, 25_000, "unknown");
        // Falls back to sonnet pricing
        assert!((cost - 0.675).abs() < 0.001);
    }
}
