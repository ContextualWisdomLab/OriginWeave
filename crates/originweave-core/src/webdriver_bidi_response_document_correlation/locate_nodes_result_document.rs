use super::WebDriverBiDiLocateNodesResponseDocumentError;

pub(super) struct WireLocateNodesNode {
    remote_type: String,
    shared_id: Option<String>,
}

impl WireLocateNodesNode {
    pub(super) fn as_admission_parts(&self) -> (&str, Option<&str>) {
        (self.remote_type.as_str(), self.shared_id.as_deref())
    }
}

pub(super) fn parse_wire_locate_nodes_result(
    input: &str,
) -> Result<Vec<WireLocateNodesNode>, WebDriverBiDiLocateNodesResponseDocumentError> {
    ResultParser::new(input).parse()
}

struct ResultParser<'input> {
    input: &'input [u8],
    position: usize,
}

impl<'input> ResultParser<'input> {
    const fn new(input: &'input str) -> Self {
        Self {
            input: input.as_bytes(),
            position: 0,
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
                    return Err(WebDriverBiDiLocateNodesResponseDocumentError::DuplicateResultNodes);
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
        if self.peek_byte() == Some(b']') {
            self.position += 1;
            return Ok(nodes);
        }

        loop {
            nodes.push(self.parse_node()?);
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
        let mut decoded = Vec::new();
        loop {
            let byte = self
                .peek_byte()
                .ok_or(WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant)?;
            match byte {
                b'"' => {
                    self.position += 1;
                    return String::from_utf8(decoded).map_err(|_error| {
                        WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant
                    });
                }
                b'\\' => {
                    self.position += 1;
                    self.parse_escape(&mut decoded)?;
                }
                0x00..=0x1f => {
                    return Err(
                        WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant,
                    );
                }
                _ => {
                    decoded.push(byte);
                    self.position += 1;
                }
            }
        }
    }

    fn parse_escape(
        &mut self,
        decoded: &mut Vec<u8>,
    ) -> Result<(), WebDriverBiDiLocateNodesResponseDocumentError> {
        let escaped = self
            .peek_byte()
            .ok_or(WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant)?;
        self.position += 1;
        match escaped {
            b'"' => decoded.push(b'"'),
            b'\\' => decoded.push(b'\\'),
            b'/' => decoded.push(b'/'),
            b'b' => decoded.push(0x08),
            b'f' => decoded.push(0x0c),
            b'n' => decoded.push(b'\n'),
            b'r' => decoded.push(b'\r'),
            b't' => decoded.push(b'\t'),
            b'u' => self.parse_unicode_escape(decoded)?,
            _ => return Err(WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant),
        }
        Ok(())
    }

    fn parse_unicode_escape(
        &mut self,
        decoded: &mut Vec<u8>,
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
            if (0xdc00..=0xdfff).contains(&first) {
                return Err(WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant);
            }
            u32::from(first)
        };
        let character = char::from_u32(scalar)
            .ok_or(WebDriverBiDiLocateNodesResponseDocumentError::ResultParserInvariant)?;
        let mut encoded = [0_u8; 4];
        decoded.extend_from_slice(character.encode_utf8(&mut encoded).as_bytes());
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
        self.input.get(self.position).copied()
    }
}
