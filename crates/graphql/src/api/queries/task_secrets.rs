use crate::schema;

#[derive(cynic::InlineFragments, Debug)]
pub enum ManagedSecretValue {
    ManagedSecretRawValue(ManagedSecretRawValue),
    ManagedSecretAnthropicApiKeyValue(ManagedSecretAnthropicApiKeyValue),
    ManagedSecretAnthropicBedrockAccessKeyValue(ManagedSecretAnthropicBedrockAccessKeyValue),
    ManagedSecretAnthropicBedrockApiKeyValue(ManagedSecretAnthropicBedrockApiKeyValue),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ManagedSecretRawValue {
    pub value: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ManagedSecretAnthropicApiKeyValue {
    pub api_key: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ManagedSecretAnthropicBedrockAccessKeyValue {
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
    /// Optional session token. Only set for temporary/STS credentials.
    pub aws_session_token: Option<String>,
    pub aws_region: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ManagedSecretAnthropicBedrockApiKeyValue {
    pub aws_bearer_token_bedrock: String,
    pub aws_region: String,
}
