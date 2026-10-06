//! Deterministic RFC 9309 robots-exclusion parsing and matching.
//!
//! This module turns an already fetched `robots.txt` body into a
//! [`RobotsDecision`] for one crawler product token and one request path. It
//! performs no I/O, reads no clock, and never panics. Every stored structure is
//! bounded by the accepted body prefix ([`MAX_ROBOTS_BODY_BYTES`]) and every
//! request path is bounded by [`MAX_ROBOTS_PATH_BYTES`] after normalization.
//!
//! Robots rules are crawler etiquette, not access authorization: an `Allowed`
//! result only feeds the existing policy evaluator and grants no capability.
//! Evaluation evidence ([`RobotsEvaluation`]) is value-free: it records the
//! decision, the rule kind and the 1-based line number, never the pattern or
//! the request path.

use std::cmp::Reverse;
use std::fmt;

use originweave_core::RobotsDecision;

/// Maximum number of body bytes parsed (RFC 9309 section 2.5 requires at least 500 KiB).
pub const MAX_ROBOTS_BODY_BYTES: usize = 512_000;

/// Maximum length, in bytes after percent-encoding normalization, of a request path.
pub const MAX_ROBOTS_PATH_BYTES: usize = 8_192;

/// Maximum number of accepted `allow`/`disallow` rules across a whole body.
pub const MAX_ROBOTS_RULES: usize = 10_000;

/// Maximum length, in bytes, of a crawler product token.
pub const MAX_ROBOTS_PRODUCT_TOKEN_BYTES: usize = 128;

/// Raw request-path bytes that are normalized before the length limit is decided.
///
/// Normalization emits at least one byte for every three input bytes, so any
/// input longer than this bound normalizes to more than [`MAX_ROBOTS_PATH_BYTES`]
/// bytes. Bounding the input first keeps the allocation independent of the
/// caller's path length.
const NORMALIZATION_INPUT_BYTES: usize = MAX_ROBOTS_PATH_BYTES * 3 + 1;

/// The UTF-8 byte-order mark that may prefix a body.
const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

/// Characters trimmed around record keys and values.
const RECORD_WHITESPACE: [char; 2] = [' ', '\t'];

/// The path that RFC 9309 section 2.2.2 implicitly allows.
const ROBOTS_TXT_PATH: &str = "/robots.txt";

/// A validated crawler product token (RFC 9309 section 2.2.1).
///
/// A token is 1 to [`MAX_ROBOTS_PRODUCT_TOKEN_BYTES`] bytes of ASCII letters,
/// `_` and `-`. The wildcard `*` is not a product token: a crawler must name
/// itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobotsProductToken(String);

impl RobotsProductToken {
    /// Validate a crawler product token.
    ///
    /// # Errors
    ///
    /// Returns [`RobotsProductTokenError::Empty`] for an empty value,
    /// [`RobotsProductTokenError::TooLong`] for a value longer than
    /// [`MAX_ROBOTS_PRODUCT_TOKEN_BYTES`], and
    /// [`RobotsProductTokenError::InvalidCharacter`] for any other character.
    pub fn parse(value: &str) -> Result<Self, RobotsProductTokenError> {
        if value.is_empty() {
            return Err(RobotsProductTokenError::Empty);
        }
        if value.len() > MAX_ROBOTS_PRODUCT_TOKEN_BYTES {
            return Err(RobotsProductTokenError::TooLong);
        }
        if !value.bytes().all(is_product_token_byte) {
            return Err(RobotsProductTokenError::InvalidCharacter);
        }
        Ok(Self(value.to_owned()))
    }

    /// Return the token exactly as supplied.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A reason a crawler product token was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RobotsProductTokenError {
    /// The token was empty.
    Empty,
    /// The token exceeded [`MAX_ROBOTS_PRODUCT_TOKEN_BYTES`].
    TooLong,
    /// The token contained a character other than an ASCII letter, `_` or `-`.
    InvalidCharacter,
}

impl fmt::Display for RobotsProductTokenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "robots product token is empty",
            Self::TooLong => "robots product token exceeds the length limit",
            Self::InvalidCharacter => "robots product token may contain only letters, '_' and '-'",
        })
    }
}

impl std::error::Error for RobotsProductTokenError {}

/// A reason a request path was rejected before matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RobotsPathError {
    /// The path did not start with `/` (for example a relative or absolute URI).
    MissingLeadingSlash,
    /// The path contained a C0, DEL or C1 control character.
    ControlCharacter,
    /// The path contained a `#` fragment delimiter.
    Fragment,
    /// The normalized path exceeded [`MAX_ROBOTS_PATH_BYTES`].
    TooLong,
}

impl fmt::Display for RobotsPathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingLeadingSlash => "robots request path must start with '/'",
            Self::ControlCharacter => "robots request path contains a control character",
            Self::Fragment => "robots request path must not contain a fragment",
            Self::TooLong => "robots request path exceeds the normalized length limit",
        })
    }
}

impl std::error::Error for RobotsPathError {}

/// The kind of a robots rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RobotsRuleKind {
    /// An `allow` rule.
    Allow,
    /// A `disallow` rule.
    Disallow,
}

impl RobotsRuleKind {
    /// Rank used to break specificity ties: `allow` wins (RFC 9309 section 2.2.2).
    const fn tie_rank(self) -> u8 {
        match self {
            Self::Allow => 1,
            Self::Disallow => 0,
        }
    }

    /// The decision produced when a rule of this kind is the most specific match.
    const fn decision(self) -> RobotsDecision {
        match self {
            Self::Allow => RobotsDecision::Allowed,
            Self::Disallow => RobotsDecision::Disallowed,
        }
    }
}

/// Value-free evidence identifying the rule that decided an evaluation.
///
/// It deliberately carries no pattern text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RobotsMatchedRule {
    kind: RobotsRuleKind,
    line_number: usize,
}

impl RobotsMatchedRule {
    /// Return whether the deciding rule was `allow` or `disallow`.
    #[must_use]
    pub const fn kind(&self) -> RobotsRuleKind {
        self.kind
    }

    /// Return the 1-based physical line number of the deciding rule in the body.
    #[must_use]
    pub const fn line_number(&self) -> usize {
        self.line_number
    }
}

/// Why an evaluation reached its decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RobotsBasis {
    /// The most specific matching rule decided the request.
    MatchedRule(RobotsMatchedRule),
    /// No group named the crawler and no `*` group existed, so nothing applies.
    NoApplicableGroup,
    /// The applicable groups had no rule matching the path.
    NoMatchingRule,
    /// The path was `/robots.txt`, which is implicitly allowed (RFC 9309 section 2.2.2).
    ImplicitRobotsTxt,
    /// The body declared more than [`MAX_ROBOTS_RULES`] rules, so the result is unknown.
    RuleLimitExceeded,
    /// The robots file was unavailable (for example HTTP 4xx).
    Unavailable,
    /// The robots file was unreachable (for example HTTP 5xx or a network error).
    Unreachable,
    /// Fetching the robots file exceeded the redirect limit.
    RedirectLimitExceeded,
}

/// The value-free result of evaluating one request path.
///
/// Its `Debug` output contains no path or pattern text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RobotsEvaluation {
    decision: RobotsDecision,
    basis: RobotsBasis,
    body_truncated: bool,
}

impl RobotsEvaluation {
    /// Return the decision to hand to the policy evaluator.
    #[must_use]
    pub const fn decision(&self) -> RobotsDecision {
        self.decision
    }

    /// Return why the decision was reached.
    #[must_use]
    pub const fn basis(&self) -> RobotsBasis {
        self.basis
    }

    /// Return whether the body exceeded [`MAX_ROBOTS_BODY_BYTES`] and was cut.
    #[must_use]
    pub const fn body_truncated(&self) -> bool {
        self.body_truncated
    }
}

/// One normalized rule pattern.
///
/// `text` is ASCII; `*` occurs in it only as a wildcard and `$` never occurs.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RulePattern {
    text: String,
    anchored: bool,
    specificity: usize,
}

impl RulePattern {
    /// Accept a trimmed, non-empty rule value or return `None` to ignore it.
    fn parse(value: &str) -> Option<Self> {
        if !value.starts_with(['/', '*']) {
            return None;
        }
        // Non-short-circuit `|`: one combined character class, no internal whitespace.
        if value.contains(|character: char| character.is_control() | character.is_whitespace()) {
            return None;
        }
        let (raw, anchored) = value
            .strip_suffix('$')
            .map_or((value, false), |raw| (raw, true));
        let text = normalize(raw.as_bytes(), true);
        Some(Self {
            specificity: text.len() + usize::from(anchored),
            text,
            anchored,
        })
    }

    /// Match a normalized request path in time linear in the path and pattern.
    ///
    /// The pattern is split at `*`. The first segment must be a prefix; middle
    /// segments are found leftmost with the linear `str::find`; the last segment
    /// must be a suffix of the unconsumed remainder when anchored, or occur in it
    /// otherwise. Leftmost matching leaves the longest possible remainder, so
    /// this greedy scan is exact and never backtracks.
    fn matches(&self, path: &str) -> bool {
        let mut segments = self.text.split('*');
        let first = segments.next().unwrap_or_default();
        let last = segments.next_back();
        path.strip_prefix(first)
            .and_then(|rest| {
                segments.try_fold(rest, |rest, middle| {
                    rest.find(middle)
                        .and_then(|found| rest.get(found + middle.len()..))
                })
            })
            .is_some_and(|rest| match last {
                None => !self.anchored || rest.is_empty(),
                Some(last) if self.anchored => rest.ends_with(last),
                Some(last) => rest.contains(last),
            })
    }
}

/// One accepted rule.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Rule {
    kind: RobotsRuleKind,
    line_number: usize,
    pattern: RulePattern,
}

impl Rule {
    /// Ordering key: longest pattern, then `allow`, then the earliest line.
    fn priority(&self) -> (usize, u8, Reverse<usize>) {
        (
            self.pattern.specificity,
            self.kind.tie_rank(),
            Reverse(self.line_number),
        )
    }
}

/// One group: its user-agent names and the rules that follow them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Group {
    wildcard: bool,
    agents: Vec<String>,
    rules: Vec<Rule>,
}

impl Group {
    /// Return whether the group names this crawler (ASCII case-insensitive equality).
    fn names(&self, agent: &RobotsProductToken) -> bool {
        self.agents
            .iter()
            .any(|name| name.eq_ignore_ascii_case(agent.as_str()))
    }
}

/// Parsed robots rules for one body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobotsRules {
    groups: Vec<Group>,
    rule_count: usize,
    body_truncated: bool,
    rule_limit_exceeded: bool,
}

impl RobotsRules {
    /// Parse a robots body. Parsing never fails: invalid lines are ignored.
    ///
    /// Only the first [`MAX_ROBOTS_BODY_BYTES`] bytes are parsed; when a body is
    /// longer, a final partial line is dropped so that a cut rule can never
    /// widen into a shorter, broader rule.
    #[must_use]
    pub fn parse(body: &[u8]) -> Self {
        let body_truncated = body.len() > MAX_ROBOTS_BODY_BYTES;
        let mut kept = body.get(..MAX_ROBOTS_BODY_BYTES).unwrap_or(body);
        if body_truncated {
            let complete = kept
                .iter()
                .rposition(|byte| matches!(byte, b'\r' | b'\n'))
                .map_or(0, |end| end + 1);
            kept = kept.get(..complete).unwrap_or_default();
        }
        let mut rest = kept.strip_prefix(UTF8_BOM).unwrap_or(kept);

        let mut parser = Parser::default();
        let mut line_number = 0;
        while !rest.is_empty() {
            let end = rest
                .iter()
                .position(|byte| matches!(byte, b'\r' | b'\n'))
                .unwrap_or(rest.len());
            let (line, terminator) = rest.split_at_checked(end).unwrap_or((rest, &[]));
            line_number += 1;
            parser.line(line_number, line);
            rest = terminator
                .strip_prefix(b"\r\n")
                .or_else(|| terminator.get(1..))
                .unwrap_or_default();
        }
        parser.finish(body_truncated)
    }

    /// Decide whether `agent` may crawl `path` under these rules.
    ///
    /// `path` is the request target path including any query string.
    ///
    /// # Errors
    ///
    /// Returns a [`RobotsPathError`] when the path is not a bounded,
    /// slash-rooted, control-free and fragment-free request path.
    pub fn decide(
        &self,
        agent: &RobotsProductToken,
        path: &str,
    ) -> Result<RobotsEvaluation, RobotsPathError> {
        let request = RequestPath::parse(path)?;
        Ok(self.decide_request(agent, &request))
    }

    /// Return the number of stored accepted rules (at most [`MAX_ROBOTS_RULES`]).
    #[must_use]
    pub const fn rule_count(&self) -> usize {
        self.rule_count
    }

    /// Return whether the body exceeded [`MAX_ROBOTS_BODY_BYTES`] and was cut.
    #[must_use]
    pub const fn body_truncated(&self) -> bool {
        self.body_truncated
    }

    /// Return whether the body declared more than [`MAX_ROBOTS_RULES`] rules.
    #[must_use]
    pub const fn rule_limit_exceeded(&self) -> bool {
        self.rule_limit_exceeded
    }

    fn decide_request(
        &self,
        agent: &RobotsProductToken,
        request: &RequestPath,
    ) -> RobotsEvaluation {
        request.evaluate(self.body_truncated, &|path| self.judge(agent, path))
    }

    fn judge(&self, agent: &RobotsProductToken, path: &str) -> (RobotsDecision, RobotsBasis) {
        if self.rule_limit_exceeded {
            return (RobotsDecision::Unknown, RobotsBasis::RuleLimitExceeded);
        }
        let mut applicable: Vec<&Group> = self
            .groups
            .iter()
            .filter(|group| group.names(agent))
            .collect();
        if applicable.is_empty() {
            applicable = self.groups.iter().filter(|group| group.wildcard).collect();
        }
        if applicable.is_empty() {
            return (RobotsDecision::Allowed, RobotsBasis::NoApplicableGroup);
        }
        applicable
            .iter()
            .flat_map(|group| group.rules.iter())
            .filter(|rule| rule.pattern.matches(path))
            .max_by_key(|rule| rule.priority())
            .map_or(
                (RobotsDecision::Allowed, RobotsBasis::NoMatchingRule),
                |rule| {
                    (
                        rule.kind.decision(),
                        RobotsBasis::MatchedRule(RobotsMatchedRule {
                            kind: rule.kind,
                            line_number: rule.line_number,
                        }),
                    )
                },
            )
    }
}

/// Incremental group builder used by [`RobotsRules::parse`].
#[derive(Default)]
struct Parser {
    groups: Vec<Group>,
    current: Option<Group>,
    in_agent_run: bool,
    rule_count: usize,
    rule_limit_exceeded: bool,
}

impl Parser {
    fn line(&mut self, line_number: usize, line: &[u8]) {
        let Ok(line) = std::str::from_utf8(line) else {
            return;
        };
        let content = line.split_once('#').map_or(line, |(before, _)| before);
        let Some((key, value)) = content.split_once(':') else {
            return;
        };
        let key = key.trim_matches(RECORD_WHITESPACE);
        let value = value.trim_matches(RECORD_WHITESPACE);
        let kind = if key.eq_ignore_ascii_case("user-agent") {
            self.user_agent(value);
            return;
        } else if key.eq_ignore_ascii_case("allow") {
            RobotsRuleKind::Allow
        } else if key.eq_ignore_ascii_case("disallow") {
            RobotsRuleKind::Disallow
        } else {
            return;
        };
        self.rule(kind, line_number, value);
    }

    fn user_agent(&mut self, value: &str) {
        if !self.in_agent_run {
            self.groups.extend(self.current.take());
            self.in_agent_run = true;
        }
        let group = self.current.get_or_insert_with(Group::default);
        group.wildcard |= value == "*";
        let end = value
            .bytes()
            .position(|byte| !is_product_token_byte(byte))
            .unwrap_or(value.len());
        let token = value.get(..end).unwrap_or_default();
        // A stored name longer than any valid product token can never match,
        // so keep at most one byte beyond the limit.
        let token = token
            .get(..=MAX_ROBOTS_PRODUCT_TOKEN_BYTES)
            .unwrap_or(token);
        group.agents.push(token.to_owned());
    }

    fn rule(&mut self, kind: RobotsRuleKind, line_number: usize, value: &str) {
        self.in_agent_run = false;
        let Some(group) = self.current.as_mut() else {
            return;
        };
        let Some(pattern) = RulePattern::parse(value) else {
            return;
        };
        if self.rule_count == MAX_ROBOTS_RULES {
            self.rule_limit_exceeded = true;
            return;
        }
        self.rule_count += 1;
        group.rules.push(Rule {
            kind,
            line_number,
            pattern,
        });
    }

    fn finish(mut self, body_truncated: bool) -> RobotsRules {
        self.groups.extend(self.current.take());
        RobotsRules {
            groups: self.groups,
            rule_count: self.rule_count,
            body_truncated,
            rule_limit_exceeded: self.rule_limit_exceeded,
        }
    }
}

/// A validated, normalized request path.
struct RequestPath(String);

impl RequestPath {
    fn parse(path: &str) -> Result<Self, RobotsPathError> {
        if !path.starts_with('/') {
            return Err(RobotsPathError::MissingLeadingSlash);
        }
        for character in path.chars() {
            if character.is_control() {
                return Err(RobotsPathError::ControlCharacter);
            }
            if character == '#' {
                return Err(RobotsPathError::Fragment);
            }
        }
        let bounded = path
            .as_bytes()
            .get(..NORMALIZATION_INPUT_BYTES)
            .unwrap_or(path.as_bytes());
        let normalized = normalize(bounded, false);
        if normalized.len() > MAX_ROBOTS_PATH_BYTES {
            return Err(RobotsPathError::TooLong);
        }
        Ok(Self(normalized))
    }

    /// Apply the implicit `/robots.txt` allowance, otherwise defer to `judge`.
    fn evaluate(
        &self,
        body_truncated: bool,
        judge: &dyn Fn(&str) -> (RobotsDecision, RobotsBasis),
    ) -> RobotsEvaluation {
        let (decision, basis) = if self.0 == ROBOTS_TXT_PATH {
            (RobotsDecision::Allowed, RobotsBasis::ImplicitRobotsTxt)
        } else {
            judge(&self.0)
        };
        RobotsEvaluation {
            decision,
            basis,
            body_truncated,
        }
    }
}

/// The result of fetching `/robots.txt`, as classified by the caller's transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RobotsFetchOutcome<'a> {
    /// The file was fetched successfully (HTTP 2xx after at most five redirects).
    Success {
        /// The raw response body; only a bounded prefix is parsed.
        body: &'a [u8],
    },
    /// The file was unavailable, for example HTTP 4xx (RFC 9309 section 2.3.1.3).
    Unavailable,
    /// The file was unreachable, for example HTTP 5xx or a network error
    /// (RFC 9309 section 2.3.1.4).
    Unreachable,
    /// More than five consecutive redirects were encountered (RFC 9309 section 2.3.1.2).
    RedirectLimitExceeded,
}

/// How an unavailable or redirect-exhausted robots file is treated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnavailableRobotsPolicy {
    /// Allow crawling, as RFC 9309 section 2.3.1.3 permits.
    AllowPerRfc9309,
    /// Treat the robots decision as unknown, which policy denies.
    TreatAsUnknown,
}

impl UnavailableRobotsPolicy {
    const fn decision(self) -> RobotsDecision {
        match self {
            Self::AllowPerRfc9309 => RobotsDecision::Allowed,
            Self::TreatAsUnknown => RobotsDecision::Unknown,
        }
    }
}

/// Evaluate one request path against a classified robots fetch outcome.
///
/// The path is validated before the outcome is considered. An unreachable
/// file means complete disallow; an unavailable or redirect-exhausted file
/// follows `unavailable`. `/robots.txt` itself is always allowed.
///
/// # Errors
///
/// Returns a [`RobotsPathError`] for an invalid request path, whatever the
/// fetch outcome.
pub fn evaluate_robots(
    outcome: RobotsFetchOutcome<'_>,
    unavailable: UnavailableRobotsPolicy,
    agent: &RobotsProductToken,
    path: &str,
) -> Result<RobotsEvaluation, RobotsPathError> {
    let request = RequestPath::parse(path)?;
    let fixed = match outcome {
        RobotsFetchOutcome::Success { body } => {
            return Ok(RobotsRules::parse(body).decide_request(agent, &request));
        }
        RobotsFetchOutcome::Unreachable => (RobotsDecision::Disallowed, RobotsBasis::Unreachable),
        RobotsFetchOutcome::Unavailable => (unavailable.decision(), RobotsBasis::Unavailable),
        RobotsFetchOutcome::RedirectLimitExceeded => {
            (unavailable.decision(), RobotsBasis::RedirectLimitExceeded)
        }
    };
    Ok(request.evaluate(false, &|_| fixed))
}

/// Return whether a byte may appear in a product token.
const fn is_product_token_byte(byte: u8) -> bool {
    matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'_' | b'-')
}

/// Return whether a decoded octet is an RFC 3986 unreserved character.
const fn is_unreserved(byte: u8) -> bool {
    matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~')
}

/// Return the value of an ASCII hexadecimal digit.
fn hex_value(byte: Option<&u8>) -> Option<u8> {
    byte.and_then(|byte| char::from(*byte).to_digit(16))
        .and_then(|digit| u8::try_from(digit).ok())
}

/// Append `%XX` with uppercase hexadecimal digits.
fn push_percent_encoded(out: &mut String, byte: u8) {
    out.push('%');
    for nibble in [byte >> 4, byte & 0x0F] {
        out.push(
            char::from_digit(u32::from(nibble), 16)
                .unwrap_or_default()
                .to_ascii_uppercase(),
        );
    }
}

/// Normalize a rule literal or request path into comparable ASCII.
///
/// Non-ASCII bytes are percent-encoded, `%HH` of an unreserved character is
/// decoded, any other valid `%HH` is uppercased, and a stray `%` becomes `%25`.
/// `$` is always encoded; `*` is kept as a wildcard only when `wildcard` is set
/// (rules) and is encoded otherwise (request paths).
fn normalize(raw: &[u8], wildcard: bool) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut index = 0;
    while let Some(&byte) = raw.get(index) {
        index += 1;
        match byte {
            b'%' => {
                match hex_value(raw.get(index))
                    .zip(hex_value(raw.get(index + 1)))
                    .map(|(high, low)| (high << 4) | low)
                {
                    Some(decoded) => {
                        index += 2;
                        if is_unreserved(decoded) {
                            out.push(char::from(decoded));
                        } else {
                            push_percent_encoded(&mut out, decoded);
                        }
                    }
                    None => out.push_str("%25"),
                }
            }
            b'*' if wildcard => out.push('*'),
            b'*' | b'$' | 0x80..=0xFF => push_percent_encoded(&mut out, byte),
            _ => out.push(char::from(byte)),
        }
    }
    out
}
