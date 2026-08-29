//! Typed classification for benchmark execution failures.
//!
//! The classification separates known product-threshold failures from evidence
//! that is insufficient to make a product release claim. It never converts an
//! infrastructure, site-drift, or benchmark-harness failure into passing
//! evidence.

use crate::release_acceptance::BenchmarkSuiteOutcome;

/// First-causal-boundary classification for one benchmark failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BenchmarkFailureClass {
    /// A deterministic product contract or required post-condition failed.
    DeterministicContractFailure,
    /// A stochastic product-quality threshold failed with valid benchmark evidence.
    StochasticModelFailure,
    /// The external benchmark site changed outside the reviewed benchmark contract.
    ExternalSiteDrift,
    /// Execution infrastructure failed before valid product evidence was established.
    InfrastructureFailure,
    /// The benchmark harness or oracle is defective or otherwise non-authoritative.
    BenchmarkDefect,
}

impl BenchmarkFailureClass {
    /// Return the stable snake-case identifier retained in benchmark evidence.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DeterministicContractFailure => "deterministic_contract_failure",
            Self::StochasticModelFailure => "stochastic_model_failure",
            Self::ExternalSiteDrift => "external_site_drift",
            Self::InfrastructureFailure => "infrastructure_failure",
            Self::BenchmarkDefect => "benchmark_defect",
        }
    }

    /// Map the failure class to the only release-suite outcome it can establish.
    ///
    /// Known product failures are authoritative failed-suite evidence. Site,
    /// infrastructure, and benchmark defects remain inconclusive because they do
    /// not establish that the product passed or failed the governed threshold.
    #[must_use]
    pub const fn suite_outcome(self) -> BenchmarkSuiteOutcome {
        match self {
            Self::DeterministicContractFailure | Self::StochasticModelFailure => {
                BenchmarkSuiteOutcome::Failed
            }
            Self::ExternalSiteDrift | Self::InfrastructureFailure | Self::BenchmarkDefect => {
                BenchmarkSuiteOutcome::Inconclusive
            }
        }
    }
}
