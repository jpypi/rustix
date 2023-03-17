use serde::{Deserialize, Serialize};

#[derive(Serialize, Copy, Clone)]
#[allow(unused)]
pub enum ModelType {
    #[serde(rename = "text-davinci-003")]
    Davinci,
    #[serde(rename = "text-curie-001")]
    Curie,
    #[serde(rename = "text-babbage-001")]
    Babbage,
    #[serde(rename = "text-ada-001")]
    Ada,
}

#[derive(Serialize, Copy, Clone)]
#[allow(unused)]
pub struct AIModel {
    kind: ModelType,
    cost_per_1k: f64,
    max_tokens: u64,
}

#[derive(Serialize)]
pub struct Query {
    pub model: ModelType,
    pub prompt: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub top_p: Option<f32>,
    pub n: Option<u32>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub user: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Response {
    Success(ResponseSuccess),
    Error(ResponseError)
}

#[derive(Deserialize)]
#[allow(unused)]
pub struct ResponseSuccess {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ResponseChoice>,
    pub usage: ResponseUsage,
}


#[derive(Deserialize)]
#[allow(unused)]
pub struct ResponseChoice {
    pub text: String,
    pub index: u32,
    pub logprobs: Option<String>,
    pub finish_reason: String,
}

#[derive(Deserialize)]
#[allow(unused)]
pub struct ResponseUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
pub struct ResponseError {
    error: ErrorContents,
}


#[derive(Deserialize, Debug)]
#[allow(unused)]
pub struct ErrorContents {
    message: String,
    #[serde(rename = "type")]
    type_: String,
    param: Option<String>,
    code: Option<String>,
}