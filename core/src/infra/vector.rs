//! Vector operations integration with infra-vector.
//!
//! This module provides embeddings, similarity calculations, and
//! vector indexing capabilities for LLM evaluation tasks.

use infra_vector::{Vector, VectorIndex, IndexConfig, SearchResult, cosine_similarity, euclidean_distance, dot_product};
use infra_errors::InfraResult;

/// Create a vector from a slice of floats
///
/// # Example
///
/// ```rust,ignore
/// use llm_test_bench_core::infra::vector::from_slice;
///
/// let v = from_slice(&[0.1, 0.2, 0.3]);
/// ```
pub fn from_slice(data: &[f32]) -> Vector {
    Vector::new(data.to_vec())
}

/// Create a zero vector of the given dimension
pub fn zeros(dim: usize) -> Vector {
    infra_vector::zeros(dim)
}

/// Create a random unit vector of the given dimension
pub fn random_unit(dim: usize) -> Vector {
    infra_vector::random_unit(dim)
}

/// Calculate cosine similarity between two embeddings
///
/// Returns a value between -1.0 and 1.0, where 1.0 indicates
/// identical direction and -1.0 indicates opposite direction.
///
/// # Example
///
/// ```rust,ignore
/// use llm_test_bench_core::infra::vector::{from_slice, similarity};
///
/// let v1 = from_slice(&[1.0, 0.0, 0.0]);
/// let v2 = from_slice(&[1.0, 0.0, 0.0]);
/// let sim = similarity(&v1, &v2)?;
/// assert!((sim - 1.0).abs() < 0.001);
/// ```
pub fn similarity(a: &Vector, b: &Vector) -> InfraResult<f32> {
    cosine_similarity(a, b)
}

/// Calculate Euclidean distance between two embeddings
pub fn distance(a: &Vector, b: &Vector) -> InfraResult<f32> {
    euclidean_distance(a, b)
}

/// Calculate dot product of two embeddings
pub fn dot(a: &Vector, b: &Vector) -> InfraResult<f32> {
    dot_product(a, b)
}

/// Create a new vector index for similarity search
///
/// # Example
///
/// ```rust,ignore
/// use llm_test_bench_core::infra::vector::{create_index, from_slice};
///
/// let mut index = create_index(384); // 384-dimensional embeddings
/// index.insert("doc-1", from_slice(&[...]))?;
///
/// let query = from_slice(&[...]);
/// let results = index.search(&query, 5)?; // top 5 results
/// ```
pub fn create_index(dimensions: usize) -> VectorIndex {
    VectorIndex::new(IndexConfig::new(dimensions))
}

/// Embedding dimensions for common models
pub mod dimensions {
    /// OpenAI text-embedding-ada-002
    pub const ADA_002: usize = 1536;
    /// OpenAI text-embedding-3-small
    pub const OPENAI_SMALL: usize = 1536;
    /// OpenAI text-embedding-3-large
    pub const OPENAI_LARGE: usize = 3072;
    /// Sentence transformers all-MiniLM-L6-v2
    pub const MINILM: usize = 384;
    /// Sentence transformers all-mpnet-base-v2
    pub const MPNET: usize = 768;
    /// Cohere embed-english-v3
    pub const COHERE_V3: usize = 1024;
    /// Voyage AI voyage-2
    pub const VOYAGE_2: usize = 1024;
}

/// Helper for comparing embeddings in evaluation tasks
pub struct EmbeddingComparator {
    threshold: f32,
}

impl EmbeddingComparator {
    /// Create a new comparator with a similarity threshold
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }

    /// Check if two embeddings are similar (above threshold)
    pub fn are_similar(&self, a: &Vector, b: &Vector) -> InfraResult<bool> {
        let sim = similarity(a, b)?;
        Ok(sim >= self.threshold)
    }

    /// Batch compare a query against multiple candidates
    pub fn find_similar(&self, query: &Vector, candidates: &[(String, Vector)]) -> InfraResult<Vec<(String, f32)>> {
        let mut results = Vec::new();
        for (id, candidate) in candidates {
            let sim = similarity(query, candidate)?;
            if sim >= self.threshold {
                results.push((id.clone(), sim));
            }
        }
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_slice() {
        let v = from_slice(&[1.0, 2.0, 3.0]);
        assert_eq!(v.dim(), 3);
    }

    #[test]
    fn test_zeros() {
        let v = zeros(10);
        assert_eq!(v.dim(), 10);
    }

    #[test]
    fn test_similarity_identical() {
        let v1 = from_slice(&[1.0, 0.0, 0.0]);
        let v2 = from_slice(&[1.0, 0.0, 0.0]);
        let sim = similarity(&v1, &v2).unwrap();
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_similarity_orthogonal() {
        let v1 = from_slice(&[1.0, 0.0, 0.0]);
        let v2 = from_slice(&[0.0, 1.0, 0.0]);
        let sim = similarity(&v1, &v2).unwrap();
        assert!(sim.abs() < 0.001);
    }

    #[test]
    fn test_embedding_comparator() {
        let comparator = EmbeddingComparator::new(0.9);
        let v1 = from_slice(&[1.0, 0.0, 0.0]);
        let v2 = from_slice(&[1.0, 0.0, 0.0]);
        assert!(comparator.are_similar(&v1, &v2).unwrap());
    }

    #[test]
    fn test_dimensions() {
        assert_eq!(dimensions::ADA_002, 1536);
        assert_eq!(dimensions::MINILM, 384);
    }
}
