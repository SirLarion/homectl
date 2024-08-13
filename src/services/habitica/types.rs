use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct SubTask {
  pub text: String,
  pub completed: bool
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Task {
  pub text: String,
  #[serde(rename = "type")]
  pub task_type: String,
  pub priority: f32,
  pub notes: Option<String>,
  pub date: Option<String>,
  pub checklist: Option<Vec<SubTask>>,
}
