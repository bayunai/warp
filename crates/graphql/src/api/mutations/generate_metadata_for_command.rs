use crate::schema;

#[derive(cynic::QueryFragment, Debug)]
pub struct GenerateMetadataForCommandSuccess {
    pub description: String,
    pub parameterized_command: String,
    pub parameters: Vec<GeneratedMetadataForCommand>,
    pub title: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct GeneratedMetadataForCommand {
    pub description: String,
    pub name: String,
    pub value: String,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum GenerateMetadataForCommandFailureType {
    AiProviderError,
    BadCommand,
    Other,
    RateLimited,
}
