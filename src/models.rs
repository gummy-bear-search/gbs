use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSettings {
    pub number_of_shards: Option<u32>,
    pub number_of_replicas: Option<u32>,
    pub analysis: Option<AnalysisSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSettings {
    pub analyzer: Option<std::collections::HashMap<String, Analyzer>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analyzer {
    #[serde(rename = "type")]
    pub type_: String,
    pub tokenizer: Option<String>,
    pub filter: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapping {
    pub properties: std::collections::HashMap<String, FieldMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FieldMapping {
    #[serde(rename = "text")]
    Text {
        analyzer: Option<String>,
    },
    #[serde(rename = "keyword")]
    Keyword,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "long")]
    Long,
    #[serde(rename = "date")]
    Date,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIndexRequest {
    pub settings: Option<IndexSettings>,
    pub mappings: Option<Mapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: Query,
    pub from: Option<u32>,
    pub size: Option<u32>,
    pub sort: Option<Vec<Sort>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Query {
    Match {
        #[serde(flatten)]
        match_query: std::collections::HashMap<String, MatchQuery>,
    },
    Bool {
        bool: BoolQuery,
    },
    Term {
        #[serde(flatten)]
        term: std::collections::HashMap<String, serde_json::Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchQuery {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoolQuery {
    pub must: Option<Vec<Query>>,
    pub should: Option<Vec<Query>>,
    pub must_not: Option<Vec<Query>>,
    pub filter: Option<Vec<Query>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    #[serde(flatten)]
    pub field: std::collections::HashMap<String, SortOrder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortOrder {
    pub order: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub took: u32,
    pub timed_out: bool,
    pub _shards: ShardsInfo,
    pub hits: Hits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardsInfo {
    pub total: u32,
    pub successful: u32,
    pub skipped: u32,
    pub failed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hits {
    pub total: TotalHits,
    pub max_score: Option<f64>,
    pub hits: Vec<Hit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalHits {
    pub value: u32,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hit {
    pub _index: String,
    pub _type: String,
    pub _id: String,
    pub _score: Option<f64>,
    pub _source: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperation {
    pub action: BulkAction,
    pub document: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum BulkAction {
    #[serde(rename = "index")]
    Index {
        _index: String,
        _id: Option<String>,
    },
    #[serde(rename = "create")]
    Create {
        _index: String,
        _id: Option<String>,
    },
    #[serde(rename = "update")]
    Update {
        _index: String,
        _id: String,
    },
    #[serde(rename = "delete")]
    Delete {
        _index: String,
        _id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkResponse {
    pub took: u32,
    pub errors: bool,
    pub items: Vec<BulkItemResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkItemResponse {
    #[serde(flatten)]
    pub result: std::collections::HashMap<String, BulkOperationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationResult {
    pub _index: String,
    pub _type: String,
    pub _id: String,
    pub _version: Option<u32>,
    pub result: Option<String>,
    pub status: u16,
    pub error: Option<BulkError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkError {
    pub r#type: String,
    pub reason: String,
}
