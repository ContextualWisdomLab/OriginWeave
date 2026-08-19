def _cleanup_crashed_browser_session(driver_port: int, session_id: str | None) -> None:
    """Delete a crash session while ignoring only reviewed post-crash transport loss."""

    if session_id is None:
        return
    try:
        _json_request(
            driver_port,
            "DELETE",
            _webdriver_path(session_id, ""),
            {},
        )
    except (
        OSError,
        RuntimeError,
        json.JSONDecodeError,
        http.client.IncompleteRead,
    ):
        return
