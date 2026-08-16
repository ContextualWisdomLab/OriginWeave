//! Same-call dispatch gating for tracked sensitive-handle reservations.

use crate::{
    HandleUseDecision, SensitiveDataAuthority, SensitiveHandleUseReservation,
    SensitiveHandleUseState,
};

impl SensitiveHandleUseState {
    /// Recheck one tracked reservation and immediately invoke one disclosure callback.
    ///
    /// `dispatch` is invoked exactly once only when the supplied reservation,
    /// authority, authenticated audience, trusted time, and current revocation state
    /// pass [`Self::recheck_reservation`]. A denial returns the exact
    /// [`HandleUseDecision`] and never invokes the callback.
    ///
    /// This same-call composition narrows the check/use gap but does not prove that
    /// the callback disclosed a protected value and does not settle the reservation.
    /// The trusted broker must keep this call and the later truthful
    /// [`Self::commit_reservation`] or [`Self::compensate_reservation`] inside its
    /// exclusive transaction or locking boundary. Callback return values and
    /// failures must not be treated as automatic disclosure evidence.
    pub fn dispatch_if_reservation_current<R, F>(
        &self,
        reservation: &SensitiveHandleUseReservation,
        authority: SensitiveDataAuthority,
        audience_id: &str,
        now_epoch_seconds: u64,
        dispatch: F,
    ) -> Result<R, HandleUseDecision>
    where
        F: FnOnce() -> R,
    {
        let decision =
            self.recheck_reservation(reservation, authority, audience_id, now_epoch_seconds);
        if decision != HandleUseDecision::Authorized {
            return Err(decision);
        }
        Ok(dispatch())
    }
}
