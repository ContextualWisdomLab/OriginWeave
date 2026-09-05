use std::{error::Error, fmt};

use crate::{WebDriverBiDiSessionEndResult, WebDriverBiDiWebSocketTransportClosureObservation};

/// Fail-closed operational disposition derived from teardown observations.
///
/// The current assessment can report only `OperationalTeardownPending`. Transport closure already
/// has a typed owner, while browser-process exit and task-profile removal are not represented until
/// their own typed runtime owners are connected. Missing owner evidence cannot establish operational
/// completion or grant process, profile, browser, network, policy, secret, or Agent authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebDriverBiDiSessionTeardownDisposition {
    /// Operational teardown is not yet proven by typed observations from every owning boundary.
    OperationalTeardownPending,
}

/// Explicit operational observations retained after a correlated WebDriver BiDi `session.end` ack.
///
/// Transport closure is represented by the typed observation produced only by consuming the exact
/// established WebSocket at its bounded closure-observation boundary. Browser-process exit and task
/// profile removal are deliberately absent until their own typed runtime owners can supply
/// non-forgeable evidence.
#[derive(Debug, Eq, PartialEq)]
pub struct WebDriverBiDiSessionTeardownObservations {
    transport_closure_observation: Option<WebDriverBiDiWebSocketTransportClosureObservation>,
}

impl WebDriverBiDiSessionTeardownObservations {
    /// Construct the typed transport observation retained by this boundary.
    ///
    /// Transport closure cannot be asserted with a caller-supplied boolean. `Some` requires the
    /// typed closure observation returned by the bounded transport owner; `None` keeps teardown
    /// pending. Process and profile state cannot be supplied as placeholders and remain unproven
    /// until their typed runtime owners are connected.
    #[must_use]
    pub const fn new(
        transport_closure_observation: Option<WebDriverBiDiWebSocketTransportClosureObservation>,
    ) -> Self {
        Self {
            transport_closure_observation,
        }
    }

    /// Return whether typed closure evidence for the exact acknowledged transport was supplied.
    ///
    /// A teardown assessment is constructed only after connection-generation equality has been
    /// checked, so a retained observation belongs to the same exact connection as its protocol ack.
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
}

/// Fail-closed failures while binding protocol acknowledgment to operational teardown evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebDriverBiDiSessionTeardownAssessmentError {
    /// The closure observation belongs to a different process-local connection generation.
    TransportConnectionMismatch,
}

impl fmt::Display for WebDriverBiDiSessionTeardownAssessmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TransportConnectionMismatch => formatter.write_str(
                "WebDriver BiDi transport closure does not match the acknowledged connection",
            ),
        }
    }
}

impl Error for WebDriverBiDiSessionTeardownAssessmentError {}

/// One correlated `session.end` acknowledgment kept separate from operational teardown evidence.
///
/// A protocol acknowledgment alone is never operational completion. Typed transport closure is
/// retained only when its private process-local connection generation matches the generation bound
/// to the acknowledged `session.end` command before I/O. Process/profile evidence remains absent
/// until its owning runtime boundaries can provide typed observations. Consequently this assessment
/// remains fail closed in the current dependency-ordered slice.
#[derive(Debug, Eq, PartialEq)]
pub struct WebDriverBiDiSessionTeardownAssessment {
    protocol_ack: WebDriverBiDiSessionEndResult,
    observations: WebDriverBiDiSessionTeardownObservations,
}

impl WebDriverBiDiSessionTeardownAssessment {
    /// Bind one correlated protocol acknowledgment to separately supplied operational observations.
    ///
    /// If typed transport closure is present, its private connection generation must equal the one
    /// retained by the protocol acknowledgment. A closure from another socket is rejected even when
    /// both transports reuse the same WebDriver session identifier and command id.
    pub fn from_protocol_ack(
        protocol_ack: WebDriverBiDiSessionEndResult,
        observations: WebDriverBiDiSessionTeardownObservations,
    ) -> Result<Self, WebDriverBiDiSessionTeardownAssessmentError> {
        if let Some(transport_closure) = observations.transport_closure_observation() {
            if transport_closure.connection_generation() != protocol_ack.connection_generation() {
                return Err(
                    WebDriverBiDiSessionTeardownAssessmentError::TransportConnectionMismatch,
                );
            }
        }
        Ok(Self {
            protocol_ack,
            observations,
        })
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

    /// Return whether typed evidence proves every required operational teardown boundary.
    ///
    /// This is always `false` while typed browser-process-exit and task-profile-removal evidence is
    /// absent from this boundary.
    #[must_use]
    pub const fn is_operationally_complete(&self) -> bool {
        false
    }

    /// Return the fail-closed disposition for the currently supplied observations.
    ///
    /// The current boundary cannot emit an operational-completion disposition because process and
    /// profile cleanup still lack typed owner evidence.
    #[must_use]
    pub const fn disposition(&self) -> WebDriverBiDiSessionTeardownDisposition {
        WebDriverBiDiSessionTeardownDisposition::OperationalTeardownPending
    }
}
