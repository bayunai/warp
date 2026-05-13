use crate::schema;

#[derive(cynic::QueryFragment, Debug)]
pub struct GeneratedCommand {
    pub command: String,
    pub description: String,
    pub parameters: Vec<GeneratedCommandParameter>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct GeneratedCommandParameter {
    pub description: String,
    pub id: String,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum GenerateCommandsFailureType {
    AiProviderError,
    BadPrompt,
    Other,
    RateLimited,
}
