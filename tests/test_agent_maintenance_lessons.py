"""Keep verified maintenance lessons discoverable without relaxing release gates."""

from pathlib import Path
import unittest


class AgentMaintenanceLessonsTests(unittest.TestCase):
    def test_verification_lessons_remain_actionable(self) -> None:
        text = (Path(__file__).resolve().parents[1] / "AGENTS.md").read_text()
        for instruction in (
            "## Verified maintenance lessons",
            "Resume the existing process",
            "cargo doc --workspace",
            "open the link destinations",
            "original connection can still complete",
        ):
            with self.subTest(instruction=instruction):
                self.assertIn(instruction, text)

    def test_publishing_credentials_do_not_override_release_readiness(self) -> None:
        text = (Path(__file__).resolve().parents[1] / "AGENTS.md").read_text()
        self.assertIn("Secret availability is not release readiness", text)
        self.assertIn("Do not remove `publish = false` merely to make a publish command succeed", text)
        self.assertIn("an explicit version decision", text)


if __name__ == "__main__":
    unittest.main()
