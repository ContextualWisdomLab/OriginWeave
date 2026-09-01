use super::WebDriverBiDiLocateNodesResponseDocumentError;

pub(super) struct WireLocateNodesNode {
    remote_type: String,
    shared_id: Option<String>,
}

impl WireLocateNodesNode {
    pub(super) fn as_admission_parts(&self) -> (&str, Option<&str>) {
        (self.remote_type.as_str(), self.shared_id.as_deref())
    }

    fn overflow_count_marker() -> Self {
        Self {
            remote_type: String::new(),
            shared_id: None,
        }
    }
}

#[cfg(test)]
pub(super) fn parse_wire_locate_nodes_result(
    input: &str,
) -> Result<Vec<WireLocateNodesNode>, WebDriverBiDiLocateNodesResponseDocumentError> {
    ResultParser::new(input).parse()
}

pub(super) fn parse_wire_locate_nodes_result_bounded(
    input: &str,
    max_node_count: u16,
) -> Result<Vec<WireLocateNodesNode>, WebDriverBiDiLocateNodesResponseDocumentError> {
    ResultParser::with_node_budget(input, usize::from(max_node_count)).parse()
}

struct ResultParser<'input> {
    input: &'input str,
    position: usize,
    max_node_count: Option<usize>,
}

impl<'input> ResultParser<'input> {
    #[cfg(test)]
    const fn new(input: &'input str) -> Self {
        Self {
            input,
            position: 0,
            max_node_count: None,
        }
    }

    const fn with_node_budget(input: &'input str, max_node_count: usize) -> Self {
        Self {
            input,
            position: 0,
            max_node_count: Some(max_node_count),
        }
    }

    fn parse(
        mut self,
    ) -> Result<Vec<WireLocateNodesNode>, WebDriverBiDiLocateNodesResponseDocumentError> {
        self.skip_whitespace();
        self.expect_byte(b'{')?;
        self.skip_whitespace();

        loop {
            if self.peek_byte() == Some(b'}') {
                return Err(WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant);
            }
            let field_name = self.parse_string()?;
            self.skip_whitespace();
            self.expect_byte(b':')?;
            self.skip_whitespace();
            if field_name == "result" {
                return self.parse_result_object();
            }
            self.skip_value()?;
            self.skip_whitespace();
            match self.peek_byte() {
                Some(b',') => {
                    self.position += 1;
                    self.skip_whitespace();
                }
                Some(b'}') => {
                    return Err(
                        WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant,
                    );
                }
                _ => {
                    return Err(
                        WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant,
                    );
                }
            }
        }
    }

    fn parse_result_object(
        &mut self,
    ) -> Result<Vec<WireLocateNodesNode>, WebDriverBiDiLocateNodesResponseDocumentError> {
        if self.peek_byte() != Some(b'{') {
            return Err(WebDriverBiDiLocateNodesResponseDocumentError::InvalidResultNodes);
        }
        self.position += 1;
        self.skip_whitespace();
        let mut nodes = None;

        if self.peek_byte() == Some(b'}') {
            self.position += 1;
            return Err(WebDriverBiDiLocateNodesResponseDocumentError::MissingResultNodes);
        }

        loop {
            let field_name = self.parse_string()?;
            self.skip_whitespace();
            self.expect_byte(b':')?;
            self.skip_whitespace();
            if field_name == "nodes" {
                if nodes.is_some() {
                    return Err(
                        WebDriverBiDiLocateNodesResponseDocumentError::DuplicateResultNodes,
                    );
                }
                nodes = Some(self.parse_nodes_array()?);
            } else {
                self.skip_value()?;
            }
            self.skip_whitespace();
            match self.peek_byte() {
                Some(b',') => {
                    self.position += 1;
                    self.skip_whitespace();
                }
                Some(b'}') => {
                    self.position += 1;
                    break;
                }
                _ => {
                    return Err(
                        WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant,
                    );
                }
            }
        }

        nodes.ok_or(WebDriverBiDiLocateNodesResponseDocumentError::MissingResultNodes)
    }

    fn parse_nodes_array(
        &mut self,
    ) -> Result<Vec<WireLocateNodesNode>, WebDriverBiDiLocateNodesResponseDocumentError> {
        if self.peek_byte() != Some(b'[') {
            return Err(WebDriverBiDiLocateNodesResponseDocumentError::InvalidResultNodes);
        }
        self.position += 1;
        self.skip_whitespace();
        let mut nodes = Vec::new();
        let mut over_budget = false;
        if self.peek_byte() == Some(b']') {
            self.position += 1;
            return Ok(nodes);
        }

        loop {
            let at_node_budget = self
                .max_node_count
                .is_some_and(|max_node_count| nodes.len() >= max_node_count);
            if over_budget || at_node_budget {
                self.skip_value()?;
                if !over_budget {
                    nodes.push(WireLocateNodesNode::overflow_count_marker());
                    over_budget = true;
                }
            } else {
                nodes.push(self.parse_node()?);
            }
            self.skip_whitespace();
            match self.peek_byte() {
                Some(b',') => {
                    self.position += 1;
                    self.skip_whitespace();
                }
                Some(b']') => {
                    self.position += 1;
                    return Ok(nodes);
                }
                _ => {
                    return Err(
                        WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant,
                    );
                }
            }
        }
    }

    fn parse_node(
        &mut self,
    ) -> Result<WireLocateNodesNode, WebDriverBiDiLocateNodesResponseDocumentError> {
        if self.peek_byte() != Some(b'{') {
            return Err(WebDriverBiDiLocateNodesResponseDocumentError::InvalidResultNode);
        }
        self.position += 1;
        self.skip_whitespace();
        let mut remote_type = None;
        let mut shared_id = None;
        let mut shared_id_seen = false;

        if self.peek_byte() == Some(b'}') {
            self.position += 1;
            return Err(WebDriverBiDiLocateNodesResponseDocumentError::MissingResultNodeType);
        }

        loop {
            let field_name = self.parse_string()?;
            self.skip_whitespace();
            self.expect_byte(b':')?;
            self.skip_whitespace();
            match field_name.as_str() {
                "type" => {
                    if remote_type.is_some() {
                        return Err(
                            WebDriverBiDiLocateNodesResponseDocumentError::DuplicateResultNodeField,
                        );
                    }
                    if self.peek_byte() != Some(b'"') {
                        return Err(
                            WebDriverBiDiLocateNodesResponseDocumentError::InvalidResultNodeType,
                        );
                    }
                    remote_type = Some(self.parse_string()?);
                }
                "sharedId" => {
                    if shared_id_seen {
                        return Err(
                            WebDriverBiDiLocateNodesResponseDocumentError::DuplicateResultNodeField,
                        );
                    }
                    shared_id_seen = true;
                    if self.peek_byte() != Some(b'"') {
                        return Err(
                            WebDriverBiDiLocateNodesResponseDocumentError::InvalidResultNodeSharedId,
                        );
                    }
                    shared_id = Some(self.parse_string()?);
                }
                _ => self.skip_value()?,
            }
            self.skip_whitespace();
            match self.peek_byte() {
                Some(b',') => {
                    self.position += 1;
                    self.skip_whitespace();
                }
                Some(b'}') => {
                    self.position += 1;
                    break;
                }
                _ => {
                    return Err(
                        WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant,
                    );
                }
            }
        }

        Ok(WireLocateNodesNode {
            remote_type: remote_type
                .ok_or(WebDriverBiDiLocateNodesResponseDocumentError::MissingResultNodeType)?,
            shared_id,
        })
    }

    fn skip_value(&mut self) -> Result<(), WebDriverBiDiLocateNodesResponseDocumentError> {
        match self.peek_byte() {
            Some(b'{') => self.skip_object(),
            Some(b'[') => self.skip_array(),
            Some(b'"') => {
                let _value = self.parse_string()?;
                Ok(())
            }
            Some(b'-' | b'0'..=b'9') => self.skip_number(),
            Some(b't') => self.skip_literal(b"true"),
            Some(b'f') => self.skip_literal(b"false"),
            Some(b'n') => self.skip_literal(b"null"),
            _ => Err(WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant),
        }
    }

    fn skip_object(&mut self) -> Result<(), WebDriverBiDiLocateNodesResponseDocumentError> {
        self.expect_byte(b'{')?;
        self.skip_whitespace();
        if self.peek_byte() == Some(b'}') {
            self.position += 1;
            return Ok(());
        }
        loop {
            let _field_name = self.parse_string()?;
            self.skip_whitespace();
            self.expect_byte(b':')?;
            self.skip_whitespace();
            self.skip_value()?;
            self.skip_whitespace();
            match self.peek_byte() {
                Some(b',') => {
                    self.position += 1;
                    self.skip_whitespace();
                }
                Some(b'}') => {
                    self.position += 1;
                    return Ok(());
                }
                _ => {
                    return Err(
                        WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant,
                    );
                }
            }
        }
    }

    fn skip_array(&mut self) -> Result<(), WebDriverBiDiLocateNodesResponseDocumentError> {
        self.expect_byte(b'[')?;
        self.skip_whitespace();
        if self.peek_byte() == Some(b']') {
            self.position += 1;
            return Ok(());
        }
        loop {
            self.skip_value()?;
            self.skip_whitespace();
            match self.peek_byte() {
                Some(b',') => {
                    self.position += 1;
                    self.skip_whitespace();
                }
                Some(b']') => {
                    self.position += 1;
                    return Ok(());
                }
                _ => {
                    return Err(
                        WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant,
                    );
                }
            }
        }
    }

    fn skip_number(&mut self) -> Result<(), WebDriverBiDiLocateNodesResponseDocumentError> {
        let start = self.position;
        while let Some(byte) = self.peek_byte() {
            if matches!(byte, b',' | b']' | b'}' | b' ' | b'\t' | b'\r' | b'\n') {
                break;
            }
            self.position += 1;
        }
        if self.position == start {
            return Err(WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant);
        }
        Ok(())
    }

    fn skip_literal(
        &mut self,
        literal: &[u8],
    ) -> Result<(), WebDriverBiDiLocateNodesResponseDocumentError> {
        for expected in literal {
            self.expect_byte(*expected)?;
        }
        Ok(())
    }

    fn parse_string(&mut self) -> Result<String, WebDriverBiDiLocateNodesResponseDocumentError> {
        self.expect_byte(b'"')?;
        let mut decoded = String::new();
        let mut literal_start = self.position;
        loop {
            let byte = self
                .peek_byte()
                .ok_or(WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant)?;
            match byte {
                b'"' => {
                    decoded.push_str(&self.input[literal_start..self.position]);
                    self.position += 1;
                    return Ok(decoded);
                }
                b'\\' => {
                    decoded.push_str(&self.input[literal_start..self.position]);
                    self.position += 1;
                    self.parse_escape(&mut decoded)?;
                    literal_start = self.position;
                }
                0x00..=0x1f => {
                    return Err(
                        WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant,
                    );
                }
                _ => {
                    self.position += 1;
                }
            }
        }
    }

    fn parse_escape(
        &mut self,
        decoded: &mut String,
    ) -> Result<(), WebDriverBiDiLocateNodesResponseDocumentError> {
        let escaped = self
            .peek_byte()
            .ok_or(WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant)?;
        self.position += 1;
        match escaped {
            b'"' => decoded.push('"'),
            b'\\' => decoded.push('\\'),
            b'/' => decoded.push('/'),
            b'b' => decoded.push('\u{0008}'),
            b'f' => decoded.push('\u{000c}'),
            b'n' => decoded.push('\n'),
            b'r' => decoded.push('\r'),
            b't' => decoded.push('\t'),
            b'u' => self.parse_unicode_escape(decoded)?,
            _ => return Err(WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant),
        }
        Ok(())
    }

    fn parse_unicode_escape(
        &mut self,
        decoded: &mut String,
    ) -> Result<(), WebDriverBiDiLocateNodesResponseDocumentError> {
        let first = self.parse_hex_quad()?;
        let scalar = if (0xd800..=0xdbff).contains(&first) {
            self.expect_byte(b'\\')?;
            self.expect_byte(b'u')?;
            let second = self.parse_hex_quad()?;
            if !(0xdc00..=0xdfff).contains(&second) {
                return Err(WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant);
            }
            0x1_0000 + (((u32::from(first) - 0xd800) << 10) | (u32::from(second) - 0xdc00))
        } else {
            u32::from(first)
        };
        let character = char::from_u32(scalar)
            .ok_or(WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant)?;
        decoded.push(character);
        Ok(())
    }

    fn parse_hex_quad(&mut self) -> Result<u16, WebDriverBiDiLocateNodesResponseDocumentError> {
        let mut value = 0_u16;
        for _ in 0..4 {
            let byte = self
                .peek_byte()
                .ok_or(WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant)?;
            self.position += 1;
            let digit = match byte {
                b'0'..=b'9' => u16::from(byte - b'0'),
                b'a'..=b'f' => u16::from(byte - b'a' + 10),
                b'A'..=b'F' => u16::from(byte - b'A' + 10),
                _ => {
                    return Err(
                        WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant,
                    );
                }
            };
            value = (value << 4) | digit;
        }
        Ok(value)
    }

    fn expect_byte(
        &mut self,
        expected: u8,
    ) -> Result<(), WebDriverBiDiLocateNodesResponseDocumentError> {
        if self.peek_byte() != Some(expected) {
            return Err(WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant);
        }
        self.position += 1;
        Ok(())
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_byte(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.position += 1;
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.input.as_bytes().get(self.position).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INVARIANT: WebDriverBiDiLocateNodesResponseDocumentError =
        WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant;

    #[test]
    fn second_pass_parser_covers_valid_skipped_value_and_escape_shapes() {
        let raw = concat!(
            " \n{\t\"metadata\": [true,false,null,{\"a\":1,\"b\":2}],",
            "\"result\":{",
            "\"emptyObject\":{},\"emptyArray\":[],",
            "\"object\":{\"first\":1,\"second\":2},",
            "\"array\":[true,false,null],",
            "\"escaped\":\"\\\"\\\\\\/\\b\\f\\n\\r\\t\",",
            "\"utf8\":\"é\",",
            "\"number\":-1.25e+2,\"truth\":true,\"falsehood\":false,\"nothing\":null,",
            "\"nodes\":[{",
            "\"ignored\":{\"nested\":[1,2]},",
            "\"type\":\"no\\u0064e\",",
            "\"sharedId\":\"node-\\u0041-\\u00E9-\\u263A-\\uD83D\\uDE00-\\u00af-\\u00AF\"",
            "}]}}"
        );
        let nodes = parse_wire_locate_nodes_result(raw)
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].remote_type, "node");
        assert_eq!(nodes[0].shared_id.as_deref(), Some("node-A-é-☺-😀-¯-¯"));
    }

    #[test]
    fn second_pass_parser_rejects_structural_and_typed_result_faults() {
        let cases = [
            ("", INVARIANT),
            ("{}", INVARIANT),
            (r#"{"metadata":0}"#, INVARIANT),
            (r#"{"metadata":0]"#, INVARIANT),
            (r#"{"metadata" 0,"result":{"nodes":[]}}"#, INVARIANT),
            (r#"{metadata:0,"result":{"nodes":[]}}"#, INVARIANT),
            (
                r#"{"result":[]}"#,
                WebDriverBiDiLocateNodesResponseDocumentError::InvalidResultNodes,
            ),
            (
                r#"{"result":{}}"#,
                WebDriverBiDiLocateNodesResponseDocumentError::MissingResultNodes,
            ),
            (
                r#"{"result":{"other":0}}"#,
                WebDriverBiDiLocateNodesResponseDocumentError::MissingResultNodes,
            ),
            (
                r#"{"result":{"nodes":0}}"#,
                WebDriverBiDiLocateNodesResponseDocumentError::InvalidResultNodes,
            ),
            (
                r#"{"result":{"nodes":[0]}}"#,
                WebDriverBiDiLocateNodesResponseDocumentError::InvalidResultNode,
            ),
            (
                r#"{"result":{"nodes":[{}]}}"#,
                WebDriverBiDiLocateNodesResponseDocumentError::MissingResultNodeType,
            ),
            (
                r#"{"result":{"nodes":[{"sharedId":"node-a"}]}}"#,
                WebDriverBiDiLocateNodesResponseDocumentError::MissingResultNodeType,
            ),
            (
                r#"{"result":{"nodes":[{"type":0}]}}"#,
                WebDriverBiDiLocateNodesResponseDocumentError::InvalidResultNodeType,
            ),
            (
                r#"{"result":{"nodes":[{"type":"node","sharedId":0}]}}"#,
                WebDriverBiDiLocateNodesResponseDocumentError::InvalidResultNodeSharedId,
            ),
            (
                r#"{"result":{"nodes":[],"nodes":[]}}"#,
                WebDriverBiDiLocateNodesResponseDocumentError::DuplicateResultNodes,
            ),
            (
                r#"{"result":{"nodes":[{"type":"node","type":"node"}]}}"#,
                WebDriverBiDiLocateNodesResponseDocumentError::DuplicateResultNodeField,
            ),
            (
                r#"{"result":{"nodes":[{"type":"node","sharedId":"a","sharedId":"b"}]}}"#,
                WebDriverBiDiLocateNodesResponseDocumentError::DuplicateResultNodeField,
            ),
            (r#"{"result":{"nodes":[] "other":0}}"#, INVARIANT),
            (r#"{"result":{"nodes":[{"type":"node"} 0]}}"#, INVARIANT),
            (
                r#"{"result":{"nodes":[{"type":"node" "sharedId":"node-a"}]}}"#,
                INVARIANT,
            ),
            (r#"{"metadata":?,"result":{"nodes":[]}}"#, INVARIANT),
            (r#"{"result":{?}}"#, INVARIANT),
            (r#"{"result":{"nodes" []}}"#, INVARIANT),
            (r#"{"result":{"other":?,"nodes":[]}}"#, INVARIANT),
            (r#"{"result":{"nodes":[{?}]}}"#, INVARIANT),
            (r#"{"result":{"nodes":[{"type" "node"}]}}"#, INVARIANT),
            (r#"{"result":{"nodes":[{"type":"\x"}]}}"#, INVARIANT),
            (
                r#"{"result":{"nodes":[{"type":"node","sharedId":"\x"}]}}"#,
                INVARIANT,
            ),
            (
                r#"{"result":{"nodes":[{"ignored":?,"type":"node"}]}}"#,
                INVARIANT,
            ),
        ];

        for (raw, expected) in cases {
            assert_eq!(parse_wire_locate_nodes_result(raw).err(), Some(expected));
        }
    }

    #[test]
    fn bounded_parser_uses_one_count_marker_and_skips_overflow_node_shapes() {
        let nodes = parse_wire_locate_nodes_result_bounded(
            r#"{"result":{"nodes":[{"type":"node","sharedId":"node-a"},{"type":1},{"type":2}]}}"#,
            1,
        )
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].remote_type, "node");
        assert_eq!(nodes[1].as_admission_parts(), ("", None));
    }

    #[test]
    fn skip_helpers_cover_empty_nonempty_and_malformed_containers() {
        let mut empty_object = ResultParser::new("{}");
        assert_eq!(empty_object.skip_object(), Ok(()));

        let mut object = ResultParser::new(r#"{"a":0,"b":1}"#);
        assert_eq!(object.skip_object(), Ok(()));

        let mut malformed_object = ResultParser::new(r#"{"a":0 "b":1}"#);
        assert_eq!(malformed_object.skip_object(), Err(INVARIANT));

        let mut empty_array = ResultParser::new("[]");
        assert_eq!(empty_array.skip_array(), Ok(()));

        let mut array = ResultParser::new("[0,1]");
        assert_eq!(array.skip_array(), Ok(()));

        let mut malformed_array = ResultParser::new("[0 1]");
        assert_eq!(malformed_array.skip_array(), Err(INVARIANT));

        let mut unknown = ResultParser::new("?");
        assert_eq!(unknown.skip_value(), Err(INVARIANT));

        let mut empty_number = ResultParser::new("");
        assert_eq!(empty_number.skip_number(), Err(INVARIANT));

        let mut terminal_number = ResultParser::new("123");
        assert_eq!(terminal_number.skip_number(), Ok(()));
        assert_eq!(terminal_number.peek_byte(), None);

        let mut malformed_string_value = ResultParser::new(r#""\x""#);
        assert_eq!(malformed_string_value.skip_value(), Err(INVARIANT));

        let mut wrong_object_opener = ResultParser::new("[]");
        assert_eq!(wrong_object_opener.skip_object(), Err(INVARIANT));

        let mut malformed_object_key = ResultParser::new("{?}");
        assert_eq!(malformed_object_key.skip_object(), Err(INVARIANT));

        let mut missing_object_colon = ResultParser::new(r#"{"a" 0}"#);
        assert_eq!(missing_object_colon.skip_object(), Err(INVARIANT));

        let mut malformed_object_value = ResultParser::new(r#"{"a":?}"#);
        assert_eq!(malformed_object_value.skip_object(), Err(INVARIANT));

        let mut wrong_array_opener = ResultParser::new("{}");
        assert_eq!(wrong_array_opener.skip_array(), Err(INVARIANT));

        let mut malformed_array_value = ResultParser::new("[?]");
        assert_eq!(malformed_array_value.skip_array(), Err(INVARIANT));

        let mut truncated_literal = ResultParser::new("tru");
        assert_eq!(truncated_literal.skip_literal(b"true"), Err(INVARIANT));
    }

    #[test]
    fn string_decoder_rejects_all_second_pass_escape_invariants() {
        let mut missing_open_quote = ResultParser::new("plain");
        assert_eq!(missing_open_quote.parse_string(), Err(INVARIANT));

        let mut unterminated = ResultParser::new("\"plain");
        assert_eq!(unterminated.parse_string(), Err(INVARIANT));

        let mut control = ResultParser::new("\"\u{0001}\"");
        assert_eq!(control.parse_string(), Err(INVARIANT));

        let mut missing_escape = ResultParser::new("\"\\");
        assert_eq!(missing_escape.parse_string(), Err(INVARIANT));

        let mut invalid_escape = ResultParser::new(r#""\x""#);
        assert_eq!(invalid_escape.parse_string(), Err(INVARIANT));

        for raw in [
            r#""\uD83D""#,
            r#""\uD83D\x0000""#,
            r#""\uD83D\u0041""#,
            "\"\\uD83D\\u12",
            r#""\uDE00""#,
            r#""\u12""#,
            "\"\\u12",
            r#""\u00G0""#,
        ] {
            let mut parser = ResultParser::new(raw);
            assert_eq!(parser.parse_string(), Err(INVARIANT));
        }
    }
}
