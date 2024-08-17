use serde::{Serialize, Deserialize};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct SubTask {
  pub text: String,
  pub completed: bool
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct Task {
  #[serde(rename = "_id")]
  pub id: String,
  pub text: String,
  #[serde(rename = "type")]
  pub task_type: String,
  pub priority: f32,
  pub notes: Option<String>,
  #[serde(
    deserialize_with = "time::serde::iso8601::option::deserialize", 
    serialize_with = "time::serde::iso8601::option::serialize"
  )]
  pub date: Option<OffsetDateTime>,
  pub checklist: Option<Vec<SubTask>>,
}
