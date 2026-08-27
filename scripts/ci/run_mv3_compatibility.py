                path,
                body=body,
                headers={"Content-Type": "application/json"},
            )
            response = connection.getresponse()
            raw = response.read(MAX_WEBDRIVER_RESPONSE_BYTES + 1)
        except http.client.HTTPException:
            transport_protocol_failed = True
        if transport_protocol_failed:
            raise RuntimeError("WebDriver transport protocol failure")
        if len(raw) > MAX_WEBDRIVER_RESPONSE_BYTES:
            raise RuntimeError("WebDriver response exceeded the bounded JSON limit")
    finally:
        connection.close()

    response_encoding_failed = False
    try:
        decoded_text = raw.decode("utf-8")
    except UnicodeDecodeError:
        response_encoding_failed = True
    if response_encoding_failed:
        raise RuntimeError("WebDriver transport protocol failure")
    try:
        decoded = json.loads(decoded_text)
    except json.JSONDecodeError:
        if response.status >= 400:
            raise RuntimeError(f"WebDriver HTTP {response.status} error") from None
        raise
    if not isinstance(decoded, dict):
        raise RuntimeError("WebDriver returned a non-object JSON payload")