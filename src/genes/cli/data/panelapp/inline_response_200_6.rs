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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InlineResponse2006 {
    #[serde(rename = "count")]
    count: i32,
    #[serde(rename = "next")]
    next: Option<String>,
    #[serde(rename = "previous")]
    previous: Option<String>,
    #[serde(rename = "results")]
    results: Vec<crate::genes::cli::data::panelapp::Region>,
}

impl InlineResponse2006 {
    pub fn new(
        count: i32,
        results: Vec<crate::genes::cli::data::panelapp::Region>,
    ) -> InlineResponse2006 {
        InlineResponse2006 {
            count,
            next: None,
            previous: None,
            results,
        }
    }

    pub fn set_count(&mut self, count: i32) {
        self.count = count;
    }

    pub fn with_count(mut self, count: i32) -> InlineResponse2006 {
        self.count = count;
        self
    }

    pub fn count(&self) -> &i32 {
        &self.count
    }

    pub fn set_next(&mut self, next: String) {
        self.next = Some(next);
    }

    pub fn with_next(mut self, next: String) -> InlineResponse2006 {
        self.next = Some(next);
        self
    }

    pub fn next(&self) -> Option<&String> {
        self.next.as_ref()
    }

    pub fn reset_next(&mut self) {
        self.next = None;
    }

    pub fn set_previous(&mut self, previous: String) {
        self.previous = Some(previous);
    }

    pub fn with_previous(mut self, previous: String) -> InlineResponse2006 {
        self.previous = Some(previous);
        self
    }

    pub fn previous(&self) -> Option<&String> {
        self.previous.as_ref()
    }

    pub fn reset_previous(&mut self) {
        self.previous = None;
    }

    pub fn set_results(&mut self, results: Vec<crate::genes::cli::data::panelapp::Region>) {
        self.results = results;
    }

    pub fn with_results(
        mut self,
        results: Vec<crate::genes::cli::data::panelapp::Region>,
    ) -> InlineResponse2006 {
        self.results = results;
        self
    }

    pub fn results(&self) -> &Vec<crate::genes::cli::data::panelapp::Region> {
        &self.results
    }
}
