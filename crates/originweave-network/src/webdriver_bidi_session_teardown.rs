use crate::{
    WebDriverBiDiSessionEndResult, WebDriverBiDiWebSocketTransportClosureObservation,
};

/// Fail-closed operational disposition derived from explicit teardown observations.
///
/// This value does not authenticate any observation or grant process, profile, browser, network,
/// policy, or Agent authority. `OperationallyComplete` means only that the caller supplied all
/// reviewed observation classes after a correlated `session.end` protocol acknowledgment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebDriverBiDiSessionTeardownDisposition {
    /// One or more required operational teardown observations remain absent.
    OperationalTeardownPending,
    /// Every required operational teardown observation was supplied.
    OperationallyComplete,
}

/// Explicit operational observations required after a correlated WebDriver BiDi `session.end` ack.
///
/// Transport closure is represented by the typed observation produced only by consuming the exact
/// established WebSocket at its bounded closure-observation boundary. Browser-process exit and task
/// profile removal remain caller-supplied observation facts until their own typed runtime owners are
/// connected. None of these fields grants authority or authenticates the owning runtime boundary.
#[derive(Debug, Eq, PartialEq)]
pub struct WebDriverBiDiSessionTeardownObservations {
    transport_closure_observation: Option<WebDriverBiDiWebSocketTransportClosureObservation>,
    browser_process_exited_observed: bool,
    task_profile_removed_observed: bool,
}

impl WebDriverBiDiSessionTeardownObservations {
    /// Construct the three explicit operational observations required by this boundary.
    ///
    /// Transport closure cannot be asserted with a caller-supplied boolean. `Some` requires the
    /// typed closure observation returned by the bounded transport owner; `None` keeps teardown
    /// pending. The remaining booleans are observation placeholders for later typed runtime owners.
    #[must_use]
    pub const fn new(
        transport_closure_observation: Option<WebDriverBiDiWebSocketTransportClosureObservation>,
        browser_process_exited_observed: bool,
        task_profile_removed_observed: bool,
    ) -> Self {
        Self {
            transport_closure_observation,
            browser_process_exited_observed,
            task_profile_removed_observed,
        }
    }

    /// Return whether typed closure evidence for the exact session transport was supplied.
    #[must_use]
    pub const fn transport_closed_observed(&self) -> bool {
        self.transport_closure_observation.is_some()
    }

    /// Borrow the typed transport-closure observation when one was supplied.
    #[must_use]
    pub const fn transport_closure_observation(
        &self,
    ) -> Option<&WebDriverBiDiWebSocketTransportClosureObservation> {
        self.transport_closure_observation.as_ref()
    }

    /// Return whether exit of the owned browser process was observed.
    #[must_use]
    pub const fn browser_process_exited_observed(&self) -> bool {
        self.browser_process_exited_observed
    }

    /// Return whether removal of the owned task profile was observed.
    #[must_use]
    pub const fn task_profile_removed_observed(&self) -> bool {
        self.task_profile_removed_observed
    }

    const fn operationally_complete(&self) -> bool {
        self.transport_closure_observation.is_some()
            & self.browser_process_exited_observed
            & self.task_profile_removed_observed
    }
}

/// One correlated `session.end` acknowledgment kept separate from operational teardown evidence.
///
/// A protocol acknowledgment alone is never operational completion. Callers must separately
/// provide all reviewed transport/process/profile observations. Transport closure is retained as a
/// typed observation from the consumed WebSocket, while process/profile observations remain
/// non-authoritative until authenticated by their owning runtime boundaries.
#[derive(Debug, Eq, PartialEq)]
pub struct WebDriverBiDiSessionTeardownAssessment {
    protocol_ack: WebDriverBiDiSessionEndResult,
    observations: WebDriverBiDiSessionTeardownObservations,
}

impl WebDriverBiDiSessionTeardownAssessment {
    /// Bind one correlated protocol acknowledgment to separately supplied operational observations.
    #[must_use]
    pub const fn from_protocol_ack(
        protocol_ack: WebDriverBiDiSessionEndResult,
        observations: WebDriverBiDiSessionTeardownObservations,
    ) -> Self {
        Self {
            protocol_ack,
            observations,
        }
    }

    /// Return the exact command id proven by the correlated protocol acknowledgment.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.protocol_ack.command_id()
    }

    /// Borrow the explicit operational observations bound to this assessment.
    #[must_use]
    pub const fn observations(&self) -> &WebDriverBiDiSessionTeardownObservations {
        &self.observations
    }

    /// Return whether all required operational teardown observations are present.
    #[must_use]
    pub const fn is_operationally_complete(&self) -> bool {
        self.observations.operationally_complete()
    }

    /// Return the fail-closed disposition for the currently supplied observations.
    #[must_use]
    pub const fn disposition(&self) -> WebDriverBiDiSessionTeardownDisposition {
        if self.is_operationally_complete() {
            WebDriverBiDiSessionTeardownDisposition::OperationallyComplete
        } else {
            WebDriverBiDiSessionTeardownDisposition::OperationalTeardownPending
        }
    }
}
