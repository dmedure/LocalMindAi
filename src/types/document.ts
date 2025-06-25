export interface Document {
  id: string;
  name: string;
  doc_type: string;
  size: number;
  path: string;
  summary?: string;
  indexed_at: string;
}