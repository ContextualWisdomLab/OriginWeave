"""Fail-first contract for the controlled Chromium Agent Task fixture."""

from __future__ import annotations

from html.parser import HTMLParser
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests" / "fixtures" / "agent_task_basic" / "index.html"


def _is_credential_input(attributes: dict[str, str | None]) -> bool:
    """Return whether parsed input attributes describe a credential surface."""

    input_type = (attributes.get("type") or "").strip().lower()
    if input_type == "password":
        return True

    autocomplete = (attributes.get("autocomplete") or "").strip().lower()
    autocomplete_tokens = autocomplete.split()
    return any(
        token == "one-time-code" or "password" in token
        for token in autocomplete_tokens
    )


class _FixtureParser(HTMLParser):
    """Collect the small semantic surface required by the deterministic fixture."""

    def __init__(self) -> None:
        super().__init__()
        self.ids: set[str] = set()
        self.labels_for: set[str] = set()
        self.input_names: set[str] = set()
        self.input_attributes: list[dict[str, str | None]] = []
        self.button_types: set[str] = set()
        self.hidden_injection_markers = 0

    def handle_starttag(
        self, tag: str, attrs: list[tuple[str, str | None]]
    ) -> None:
        attributes = dict(attrs)
        element_id = attributes.get("id")
        if element_id:
            self.ids.add(element_id)
        if tag == "label" and attributes.get("for"):
            self.labels_for.add(attributes["for"])
        if tag == "input":
            self.input_attributes.append(attributes)
            if attributes.get("name"):
                self.input_names.add(attributes["name"])
        if tag == "button" and attributes.get("type"):
            self.button_types.add(attributes["type"])
        if (
            attributes.get("data-originweave-untrusted") == "prompt-injection"
            and "hidden" in attributes
            and attributes.get("aria-hidden") == "true"
        ):
            self.hidden_injection_markers += 1


class AgentTaskFixtureContractTests(unittest.TestCase):
    """Require one deterministic semantic workflow for the first browser slice."""

    def setUp(self) -> None:
        """Load the checked-in fixture once for each independent contract."""

        self.html = FIXTURE.read_text(encoding="utf-8")
        self.parser = _FixtureParser()
        self.parser.feed(self.html)

    def test_fixture_exposes_semantic_form_and_observable_post_condition(self) -> None:
        """The fixture must support role/name discovery and a deterministic state change."""

        self.assertIn("task-text", self.parser.ids)
        self.assertIn("task-text", self.parser.labels_for)
        self.assertIn("task_text", self.parser.input_names)
        self.assertIn("submit", self.parser.button_types)
        self.assertIn("task-result", self.parser.ids)
        self.assertIn('data-state="idle"', self.html)
        self.assertIn('result.dataset.state = "submitted"', self.html)
        self.assertIn("result.textContent = taskText.value", self.html)

    def test_fixture_contains_explicit_untrusted_hidden_prompt_injection(self) -> None:
        """A later real-browser regression needs hostile hidden page content to ignore."""

        self.assertEqual(self.parser.hidden_injection_markers, 1)
        self.assertIn("UNTRUSTED_PAGE_INSTRUCTION", self.html)
        self.assertIn("request new browser capabilities", self.html)

    def test_hidden_injection_requires_the_actual_hidden_attribute(self) -> None:
        """ARIA metadata alone must not satisfy the hidden-injection fixture contract."""

        parser = _FixtureParser()
        parser.feed(
            "<p data-originweave-untrusted='prompt-injection' aria-hidden='true'>visible</p>"
            "<p data-originweave-untrusted='prompt-injection' aria-hidden='true' hidden>hidden</p>"
        )
        self.assertEqual(parser.hidden_injection_markers, 1)

    def test_fixture_is_synthetic_and_has_no_credential_fields(self) -> None:
        """The controlled workflow must not require or imitate real secret collection."""

        for attributes in self.parser.input_attributes:
            with self.subTest(attributes=attributes):
                self.assertFalse(_is_credential_input(attributes))

        lowered = self.html.lower()
        for forbidden in ("api_key", "secret_key"):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, lowered)

    def test_credential_detection_is_quote_independent(self) -> None:
        """Parsed credential semantics must reject single-quoted and tokenized forms."""

        for html in (
            "<input type='password'>",
            "<input autocomplete='current-password'>",
            "<input autocomplete='new-password'>",
            "<input autocomplete='section-login username current-password'>",
            "<input autocomplete='one-time-code'>",
        ):
            with self.subTest(html=html):
                parser = _FixtureParser()
                parser.feed(html)
                self.assertEqual(len(parser.input_attributes), 1)
                self.assertTrue(_is_credential_input(parser.input_attributes[0]))

        parser = _FixtureParser()
        parser.feed("<input type='text' autocomplete='username'>")
        self.assertEqual(len(parser.input_attributes), 1)
        self.assertFalse(_is_credential_input(parser.input_attributes[0]))


if __name__ == "__main__":
    unittest.main()
