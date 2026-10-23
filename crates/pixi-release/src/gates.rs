//! Running the configured gate verbs, through the one command surface (`decision-1`).

use std::process::Command;

use pixi_release_core::cut::GateReport;

/// Runs one gate verb and reports whether it passed.
pub trait GateRunner {
    /// Runs `verb`.
    ///
    /// # Errors
    ///
    /// Returns a message when the verb could not be started at all.
    fn run(&self, verb: &str) -> Result<bool, String>;
}

/// The real runner: `pixi run <verb>`, never a second command path.
#[derive(Clone, Copy, Debug, Default)]
pub struct PixiRunner;

impl GateRunner for PixiRunner {
    fn run(&self, verb: &str) -> Result<bool, String> {
        let output = Command::new("pixi")
            .args(["run", verb])
            .output()
            .map_err(|error| format!("gates: pixi is unavailable ({error})"))?;
        Ok(output.status.success())
    }
}

/// Runs every configured verb and collects the report the cut gate reads.
#[must_use]
pub fn report(runner: &dyn GateRunner, verbs: &[String]) -> GateReport {
    let mut passed = Vec::new();
    let mut failed = Vec::new();
    for verb in verbs {
        // A verb that could not start has not passed; recording it as failed is the only honest
        // answer, and `permit` refuses either way.
        if matches!(runner.run(verb), Ok(true)) {
            passed.push(verb.clone());
        } else {
            failed.push(verb.clone());
        }
    }
    GateReport::new(passed, failed)
}

#[cfg(test)]
mod tests {
    use super::{GateRunner, report};

    struct FakeRunner {
        failing: Vec<String>,
    }

    impl GateRunner for FakeRunner {
        fn run(&self, verb: &str) -> Result<bool, String> {
            Ok(!self.failing.iter().any(|failed| failed == verb))
        }
    }

    #[test]
    fn a_failing_verb_is_reported_as_failed() {
        let runner = FakeRunner {
            failing: vec!["gates".to_owned()],
        };

        let report = report(&runner, &["gates".to_owned(), "build".to_owned()]);

        assert_eq!(report.passed(), ["build"]);
        assert_eq!(report.failed(), ["gates"]);
    }

    #[test]
    fn a_verb_that_cannot_start_has_not_passed() {
        struct Broken;
        impl GateRunner for Broken {
            fn run(&self, _verb: &str) -> Result<bool, String> {
                Err("pixi missing".to_owned())
            }
        }

        let report = report(&Broken, &["gates".to_owned()]);

        assert!(report.passed().is_empty());
        assert_eq!(report.failed(), ["gates"]);
    }
}
