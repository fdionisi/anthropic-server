#[derive(Clone, clap::Subcommand)]
pub enum Provider {
    /// Use the Anthropic API
    Anthropic {
        #[clap(long, env = "ANTHROPIC_API_KEY")]
        api_key: String,
    },
    /// Use the Google Vertex AI API
    VertexAi {
        #[clap(long, env = "VERTEXAI_PROJECT")]
        project: String,
        #[clap(long, env = "VERTEXAI_REGION")]
        region: String,
    },
}
