from pathlib import Path
import unittest


SOURCE = Path("crates/originweave-network/src/webdriver_bidi_websocket_handshake.rs")


class WebDriverBiDiOpeningWriteTimeoutCleanupContract(unittest.TestCase):
    def test_successful_opening_write_clears_operation_local_socket_timeout(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        self.assertIn("fn clear_write_timeout(&self) -> io::Result<()>;", source)
        self.assertIn("TcpStream::set_write_timeout(self, None)", source)

        helper_start = source.index("fn write_request_with_clock(")
        helper_end = source.index("\n#[cfg(test)]", helper_start)
        helper = source[helper_start:helper_end]
        clear_call = helper.rfind("writer.clear_write_timeout()")
        success = helper.rfind("Ok(bytes_written)")

        self.assertGreater(clear_call, -1)
        self.assertGreater(success, clear_call)


if __name__ == "__main__":
    unittest.main()
