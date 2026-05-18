pub mod audit;
pub mod bench;
pub mod binary;
pub mod bloat;
pub mod build;
pub mod clippy;
pub mod coverage;
pub mod deny;
pub mod deps;
pub mod doc;
pub mod flamegraph;
pub mod fmt;
pub mod geiger;
pub mod metrics;
pub mod msrv;
pub mod semver_checks;
pub mod test;
pub mod udeps;

#[derive(Debug, Clone, PartialEq)]
pub enum ReportStatus {
    Ok,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub struct ReportSection {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ToolReport {
    pub status: ReportStatus,
    pub sections: Vec<ReportSection>,
}

pub trait ToolParser {
    fn parse(&self, stdout: &str, stderr: &str, exit_code: i32) -> ToolReport;
}
