from pathlib import Path
import unittest


ROOT = Path(__file__).parents[1]


class CaptureRetentionLifecycleDocumentationContractTests(unittest.TestCase):
    def test_lifecycle_boundary_is_documented_without_overclaiming_persistence(self):
        architecture = (ROOT / "ARCHITECTURE.md").read_text()
        changelog = (ROOT / "CHANGELOG.md").read_text()

        self.assertIn("CaptureLifecycle", architecture)
        self.assertIn("retention deadline", architecture)
        self.assertIn("does not persist or delete artifacts", architecture)
        self.assertIn("PR #239", changelog)
        self.assertIn("CaptureLifecycle", changelog)


if __name__ == "__main__":
    unittest.main()
