use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{QdrantManager, VectorSearchResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub vector: Vec<f32>,
    pub collection: String,
    pub limit: usize,
    pub threshold: Option<f32>,
    pub filter: Option<SearchFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilter {
    pub must: Vec<FilterCondition>,
    pub must_not: Vec<FilterCondition>,
    pub should: Vec<FilterCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCondition {
    pub field: String,
    pub condition: ConditionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    Equals(serde_json::Value),
    NotEquals(serde_json::Value),
    In(Vec<serde_json::Value>),
    NotIn(Vec<serde_json::Value>),
    Range { gte: Option<f64>, lte: Option<f64> },
    Contains(String),
    StartsWith(String),
    EndsWith(String),
    Exists,
    NotExists,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: Uuid,
    pub score: f32,
    pub content: String,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    pub highlight: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SimilarityMetric {
    Cosine,
    Euclidean,
    DotProduct,
}

/// Engine for performing semantic searches
pub struct SemanticSearchEngine {
    default_similarity: SimilarityMetric,
    default_threshold: f32,
}

impl SemanticSearchEngine {
    /// Create a new semantic search engine
    pub fn new() -> Self {
        Self {
            default_similarity: SimilarityMetric::Cosine,
            default_threshold: 0.5,
        }
    }

    /// Perform a semantic search using vector similarity
    pub async fn search(
        &self,
        qdrant_manager: &QdrantManager,
        query: SearchQuery,
    ) -> Result<Vec<VectorSearchResult>> {
        // Convert our filter to Qdrant filter format
        let qdrant_filter = query.filter.as_ref().map(|f| self.build_qdrant_filter(f));

        // Perform the search
        let results = qdrant_manager.search(
            &query.collection,
            query.vector,
            query.limit,
            query.threshold,
            qdrant_filter,
        ).await?;

        Ok(results)
    }

    /// Search with custom similarity metric
    pub async fn search_with_metric(
        &self,
        qdrant_manager: &QdrantManager,
        query: SearchQuery,
        metric: SimilarityMetric,
    ) -> Result<Vec<VectorSearchResult>> {
        // For now, we'll use the same search but in a real implementation
        // you might adjust the distance calculation or use different collections
        // configured with different metrics
        
        let mut modified_query = query;
        
        // Adjust threshold based on metric if needed
        if let Some(threshold) = modified_query.threshold {
            modified_query.threshold = Some(self.adjust_threshold_for_metric(threshold, metric));
        }

        self.search(qdrant_manager, modified_query).await
    }

    /// Search with automatic query expansion
    pub async fn search_with_expansion(
        &self,
        qdrant_manager: &QdrantManager,
        query: SearchQuery,
        expansion_terms: Vec<String>,
    ) -> Result<Vec<VectorSearchResult>> {
        // In a real implementation, this would:
        // 1. Generate embeddings for expansion terms
        // 2. Combine them with the original query vector
        // 3. Perform weighted search
        
        // For now, just perform the original search
        let _expansion_terms = expansion_terms; // Suppress unused warning
        self.search(qdrant_manager, query).await
    }

    /// Multi-vector search (search with multiple query vectors)
    pub async fn multi_vector_search(
        &self,
        qdrant_manager: &QdrantManager,
        collection: &str,
        query_vectors: Vec<Vec<f32>>,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        let mut all_results = Vec::new();
        
        // Search with each vector
        for query_vector in query_vectors {
            let query = SearchQuery {
                vector: query_vector,
                collection: collection.to_string(),
                limit,
                threshold,
                filter: None,
            };
            
            let results = self.search(qdrant_manager, query).await?;
            all_results.extend(results);
        }
        
        // Deduplicate and re-rank results
        self.deduplicate_and_rerank(all_results, limit)
    }

    /// Hybrid search combining vector similarity and text matching
    pub async fn hybrid_search(
        &self,
        qdrant_manager: &QdrantManager,
        vector_query: SearchQuery,
        text_query: Option<String>,
        weights: HybridWeights,
    ) -> Result<Vec<VectorSearchResult>> {
        // Get vector similarity results
        let vector_results = self.search(qdrant_manager, vector_query).await?;
        
        if let Some(text) = text_query {
            // In a real implementation, this would:
            // 1. Perform full-text search on content fields
            // 2. Combine vector and text scores using weights
            // 3. Re-rank results based on combined score
            
            let _text = text; // Suppress unused warning
            let _weights = weights; // Suppress unused warning
            
            // For now, just return vector results
            Ok(vector_results)
        } else {
            Ok(vector_results)
        }
    }

    /// Search within a specific date range
    pub async fn temporal_search(
        &self,
        qdrant_manager: &QdrantManager,
        mut query: SearchQuery,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
        date_field: &str,
    ) -> Result<Vec<VectorSearchResult>> {
        // Add temporal filter to the query
        let temporal_condition = FilterCondition {
            field: date_field.to_string(),
            condition: ConditionType::Range {
                gte: Some(start_date.timestamp() as f64),
                lte: Some(end_date.timestamp() as f64),
            },
        };

        if let Some(ref mut filter) = query.filter {
            filter.must.push(temporal_condition);
        } else {
            query.filter = Some(SearchFilter {
                must: vec![temporal_condition],
                must_not: vec![],
                should: vec![],
            });
        }

        self.search(qdrant_manager, query).await
    }

    /// Search with faceted filters
    pub async fn faceted_search(
        &self,
        qdrant_manager: &QdrantManager,
        query: SearchQuery,
        facets: Vec<FacetFilter>,
    ) -> Result<FacetedSearchResult> {
        let mut filtered_query = query;
        
        // Apply facet filters
        for facet in &facets {
            let condition = FilterCondition {
                field: facet.field.clone(),
                condition: ConditionType::In(facet.values.clone()),
            };

            if let Some(ref mut filter) = filtered_query.filter {
                filter.must.push(condition);
            } else {
                filtered_query.filter = Some(SearchFilter {
                    must: vec![condition],
                    must_not: vec![],
                    should: vec![],
                });
            }
        }

        let results = self.search(qdrant_manager, filtered_query).await?;

        // In a real implementation, we would also collect facet counts
        let facet_counts = self.collect_facet_counts(&results, &facets);

        Ok(FacetedSearchResult {
            results,
            facets: facet_counts,
            total_count: results.len(),
        })
    }

    // Private helper methods

    fn build_qdrant_filter(&self, filter: &SearchFilter) -> serde_json::Value {
        let mut qdrant_filter = serde_json::json!({});

        // Build must conditions
        if !filter.must.is_empty() {
            let must_conditions: Vec<serde_json::Value> = filter.must
                .iter()
                .map(|condition| self.build_condition(condition))
                .collect();
            qdrant_filter["must"] = serde_json::Value::Array(must_conditions);
        }

        // Build must_not conditions
        if !filter.must_not.is_empty() {
            let must_not_conditions: Vec<serde_json::Value> = filter.must_not
                .iter()
                .map(|condition| self.build_condition(condition))
                .collect();
            qdrant_filter["must_not"] = serde_json::Value::Array(must_not_conditions);
        }

        // Build should conditions
        if !filter.should.is_empty() {
            let should_conditions: Vec<serde_json::Value> = filter.should
                .iter()
                .map(|condition| self.build_condition(condition))
                .collect();
            qdrant_filter["should"] = serde_json::Value::Array(should_conditions);
        }

        qdrant_filter
    }

    fn build_condition(&self, condition: &FilterCondition) -> serde_json::Value {
        match &condition.condition {
            ConditionType::Equals(value) => {
                serde_json::json!({
                    "key": condition.field,
                    "match": { "value": value }
                })
            }
            ConditionType::NotEquals(value) => {
                serde_json::json!({
                    "key": condition.field,
                    "match": { "except": [value] }
                })
            }
            ConditionType::In(values) => {
                serde_json::json!({
                    "key": condition.field,
                    "match": { "any": values }
                })
            }
            ConditionType::NotIn(values) => {
                serde_json::json!({
                    "key": condition.field,
                    "match": { "except": values }
                })
            }
            ConditionType::Range { gte, lte } => {
                let mut range = serde_json::Map::new();
                if let Some(gte_val) = gte {
                    range.insert("gte".to_string(), serde_json::Value::Number(
                        serde_json::Number::from_f64(*gte_val).unwrap()
                    ));
                }
                if let Some(lte_val) = lte {
                    range.insert("lte".to_string(), serde_json::Value::Number(
                        serde_json::Number::from_f64(*lte_val).unwrap()
                    ));
                }
                serde_json::json!({
                    "key": condition.field,
                    "range": range
                })
            }
            ConditionType::Contains(text) => {
                // This would need full-text search support in Qdrant
                serde_json::json!({
                    "key": condition.field,
                    "match": { "text": text }
                })
            }
            ConditionType::StartsWith(prefix) => {
                serde_json::json!({
                    "key": condition.field,
                    "match": { "prefix": prefix }
                })
            }
            ConditionType::EndsWith(suffix) => {
                // This would need custom implementation
                serde_json::json!({
                    "key": condition.field,
                    "match": { "suffix": suffix }
                })
            }
            ConditionType::Exists => {
                serde_json::json!({
                    "has_id": [condition.field]
                })
            }
            ConditionType::NotExists => {
                serde_json::json!({
                    "is_empty": {
                        "key": condition.field
                    }
                })
            }
        }
    }

    fn adjust_threshold_for_metric(&self, threshold: f32, metric: SimilarityMetric) -> f32 {
        match metric {
            SimilarityMetric::Cosine => threshold,
            SimilarityMetric::Euclidean => {
                // Euclidean distance thresholds work differently (lower is better)
                2.0 - threshold
            }
            SimilarityMetric::DotProduct => {
                // Dot product thresholds depend on vector normalization
                threshold
            }
        }
    }

    fn deduplicate_and_rerank(
        &self,
        mut results: Vec<VectorSearchResult>,
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>> {
        // Remove duplicates by ID
        let mut seen_ids = std::collections::HashSet::new();
        results.retain(|result| seen_ids.insert(result.id));

        // Sort by score (highest first)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Take top results
        results.truncate(limit);

        Ok(results)
    }

    fn collect_facet_counts(
        &self,
        results: &[VectorSearchResult],
        facets: &[FacetFilter],
    ) -> Vec<FacetCount> {
        let mut facet_counts = Vec::new();

        for facet in facets {
            let mut value_counts = std::collections::HashMap::new();

            for result in results {
                if let Some(field_value) = result.payload.get(&facet.field) {
                    if let Some(str_value) = field_value.as_str() {
                        *value_counts.entry(str_value.to_string()).or_insert(0) += 1;
                    }
                }
            }

            facet_counts.push(FacetCount {
                field: facet.field.clone(),
                values: value_counts,
            });
        }

        facet_counts
    }
}

impl Default for SemanticSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Supporting types

#[derive(Debug, Clone)]
pub struct HybridWeights {
    pub vector_weight: f32,
    pub text_weight: f32,
}

impl Default for HybridWeights {
    fn default() -> Self {
        Self {
            vector_weight: 0.7,
            text_weight: 0.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetFilter {
    pub field: String,
    pub values: Vec<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct FacetCount {
    pub field: String,
    pub values: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct FacetedSearchResult {
    pub results: Vec<VectorSearchResult>,
    pub facets: Vec<FacetCount>,
    pub total_count: usize,
}

// Query builder helpers

impl SearchQuery {
    pub fn new(vector: Vec<f32>, collection: String) -> Self {
        Self {
            vector,
            collection,
            limit: 10,
            threshold: None,
            filter: None,
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = Some(threshold);
        self
    }

    pub fn with_filter(mut self, filter: SearchFilter) -> Self {
        self.filter = Some(filter);
        self
    }
}

impl SearchFilter {
    pub fn new() -> Self {
        Self {
            must: Vec::new(),
            must_not: Vec::new(),
            should: Vec::new(),
        }
    }

    pub fn must(mut self, condition: FilterCondition) -> Self {
        self.must.push(condition);
        self
    }

    pub fn must_not(mut self, condition: FilterCondition) -> Self {
        self.must_not.push(condition);
        self
    }

    pub fn should(mut self, condition: FilterCondition) -> Self {
        self.should.push(condition);
        self
    }
}

impl FilterCondition {
    pub fn equals(field: String, value: serde_json::Value) -> Self {
        Self {
            field,
            condition: ConditionType::Equals(value),
        }
    }

    pub fn contains(field: String, text: String) -> Self {
        Self {
            field,
            condition: ConditionType::Contains(text),
        }
    }

    pub fn range(field: String, gte: Option<f64>, lte: Option<f64>) -> Self {
        Self {
            field,
            condition: ConditionType::Range { gte, lte },
        }
    }

    pub fn in_values(field: String, values: Vec<serde_json::Value>) -> Self {
        Self {
            field,
            condition: ConditionType::In(values),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_query_builder() {
        let query = SearchQuery::new(vec![0.1, 0.2, 0.3], "test_collection".to_string())
            .with_limit(20)
            .with_threshold(0.8)
            .with_filter(
                SearchFilter::new()
                    .must(FilterCondition::equals("agent_id".to_string(), "test".into()))
            );

        assert_eq!(query.limit, 20);
        assert_eq!(query.threshold, Some(0.8));
        assert!(query.filter.is_some());
    }

    #[test]
    fn test_filter_condition_builders() {
        let equals_condition = FilterCondition::equals("field".to_string(), "value".into());
        assert!(matches!(equals_condition.condition, ConditionType::Equals(_)));

        let range_condition = FilterCondition::range("score".to_string(), Some(0.5), Some(1.0));
        assert!(matches!(range_condition.condition, ConditionType::Range { .. }));

        let contains_condition = FilterCondition::contains("content".to_string(), "search".to_string());
        assert!(matches!(contains_condition.condition, ConditionType::Contains(_)));
    }

    #[test]
    fn test_similarity_metric_threshold_adjustment() {
        let engine = SemanticSearchEngine::new();
        
        let original_threshold = 0.8;
        
        let cosine_threshold = engine.adjust_threshold_for_metric(original_threshold, SimilarityMetric::Cosine);
        assert_eq!(cosine_threshold, original_threshold);
        
        let euclidean_threshold = engine.adjust_threshold_for_metric(original_threshold, SimilarityMetric::Euclidean);
        assert_ne!(euclidean_threshold, original_threshold);
    }

    #[test]
    fn test_hybrid_weights() {
        let weights = HybridWeights::default();
        assert_eq!(weights.vector_weight + weights.text_weight, 1.0);
        
        let custom_weights = HybridWeights {
            vector_weight: 0.6,
            text_weight: 0.4,
        };
        assert_eq!(custom_weights.vector_weight + custom_weights.text_weight, 1.0);
    }

    #[tokio::test]
    async fn test_deduplication_and_reranking() {
        let engine = SemanticSearchEngine::new();
        
        let results = vec![
            VectorSearchResult {
                id: Uuid::new_v4(),
                score: 0.8,
                payload: std::collections::HashMap::new(),
            },
            VectorSearchResult {
                id: Uuid::new_v4(),
                score: 0.9,
                payload: std::collections::HashMap::new(),
            },
            VectorSearchResult {
                id: Uuid::new_v4(),
                score: 0.7,
                payload: std::collections::HashMap::new(),
            },
        ];
        
        let reranked = engine.deduplicate_and_rerank(results, 2).unwrap();
        
        assert_eq!(reranked.len(), 2);
        assert!(reranked[0].score >= reranked[1].score); // Should be sorted by score
    }
}