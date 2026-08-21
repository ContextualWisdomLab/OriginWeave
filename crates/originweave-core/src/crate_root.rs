//! OriginWeave core contracts with narrow public error integration.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::fmt;

#[path = "lib.rs"]
mod base;
mod native_messaging_frame;
pub use base::*;
pub use native_messaging_frame::*;

impl fmt::Display for NativeMessagingHostNameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHostName => formatter.write_str(
                "native-messaging host name violates the reviewed Chrome identity syntax",
            ),
        }
    }
}

impl std::error::Error for NativeMessagingHostNameError {}
