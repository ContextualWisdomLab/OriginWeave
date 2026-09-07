use originweave_core::{BrowserAuthorityRegistry, BrowserRegistryError, BrowserSessionId};

#[test]
fn session_mapping_is_read_only_exact_and_revoked_on_retirement() -> Result<(), BrowserRegistryError>
{
    let mut registry = BrowserAuthorityRegistry::new();
    let first = registry.register_session("session-a")?;
    let second = registry.register_session("session-b")?;
    registry.require_registered_session_external_identifier(first, "session-a")?;
    for (session, external) in [
        (first, "session-b"),
        (second, "session-a"),
        (first, "unknown"),
        (
            BrowserSessionId::new(99)
                .map_err(|_| BrowserRegistryError::InternalAuthorityInvariant)?,
            "session-a",
        ),
    ] {
        assert_eq!(
            registry.require_registered_session_external_identifier(session, external),
            Err(BrowserRegistryError::SessionExternalIdentifierMismatch)
        );
    }
    assert_eq!(
        registry.require_registered_session_external_identifier(first, "\n"),
        Err(BrowserRegistryError::InvalidExternalIdentifier)
    );
    registry.remove_session(first)?;
    assert_eq!(
        registry.require_registered_session_external_identifier(first, "session-a"),
        Err(BrowserRegistryError::SessionExternalIdentifierMismatch)
    );
    let replacement = registry.register_session("session-a")?;
    assert_ne!(first, replacement);
    registry.require_registered_session_external_identifier(replacement, "session-a")?;
    assert_eq!(
        registry.require_registered_session_external_identifier(first, "session-a"),
        Err(BrowserRegistryError::SessionExternalIdentifierMismatch)
    );
    assert_eq!(
        BrowserRegistryError::SessionExternalIdentifierMismatch.to_string(),
        "browser session external identifier does not match the registered session"
    );
    Ok(())
}
