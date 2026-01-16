use std::borrow::Cow;
use alloy_primitives::{Address, Bytes};
use revm::precompile::{
    u64_to_address, PrecompileError, PrecompileOutput, 
    PrecompileResult, Precompile, PrecompileId,
};

/// Poseidon hash precompile for XLayer
/// Address: 0x0000000000000000000000000000000000000100
pub const POSEIDON_ADDRESS: Address = u64_to_address(0x100);

/// Poseidon precompile instance
pub const POSEIDON: Precompile = Precompile::new(
    PrecompileId::Custom(Cow::Borrowed("poseidon")),
    POSEIDON_ADDRESS,
    poseidon_run
);

/// Gas costs for Poseidon precompile
const POSEIDON_BASE_GAS: u64 = 60;
const POSEIDON_PER_INPUT_GAS: u64 = 6;

/// Run Poseidon hash computation
pub fn poseidon_run(input: &[u8], gas_limit: u64) -> PrecompileResult {
    let num_inputs = if input.is_empty() { 0 } else { input.len() / 32 };
    let gas_cost = POSEIDON_BASE_GAS + (num_inputs as u64) * POSEIDON_PER_INPUT_GAS;
    
    tracing::info!(
        target: "xlayer::poseidon",
        input_len = input.len(),
        num_inputs = num_inputs,
        gas_cost = gas_cost,
        gas_limit = gas_limit,
        "🔮 Poseidon precompile called"
    );
    
    if gas_cost > gas_limit {
        tracing::warn!(
            target: "xlayer::poseidon",
            gas_cost = gas_cost,
            gas_limit = gas_limit,
            "❌ Poseidon precompile: Out of gas"
        );
        return Err(PrecompileError::OutOfGas);
    }
    
    if !input.is_empty() && input.len() % 32 != 0 {
        tracing::warn!(
            target: "xlayer::poseidon",
            input_len = input.len(),
            "❌ Poseidon precompile: Invalid input length"
        );
        return Err(PrecompileError::other("input length must be multiple of 32"));
    }
    
    // Execute Poseidon hash.
    let output = poseidon_hash(input)?;
    
    tracing::info!(
        target: "xlayer::poseidon",
        output_len = output.len(),
        gas_used = gas_cost,
        "✅ Poseidon precompile executed successfully"
    );
    
    Ok(PrecompileOutput::new(gas_cost, output))
}

/// Compute Poseidon hash
/// 
/// Input format: N * 32 bytes (N field elements)
/// Output format: 32 bytes (one field element)
fn poseidon_hash(input: &[u8]) -> Result<Bytes, PrecompileError> {
    use poseidon_rs::{Poseidon, Fr};
    use num_bigint::{BigInt, Sign};
    use ff_ce::PrimeField; 
    use std::str::FromStr;
    
    if input.is_empty() {
        return Ok(Bytes::from(vec![0u8; 32]));
    }
    
    let num_inputs = input.len() / 32;
    let mut fr_inputs = Vec::with_capacity(num_inputs);
    
    for i in 0..num_inputs {
        let start = i * 32;
        let end = start + 32;
        let chunk = &input[start..end];
        
        // Convert 32 bytes -> BigInt -> decimal string -> Fr.
        let big_int = BigInt::from_bytes_be(Sign::Plus, chunk);
        let decimal_str = big_int.to_string();
        
        let fr = Fr::from_str(&decimal_str).ok_or_else(|| {
            PrecompileError::other("Invalid field element: Fr::from_str returned None")
        })?;
        
        fr_inputs.push(fr);
    }
    
    let poseidon = Poseidon::new();
    let hash_result = poseidon.hash(fr_inputs)
        .map_err(|e| PrecompileError::other(format!("Poseidon hash failed: {}", e)))?;
    
    let result_str = hash_result.to_string();
    let result_bigint = BigInt::from_str(&result_str)
        .map_err(|e| PrecompileError::other(format!("Failed to convert result: {:?}", e)))?;
    let (sign, result_bytes) = result_bigint.to_bytes_be();
    
    if sign == Sign::Minus {
        return Err(PrecompileError::other("Poseidon hash resulted in negative value"));
    }
    
    let mut output = vec![0u8; 32];
    let start_pos = 32_usize.saturating_sub(result_bytes.len());
    output[start_pos..].copy_from_slice(&result_bytes);
    
    Ok(Bytes::from(output))
}
