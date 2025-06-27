#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub app: AppSettings,
    pub llm: LLMConfig,
    pub memory: MemoryConfig,
    pub vector: VectorConfig,
    pub platform: PlatformConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LLMConfig {
    pub default_model: String,
    pub models_dir: PathBuf,
    pub max_context_length: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemoryConfig {
    pub max_short_term: usize,
    pub consolidation_threshold: usize,
    pub importance_threshold: f32,
    pub max_working_memory: usize,
    pub retention_days: u32,
}