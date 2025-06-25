import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { formatFileSize, formatTime } from '../../utils/formatting';

const DocumentsView: React.FC = () => {
  const [documents, setDocuments] = useState<any[]>([]);

  const handleAddDocument = async () => {
    try {
      await invoke('add_document');
      // Reload documents
    } catch (error) {
      console.error('Failed to add document:', error);
    }
  };

  return (
    <div className="documents-container">
      <div className="documents-header">
        <button onClick={handleAddDocument}>
          📁 Add Document
        </button>
        <input
          className="search-input"
          placeholder="Search your indexed documents..."
        />
      </div>
      <div className="documents-list">
        {documents.length === 0 ? (
          <div className="empty-state">
            <p>No documents indexed yet.</p>
            <p>Add some documents to get started!</p>
          </div>
        ) : (
          documents.map((doc) => (
            <div key={doc.id} className="document-card">
              <div className="doc-header">
                <h3>{doc.name}</h3>
                <div className="doc-type">{doc.doc_type}</div>
              </div>
              <div className="doc-details">
                <p><strong>Size:</strong> {formatFileSize(doc.size)}</p>
                <p><strong>Indexed:</strong> {formatTime(doc.indexed_at)}</p>
                {doc.summary && (
                  <div className="doc-summary">
                    {doc.summary}
                  </div>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default DocumentsView;