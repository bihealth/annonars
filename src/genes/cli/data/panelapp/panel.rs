/*
 * PanelApp API
 *
 * PanelApp API
 *
 * OpenAPI spec version: v1
 * Contact: panelapp@genomicsengland.co.uk
 * Generated by: https://github.com/swagger-api/swagger-codegen.git
 */


#[allow(unused_imports)]
use serde_json::Value;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Panel {
  #[serde(rename = "id")]
  id: i32,
  #[serde(rename = "hash_id")]
  hash_id: Option<String>,
  #[serde(rename = "name")]
  name: Option<String>,
  #[serde(rename = "disease_group")]
  disease_group: Option<String>,
  #[serde(rename = "disease_sub_group")]
  disease_sub_group: Option<String>,
  #[serde(rename = "status")]
  status: Option<String>,
  #[serde(rename = "version")]
  version: Option<String>,
  #[serde(rename = "version_created")]
  version_created: Option<String>,
  #[serde(rename = "relevant_disorders")]
  relevant_disorders: Vec<String>,
  /// Object with panel statistics (number of genes or STRs)
  #[serde(rename = "stats")]
  stats: Option<Value>,
  #[serde(rename = "types")]
  types: Option<Vec<crate::genes::cli::data::panelapp::PanelType>>
}

impl Panel {
  pub fn new(id: i32, relevant_disorders: Vec<String>) -> Panel {
    Panel {
      id: id,
      hash_id: None,
      name: None,
      disease_group: None,
      disease_sub_group: None,
      status: None,
      version: None,
      version_created: None,
      relevant_disorders: relevant_disorders,
      stats: None,
      types: None
    }
  }

  pub fn set_id(&mut self, id: i32) {
    self.id = id;
  }

  pub fn with_id(mut self, id: i32) -> Panel {
    self.id = id;
    self
  }

  pub fn id(&self) -> &i32 {
    &self.id
  }


  pub fn set_hash_id(&mut self, hash_id: String) {
    self.hash_id = Some(hash_id);
  }

  pub fn with_hash_id(mut self, hash_id: String) -> Panel {
    self.hash_id = Some(hash_id);
    self
  }

  pub fn hash_id(&self) -> Option<&String> {
    self.hash_id.as_ref()
  }

  pub fn reset_hash_id(&mut self) {
    self.hash_id = None;
  }

  pub fn set_name(&mut self, name: String) {
    self.name = Some(name);
  }

  pub fn with_name(mut self, name: String) -> Panel {
    self.name = Some(name);
    self
  }

  pub fn name(&self) -> Option<&String> {
    self.name.as_ref()
  }

  pub fn reset_name(&mut self) {
    self.name = None;
  }

  pub fn set_disease_group(&mut self, disease_group: String) {
    self.disease_group = Some(disease_group);
  }

  pub fn with_disease_group(mut self, disease_group: String) -> Panel {
    self.disease_group = Some(disease_group);
    self
  }

  pub fn disease_group(&self) -> Option<&String> {
    self.disease_group.as_ref()
  }

  pub fn reset_disease_group(&mut self) {
    self.disease_group = None;
  }

  pub fn set_disease_sub_group(&mut self, disease_sub_group: String) {
    self.disease_sub_group = Some(disease_sub_group);
  }

  pub fn with_disease_sub_group(mut self, disease_sub_group: String) -> Panel {
    self.disease_sub_group = Some(disease_sub_group);
    self
  }

  pub fn disease_sub_group(&self) -> Option<&String> {
    self.disease_sub_group.as_ref()
  }

  pub fn reset_disease_sub_group(&mut self) {
    self.disease_sub_group = None;
  }

  pub fn set_status(&mut self, status: String) {
    self.status = Some(status);
  }

  pub fn with_status(mut self, status: String) -> Panel {
    self.status = Some(status);
    self
  }

  pub fn status(&self) -> Option<&String> {
    self.status.as_ref()
  }

  pub fn reset_status(&mut self) {
    self.status = None;
  }

  pub fn set_version(&mut self, version: String) {
    self.version = Some(version);
  }

  pub fn with_version(mut self, version: String) -> Panel {
    self.version = Some(version);
    self
  }

  pub fn version(&self) -> Option<&String> {
    self.version.as_ref()
  }

  pub fn reset_version(&mut self) {
    self.version = None;
  }

  pub fn set_version_created(&mut self, version_created: String) {
    self.version_created = Some(version_created);
  }

  pub fn with_version_created(mut self, version_created: String) -> Panel {
    self.version_created = Some(version_created);
    self
  }

  pub fn version_created(&self) -> Option<&String> {
    self.version_created.as_ref()
  }

  pub fn reset_version_created(&mut self) {
    self.version_created = None;
  }

  pub fn set_relevant_disorders(&mut self, relevant_disorders: Vec<String>) {
    self.relevant_disorders = relevant_disorders;
  }

  pub fn with_relevant_disorders(mut self, relevant_disorders: Vec<String>) -> Panel {
    self.relevant_disorders = relevant_disorders;
    self
  }

  pub fn relevant_disorders(&self) -> &Vec<String> {
    &self.relevant_disorders
  }


  pub fn set_stats(&mut self, stats: Value) {
    self.stats = Some(stats);
  }

  pub fn with_stats(mut self, stats: Value) -> Panel {
    self.stats = Some(stats);
    self
  }

  pub fn stats(&self) -> Option<&Value> {
    self.stats.as_ref()
  }

  pub fn reset_stats(&mut self) {
    self.stats = None;
  }

  pub fn set_types(&mut self, types: Vec<crate::genes::cli::data::panelapp::PanelType>) {
    self.types = Some(types);
  }

  pub fn with_types(mut self, types: Vec<crate::genes::cli::data::panelapp::PanelType>) -> Panel {
    self.types = Some(types);
    self
  }

  pub fn types(&self) -> Option<&Vec<crate::genes::cli::data::panelapp::PanelType>> {
    self.types.as_ref()
  }

  pub fn reset_types(&mut self) {
    self.types = None;
  }

}



