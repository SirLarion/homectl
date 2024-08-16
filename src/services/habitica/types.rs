use serde::{Serialize, Deserialize};

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
  pub date: Option<String>,
  pub checklist: Option<Vec<SubTask>>,
}
