/// Calculating the optimal gas price
pub async fn cal_optimal_gas_price(client: &crate::Aptos) -> Result<u64, String> {
    let gas_price = client.estimate_gas_price().await?;
    Ok((gas_price as f64 * 1.1) as u64)
}

/// Estimate transaction costs
pub fn estimate_transaction_cost(gas_units: u64, gas_price: u64) -> f64 {
    (gas_units as f64 * gas_price as f64) / 100_000_000.0
}
