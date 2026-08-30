use crate::WebDriverBiDiSessionEndResult;

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
/// These booleans are deliberately observation facts, not authority or evidence provenance. The
/// trusted browser/process/profile owner remains responsible for producing and authenticating the
/// underlying observations before constructing this value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WebDriverBiDiSessionTeardownObservations {
    transport_closed_observed: bool,
    browser_process_exited_observed: bool,
    task_profile_removed_observed: bool,
}

impl WebDriverBiDiSessionTeardownObservations {
    /// Construct the three explicit operational observations required by this boundary.
    #[must_use]
    pub const fn new(
        transport_closed_observed: bool,
        browser_process_exited_observed: bool,
        task_profile_removed_observed: bool,
    ) -> Self {
        Self {
            transport_closed_observed,
            browser_process_exited_observed,
            task_profile_removed_observed,
        }
    }

    /// Return whether closure of the exact session transport was observed.
    #[must_use]
    pub const fn transport_closed_observed(&self) -> bool {
        self.transport_closed_observed
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
        self.transport_closed_observed
            & self.browser_process_exited_observed
            & self.task_profile_removed_observed
    }
}

/// One correlated `session.end` acknowledgment kept separate from operational teardown evidence.
///
/// A protocol acknowledgment alone is never operational completion. Callers must separately
/// provide all reviewed transport/process/profile observations, and the observations themselves
/// remain non-authoritative until authenticated by their owning runtime boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
