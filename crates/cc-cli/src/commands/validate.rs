use crate::output;
use anyhow::Result;
use std::path::Path;

pub fn run(project_dir: &Path, fix: bool) -> Result<()> {
    println!("Validating .claude/ resources...\n");

    let findings = cc_core::validate_project(project_dir);

    if findings.is_empty() {
        output::print_success("All resources are valid.");
        return Ok(());
    }

    output::print_findings(&findings);

    let errors = findings
        .iter()
        .filter(|f| f.level == cc_core::ValidationLevel::Error)
        .count();
    let warnings = findings
        .iter()
        .filter(|f| f.level == cc_core::ValidationLevel::Warning)
        .count();

    println!();
    println!("Result: {} error(s), {} warning(s)", errors, warnings);

    if fix && warnings > 0 {
        output::print_info("Run with --fix to auto-fix warnings (not yet implemented).");
    }

    Ok(())
}
