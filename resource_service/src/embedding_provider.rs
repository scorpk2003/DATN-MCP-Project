use sha2::{Digest, Sha256};

use crate::{AppError, AppResult};

pub fn deterministic_embedding(input: &str, dimensions: usize) -> AppResult<Vec<f32>> {
    if dimensions == 0 {
        return Err(AppError::Validation(
            "embedding dimensions must be greater than zero".to_string(),
        ));
    }

    let mut vector = Vec::with_capacity(dimensions);
    let mut counter = 0_u32;
    while vector.len() < dimensions {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hasher.update(counter.to_le_bytes());
        let digest = hasher.finalize();
        for chunk in digest.chunks_exact(4) {
            if vector.len() >= dimensions {
                break;
            }
            let raw = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let unit = raw as f32 / u32::MAX as f32;
            vector.push((unit * 2.0) - 1.0);
        }
        counter = counter.wrapping_add(1);
    }

    normalize_l2(&mut vector);
    Ok(vector)
}

pub fn vector_literal(vector: &[f32]) -> AppResult<String> {
    if vector.is_empty() {
        return Err(AppError::Validation(
            "embedding vector must not be empty".to_string(),
        ));
    }
    let values = vector
        .iter()
        .map(|value| {
            if value.is_finite() {
                Ok(format!("{value:.8}"))
            } else {
                Err(AppError::Validation(
                    "embedding vector contains non-finite value".to_string(),
                ))
            }
        })
        .collect::<AppResult<Vec<_>>>()?;
    Ok(format!("[{}]", values.join(",")))
}

fn normalize_l2(vector: &mut [f32]) {
    let norm = vector
        .iter()
        .map(|value| (*value as f64) * (*value as f64))
        .sum::<f64>()
        .sqrt();
    if norm == 0.0 {
        return;
    }
    for value in vector {
        *value = (*value as f64 / norm) as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_embedding_is_stable_and_sized() {
        let first = deterministic_embedding("Rust ownership", 8).unwrap();
        let second = deterministic_embedding("Rust ownership", 8).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.len(), 8);
    }

    #[test]
    fn vector_literal_matches_pgvector_input_shape() {
        let literal = vector_literal(&[0.125, -0.5]).unwrap();

        assert_eq!(literal, "[0.12500000,-0.50000000]");
    }

    #[test]
    fn vector_literal_rejects_nan() {
        let err = vector_literal(&[f32::NAN]).unwrap_err();

        assert_eq!(err.code(), "VALIDATION_ERROR");
    }
}
