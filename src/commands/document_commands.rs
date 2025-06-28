use tauri::State;
use crate::types::{Document, AppState};
use crate::storage::DocumentStorage;
use crate::utils::{validation, error::LocalMindError, Result};

/// Get all documents
#[tauri::command]
pub async fn get_documents(state: State<'_, AppState>) -> Result<Vec<Document>, String> {
    let documents = state.documents.lock().await;
    Ok(documents.clone())
}

/// Add a new document
#[tauri::command]
pub async fn add_document(file_path: String, state: State<'_, AppState>) -> Result<Document, String> {
    // Validate file path
    validation::validate_file_path(&file_path)
        .map_err(|e| e.to_string())?;

    // Get file metadata
    let metadata = std::fs::metadata(&file_path)
        .map_err(|e| LocalMindError::FileSystem(format!("Cannot read file metadata: {}", e)).to_string())?;

    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Create document
    let document = Document::new(file_name, file_path, metadata.len());

    // Add to state
    let mut documents = state.documents.lock().await;
    documents.push(document.clone());

    // Save to storage
    DocumentStorage::save(&documents).await
        .map_err(|e| e.to_string())?;

    Ok(document)
}

/// Remove a document by ID
#[tauri::command]
pub async fn remove_document(document_id: String, state: State<'_, AppState>) -> Result<bool, String> {
    validation::validate_uuid(&document_id)
        .map_err(|e| e.to_string())?;

    // Remove from state
    let mut documents = state.documents.lock().await;
    let initial_len = documents.len();
    documents.retain(|doc| doc.id != document_id);

    let was_removed = documents.len() < initial_len;

    if was_removed {
        // Save updated list
        DocumentStorage::save(&documents).await
            .map_err(|e| e.to_string())?;
    }

    Ok(was_removed)
}

/// Get a document by ID
#[tauri::command]
pub async fn get_document_by_id(document_id: String, state: State<'_, AppState>) -> Result<Option<Document>, String> {
    validation::validate_uuid(&document_id)
        .map_err(|e| e.to_string())?;

    let documents = state.documents.lock().await;
    Ok(documents.iter().find(|doc| doc.id == document_id).cloned())
}

/// Update a document's summary
#[tauri::command]
pub async fn update_document_summary(
    document_id: String,
    summary: String,
    state: State<'_, AppState>,
) -> Result<Document, String> {
    validation::validate_uuid(&document_id)
        .map_err(|e| e.to_string())?;

    let mut documents = state.documents.lock().await;
    
    let document = documents
        .iter_mut()
        .find(|doc| doc.id == document_id)
        .ok_or_else(|| LocalMindError::Document(format!("Document not found: {}", document_id)).to_string())?;

    document.summary = if summary.trim().is_empty() {
        None
    } else {
        Some(summary.trim().to_string())
    };

    let updated_document = document.clone();

    // Save to storage
    DocumentStorage::save(&documents).await
        .map_err(|e| e.to_string())?;

    Ok(updated_document)
}

/// Search documents by name or content
#[tauri::command]
pub async fn search_documents(
    query: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<Document>, String> {
    if query.trim().is_empty() {
        return Err("Search query cannot be empty".to_string());
    }

    let query_lower = query.to_lowercase();
    let limit = limit.unwrap_or(50).min(200); // Cap at 200 results

    let documents = state.documents.lock().await;
    let mut matching_documents = Vec::new();

    for document in documents.iter() {
        let name_matches = document.name.to_lowercase().contains(&query_lower);
        let summary_matches = document.summary
            .as_ref()
            .map(|s| s.to_lowercase().contains(&query_lower))
            .unwrap_or(false);
        let type_matches = document.doc_type.to_lowercase().contains(&query_lower);

        if name_matches || summary_matches || type_matches {
            matching_documents.push(document.clone());
            if matching_documents.len() >= limit {
                break;
            }
        }
    }

    // Sort by relevance (name matches first, then summary matches)
    matching_documents.sort_by(|a, b| {
        let a_name_match = a.name.to_lowercase().contains(&query_lower);
        let b_name_match = b.name.to_lowercase().contains(&query_lower);
        
        match (a_name_match, b_name_match) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    Ok(matching_documents)
}

/// Get documents by type
#[tauri::command]
pub async fn get_documents_by_type(doc_type: String, state: State<'_, AppState>) -> Result<Vec<Document>, String> {
    let documents = state.documents.lock().await;
    let filtered: Vec<Document> = documents
        .iter()
        .filter(|doc| doc.doc_type.to_lowercase() == doc_type.to_lowercase())
        .cloned()
        .collect();

    Ok(filtered)
}

/// Get document statistics
#[tauri::command]
pub async fn get_document_statistics(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let documents = state.documents.lock().await;
    
    let total_count = documents.len();
    let total_size: u64 = documents.iter().map(|doc| doc.size).sum();
    
    // Count by type
    let mut type_counts = std::collections::HashMap::new();
    let mut type_sizes = std::collections::HashMap::new();
    
    for document in documents.iter() {
        *type_counts.entry(document.doc_type.clone()).or_insert(0) += 1;
        *type_sizes.entry(document.doc_type.clone()).or_insert(0u64) += document.size;
    }
    
    // Count documents with summaries
    let documents_with_summaries = documents.iter().filter(|doc| doc.has_summary()).count();
    
    // Find largest and smallest documents
    let largest_document = documents.iter().max_by_key(|doc| doc.size);
    let smallest_document = documents.iter().min_by_key(|doc| doc.size);
    
    Ok(serde_json::json!({
        "total_documents": total_count,
        "total_size_bytes": total_size,
        "total_size_mb": total_size as f64 / 1024.0 / 1024.0,
        "documents_with_summaries": documents_with_summaries,
        "type_counts": type_counts,
        "type_sizes": type_sizes,
        "largest_document": largest_document.map(|doc| serde_json::json!({
            "id": doc.id,
            "name": doc.name,
            "size": doc.size,
            "type": doc.doc_type
        })),
        "smallest_document": smallest_document.map(|doc| serde_json::json!({
            "id": doc.id,
            "name": doc.name,
            "size": doc.size,
            "type": doc.doc_type
        })),
        "average_size": if total_count > 0 { total_size / total_count as u64 } else { 0 },
    }))
}

/// Read document content (be careful with large files)
#[tauri::command]
pub async fn read_document_content(document_id: String, state: State<'_, AppState>) -> Result<String, String> {
    validation::validate_uuid(&document_id)
        .map_err(|e| e.to_string())?;

    let documents = state.documents.lock().await;
    let document = documents
        .iter()
        .find(|doc| doc.id == document_id)
        .ok_or_else(|| LocalMindError::Document(format!("Document not found: {}", document_id)).to_string())?;

    // Check file size limit (10MB)
    if document.size > 10_000_000 {
        return Err("Document too large to read (>10MB)".to_string());
    }

    // Check if file still exists
    if !std::path::Path::new(&document.path).exists() {
        return Err("Document file no longer exists".to_string());
    }

    // Read file content
    let content = std::fs::read_to_string(&document.path)
        .map_err(|e| format!("Failed to read document: {}", e))?;

    Ok(content)
}

/// Check if document file still exists
#[tauri::command]
pub async fn check_document_exists(document_id: String, state: State<'_, AppState>) -> Result<bool, String> {
    validation::validate_uuid(&document_id)
        .map_err(|e| e.to_string())?;

    let documents = state.documents.lock().await;
    let document = documents
        .iter()
        .find(|doc| doc.id == document_id)
        .ok_or_else(|| LocalMindError::Document(format!("Document not found: {}", document_id)).to_string())?;

    Ok(std::path::Path::new(&document.path).exists())
}

/// Export document list to JSON
#[tauri::command]
pub async fn export_document_list(state: State<'_, AppState>) -> Result<String, String> {
    let documents = state.documents.lock().await;
    
    let export_data = serde_json::json!({
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "total_documents": documents.len(),
        "documents": *documents,
    });

    serde_json::to_string_pretty(&export_data)
        .map_err(|e| format!("Failed to serialize document list: {}", e))
}

/// Clear all documents (use with caution)
#[tauri::command]
pub async fn clear_all_documents(state: State<'_, AppState>) -> Result<usize, String> {
    let mut documents = state.documents.lock().await;
    let count = documents.len();
    documents.clear();

    // Save empty list
    DocumentStorage::save(&documents).await
        .map_err(|e| e.to_string())?;

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AppState;
    use std::sync::Arc;

    fn create_test_state() -> Arc<AppState> {
        Arc::new(AppState::new())
    }

    #[tokio::test]
    async fn test_document_operations() {
        let state = create_test_state();
        let tauri_state = State::from(state.as_ref());

        // Test getting empty documents list
        let documents = get_documents(tauri_state).await.unwrap();
        assert_eq!(documents.len(), 0);

        // Test getting statistics for empty list
        let stats = get_document_statistics(tauri_state).await.unwrap();
        assert_eq!(stats["total_documents"], 0);
    }

    #[tokio::test]
    async fn test_search_validation() {
        let state = create_test_state();
        let tauri_state = State::from(state.as_ref());

        // Test empty search query
        let result = search_documents("".to_string(), None, tauri_state).await;
        assert!(result.is_err());

        // Test whitespace-only query
        let result = search_documents("   ".to_string(), None, tauri_state).await;
        assert!(result.is_err());
    }
}