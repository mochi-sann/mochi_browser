use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlToken {
    Doctype(String),
    StartTag {
        name: String,
        attributes: Vec<(String, String)>,
        self_closing: bool,
    },
    EndTag {
        name: String,
    },
    Text(String),
    Comment(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenizeError {
    UnexpectedEOF,
    InvalidTag,
    InvalidAttribute,
    MalformedComment,
}

impl fmt::Display for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEOF => write!(f, "Unexpected end of input"),
            Self::InvalidTag => write!(f, "Invalid HTML tag"),
            Self::InvalidAttribute => write!(f, "Invalid HTML attribute"),
            Self::MalformedComment => write!(f, "Malformed HTML comment"),
        }
    }
}

impl std::error::Error for TokenizeError {}

pub struct HtmlTokenizer<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> HtmlTokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.input.chars().nth(self.position + offset)
    }

    fn advance(&mut self) -> Option<char> {
        if self.position < self.input.len() {
            let c = self.input.chars().nth(self.position);
            self.position += 1;
            c
        } else {
            None
        }
    }

    fn advance_n(&mut self, n: usize) {
        for _ in 0..n {
            if self.advance().is_none() {
                break;
            }
        }
    }

    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek(0) {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
}

pub struct HtmlTokenizerIter<'a> {
    tokenizer: HtmlTokenizer<'a>,
}

impl<'a> HtmlTokenizerIter<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            tokenizer: HtmlTokenizer::new(input),
        }
    }
}

impl Iterator for HtmlTokenizerIter<'_> {
    type Item = Result<HtmlToken, TokenizeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokenizer.next_token()
    }
}

impl<'a> HtmlTokenizer<'a> {
    pub fn iter(&self) -> HtmlTokenizerIter<'a> {
        HtmlTokenizerIter {
            tokenizer: HtmlTokenizer::new(self.input),
        }
    }

    pub fn next_token(&mut self) -> Option<Result<HtmlToken, TokenizeError>> {
        self.skip_whitespace();

        if self.is_eof() {
            return None;
        }

        if self.peek(0) == Some('<') {
            self.advance();
            return Some(self.parse_tag());
        }

        self.parse_text()
    }

    fn parse_tag(&mut self) -> Result<HtmlToken, TokenizeError> {
        if self.peek(0) == Some('!') {
            self.advance();
            return self.parse_special_tag();
        }

        if self.peek(0) == Some('/') {
            self.advance();
            return self.parse_end_tag();
        }

        self.parse_start_tag()
    }

    fn parse_special_tag(&mut self) -> Result<HtmlToken, TokenizeError> {
        if self.peek(0) == Some('-') {
            if self.peek(1) == Some('-') {
                self.advance_n(2);
                self.parse_comment()
            } else {
                Err(TokenizeError::MalformedComment)
            }
        } else {
            self.parse_doctype()
        }
    }

    fn parse_doctype(&mut self) -> Result<HtmlToken, TokenizeError> {
        let mut doctype = String::new();

        while let Some(c) = self.peek(0) {
            if c == '>' {
                self.advance();
                return Ok(HtmlToken::Doctype(doctype));
            }
            if let Some(ch) = self.advance() {
                doctype.push(ch);
            } else {
                return Err(TokenizeError::UnexpectedEOF);
            }
        }

        Err(TokenizeError::UnexpectedEOF)
    }

    fn parse_comment(&mut self) -> Result<HtmlToken, TokenizeError> {
        let mut comment = String::new();
        let mut prev_dash_count = 0;

        loop {
            if let Some(c) = self.advance() {
                if prev_dash_count >= 2 && c == '>' {
                    if comment.ends_with("--") {
                        let new_len = comment.len().saturating_sub(2);
                        comment.truncate(new_len);
                    }
                    return Ok(HtmlToken::Comment(comment));
                }
                comment.push(c);
                if c == '-' {
                    prev_dash_count += 1;
                } else {
                    prev_dash_count = 0;
                }
            } else {
                return Err(TokenizeError::MalformedComment);
            }
        }
    }

    fn parse_start_tag(&mut self) -> Result<HtmlToken, TokenizeError> {
        let Some(name) = self.parse_tag_name() else {
            return Err(TokenizeError::InvalidTag);
        };

        self.skip_whitespace();

        let attributes = self.parse_attributes()?;

        self.skip_whitespace();

        let self_closing = if self.peek(0) == Some('/') {
            self.advance();
            if self.peek(0) == Some('>') {
                self.advance();
                true
            } else {
                return Err(TokenizeError::InvalidTag);
            }
        } else if self.peek(0) == Some('>') {
            self.advance();
            false
        } else {
            return Err(TokenizeError::InvalidTag);
        };

        Ok(HtmlToken::StartTag {
            name,
            attributes,
            self_closing,
        })
    }

    fn parse_end_tag(&mut self) -> Result<HtmlToken, TokenizeError> {
        let Some(name) = self.parse_tag_name() else {
            return Err(TokenizeError::InvalidTag);
        };

        self.skip_whitespace();

        if self.peek(0) == Some('>') {
            self.advance();
            Ok(HtmlToken::EndTag { name })
        } else {
            Err(TokenizeError::InvalidTag)
        }
    }

    fn parse_tag_name(&mut self) -> Option<String> {
        let mut name = String::new();

        while let Some(c) = self.peek(0) {
            if c.is_whitespace() || c == '>' || c == '/' {
                break;
            }
            if let Some(ch) = self.advance() {
                name.push(ch);
            } else {
                break;
            }
        }

        if name.is_empty() { None } else { Some(name) }
    }

    fn parse_text(&mut self) -> Option<Result<HtmlToken, TokenizeError>> {
        let mut text = String::new();

        while let Some(c) = self.peek(0) {
            if c == '<' {
                break;
            }
            if let Some(ch) = self.advance() {
                text.push(ch);
            } else {
                break;
            }
        }

        if text.is_empty() {
            self.next_token()
        } else {
            Some(Ok(HtmlToken::Text(text)))
        }
    }

    fn parse_attributes(&mut self) -> Result<Vec<(String, String)>, TokenizeError> {
        let mut attributes = Vec::new();

        loop {
            self.skip_whitespace();

            if self.peek(0) == Some('>') {
                break;
            }

            if self.peek(0) == Some('/') {
                break;
            }

            match self.parse_attribute() {
                Some(Ok(attr)) => attributes.push(attr),
                Some(Err(e)) => return Err(e),
                None => break,
            }
        }

        Ok(attributes)
    }

    fn parse_attribute(&mut self) -> Option<Result<(String, String), TokenizeError>> {
        let name = self.parse_attribute_name()?;

        self.skip_whitespace();

        if self.peek(0) == Some('=') {
            self.advance();
            self.skip_whitespace();

            match self.parse_attribute_value() {
                Ok(value) => Some(Ok((name, value))),
                Err(e) => Some(Err(e)),
            }
        } else {
            Some(Ok((name, String::new())))
        }
    }

    fn parse_attribute_name(&mut self) -> Option<String> {
        let mut name = String::new();

        while let Some(c) = self.peek(0) {
            if c.is_whitespace() || c == '=' || c == '>' || c == '/' {
                break;
            }
            if let Some(ch) = self.advance() {
                name.push(ch);
            } else {
                break;
            }
        }

        if name.is_empty() { None } else { Some(name) }
    }

    fn parse_attribute_value(&mut self) -> Result<String, TokenizeError> {
        let quote = match self.peek(0) {
            Some('"' | '\'') => match self.advance() {
                Some(q) => Some(q),
                None => return Err(TokenizeError::InvalidAttribute),
            },
            _ => None,
        };

        let mut value = String::new();

        if let Some(quote) = quote {
            loop {
                match self.advance() {
                    Some(c) => {
                        if c == quote {
                            return Ok(value);
                        }
                        value.push(c);
                    }
                    None => return Err(TokenizeError::InvalidAttribute),
                }
            }
        } else {
            while let Some(c) = self.peek(0) {
                if c.is_whitespace() || c == '>' || c == '/' {
                    break;
                }
                if let Some(ch) = self.advance() {
                    value.push(ch);
                } else {
                    return Err(TokenizeError::InvalidAttribute);
                }
            }
            Ok(value)
        }
    }
}

impl<'a> IntoIterator for &'a HtmlTokenizer<'a> {
    type Item = Result<HtmlToken, TokenizeError>;
    type IntoIter = HtmlTokenizerIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_start_tag() {
        let mut tokenizer = HtmlTokenizer::new("<div>");
        let token = tokenizer.next_token().unwrap().unwrap();

        match token {
            HtmlToken::StartTag {
                name,
                attributes,
                self_closing,
            } => {
                assert_eq!(name, "div");
                assert!(attributes.is_empty());
                assert_eq!(self_closing, false);
            }
            _ => panic!("Expected StartTag"),
        }
    }

    #[test]
    fn test_parse_start_tag_with_attributes() {
        let mut tokenizer = HtmlTokenizer::new("<div class=\"foo\" id=\"bar\">");
        let token = tokenizer.next_token().unwrap().unwrap();

        match token {
            HtmlToken::StartTag {
                name,
                attributes,
                self_closing,
            } => {
                assert_eq!(name, "div");
                assert_eq!(attributes.len(), 2);
                assert_eq!(attributes[0], ("class".to_string(), "foo".to_string()));
                assert_eq!(attributes[1], ("id".to_string(), "bar".to_string()));
                assert_eq!(self_closing, false);
            }
            _ => panic!("Expected StartTag"),
        }
    }

    #[test]
    fn test_parse_self_closing_tag() {
        let mut tokenizer = HtmlTokenizer::new("<br />");
        let token = tokenizer.next_token().unwrap().unwrap();

        match token {
            HtmlToken::StartTag {
                name,
                attributes,
                self_closing,
            } => {
                assert_eq!(name, "br");
                assert!(attributes.is_empty());
                assert_eq!(self_closing, true);
            }
            _ => panic!("Expected StartTag"),
        }
    }

    #[test]
    fn test_parse_end_tag() {
        let mut tokenizer = HtmlTokenizer::new("</div>");
        let token = tokenizer.next_token().unwrap().unwrap();

        match token {
            HtmlToken::EndTag { name } => {
                assert_eq!(name, "div");
            }
            _ => panic!("Expected EndTag"),
        }
    }

    #[test]
    fn test_parse_text() {
        let mut tokenizer = HtmlTokenizer::new("hello world");
        let token = tokenizer.next_token().unwrap().unwrap();

        match token {
            HtmlToken::Text(text) => {
                assert_eq!(text, "hello world");
            }
            _ => panic!("Expected Text"),
        }
    }

    #[test]
    fn test_parse_comment() {
        let mut tokenizer = HtmlTokenizer::new("<!-- comment -->");
        let token = tokenizer.next_token().unwrap().unwrap();

        match token {
            HtmlToken::Comment(comment) => {
                assert_eq!(comment, " comment ");
            }
            _ => panic!("Expected Comment"),
        }
    }

    #[test]
    fn test_parse_doctype() {
        let mut tokenizer = HtmlTokenizer::new("<!DOCTYPE html>");
        let token = tokenizer.next_token().unwrap().unwrap();

        match token {
            HtmlToken::Doctype(doctype) => {
                assert_eq!(doctype, "DOCTYPE html");
            }
            _ => panic!("Expected Doctype"),
        }
    }

    #[test]
    fn test_parse_attribute_with_single_quotes() {
        let mut tokenizer = HtmlTokenizer::new("<a href='example.com'>");
        let token = tokenizer.next_token().unwrap().unwrap();

        match token {
            HtmlToken::StartTag { attributes, .. } => {
                assert_eq!(attributes.len(), 1);
                assert_eq!(
                    attributes[0],
                    ("href".to_string(), "example.com".to_string())
                );
            }
            _ => panic!("Expected StartTag"),
        }
    }

    #[test]
    fn test_parse_attribute_without_quotes() {
        let mut tokenizer = HtmlTokenizer::new("<input type=text>");
        let token = tokenizer.next_token().unwrap().unwrap();

        match token {
            HtmlToken::StartTag { attributes, .. } => {
                assert_eq!(attributes.len(), 1);
                assert_eq!(attributes[0], ("type".to_string(), "text".to_string()));
            }
            _ => panic!("Expected StartTag"),
        }
    }

    #[test]
    fn test_parse_empty_attribute() {
        let mut tokenizer = HtmlTokenizer::new("<button disabled>");
        let token = tokenizer.next_token().unwrap().unwrap();

        match token {
            HtmlToken::StartTag { attributes, .. } => {
                assert_eq!(attributes.len(), 1);
                assert_eq!(attributes[0].0, "disabled");
                assert_eq!(attributes[0].1, "");
            }
            _ => panic!("Expected StartTag"),
        }
    }

    #[test]
    fn test_parse_nested_tags() {
        let input = "<div><span><a>link</a></span></div>";
        let tokenizer = HtmlTokenizer::new(input);
        let tokens: Vec<_> = tokenizer.iter().map(|t| t.unwrap()).collect();

        assert_eq!(tokens.len(), 7);

        assert_eq!(
            tokens[0],
            HtmlToken::StartTag {
                name: "div".to_string(),
                attributes: vec![],
                self_closing: false,
            }
        );

        assert_eq!(
            tokens[1],
            HtmlToken::StartTag {
                name: "span".to_string(),
                attributes: vec![],
                self_closing: false,
            }
        );

        assert_eq!(
            tokens[2],
            HtmlToken::StartTag {
                name: "a".to_string(),
                attributes: vec![],
                self_closing: false,
            }
        );

        assert_eq!(tokens[3], HtmlToken::Text("link".to_string()));

        assert_eq!(
            tokens[4],
            HtmlToken::EndTag {
                name: "a".to_string()
            }
        );

        assert_eq!(
            tokens[5],
            HtmlToken::EndTag {
                name: "span".to_string()
            }
        );

        assert_eq!(
            tokens[6],
            HtmlToken::EndTag {
                name: "div".to_string()
            }
        );
    }

    #[test]
    fn test_parse_mixed_attributes() {
        let mut tokenizer =
            HtmlTokenizer::new("<img src='test.jpg' alt=\"test\" width=100 height=\"200\"/>");
        let token = tokenizer.next_token().unwrap().unwrap();

        match token {
            HtmlToken::StartTag {
                name,
                attributes,
                self_closing,
            } => {
                assert_eq!(name, "img");
                assert_eq!(self_closing, true);
                assert_eq!(attributes.len(), 4);
                assert_eq!(attributes[0], ("src".to_string(), "test.jpg".to_string()));
                assert_eq!(attributes[1], ("alt".to_string(), "test".to_string()));
                assert_eq!(attributes[2], ("width".to_string(), "100".to_string()));
                assert_eq!(attributes[3], ("height".to_string(), "200".to_string()));
            }
            _ => panic!("Expected StartTag"),
        }
    }

    #[test]
    fn test_empty_input() {
        let tokenizer = HtmlTokenizer::new("");
        let tokens: Vec<_> = tokenizer.iter().collect();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_only_whitespace() {
        let tokenizer = HtmlTokenizer::new("   \t\n   ");
        let tokens: Vec<_> = tokenizer.iter().collect();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_error_invalid_tag() {
        let mut tokenizer = HtmlTokenizer::new("<>");
        match tokenizer.next_token() {
            Some(Err(TokenizeError::InvalidTag)) => {}
            _ => panic!("Expected InvalidTag error"),
        }
    }

    #[test]
    fn test_tokenize_error_malformed_comment() {
        let mut tokenizer = HtmlTokenizer::new("<!- comment ->");
        match tokenizer.next_token() {
            Some(Err(TokenizeError::MalformedComment)) => {}
            _ => panic!("Expected MalformedComment error"),
        }
    }
}
