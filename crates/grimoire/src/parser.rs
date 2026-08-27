use core::fmt;

use crate::{
    Address, Block, Connection, CoreGraph, Description, ExtensionParameter, ExtensionValue,
    FiniteNumber, Group, Namespace, Port, Value, Version,
};

const PROTOTYPE_NAMESPACE_ROOT: &str = "https://github.com/goldenwitch/grimoire/extension";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    pub offset: usize,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
enum TokenKind {
    Word(String),
    String(String),
    Arrow,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    Colon,
    Comma,
    Equal,
    Semicolon,
    End,
}

#[derive(Clone, Debug, PartialEq)]
struct Token {
    kind: TokenKind,
    offset: usize,
    end: usize,
}

struct Lexer<'source> {
    source: &'source str,
    offset: usize,
}

struct Parser<'source> {
    lexer: Lexer<'source>,
    current: Token,
}

pub fn parse_description(source: &str) -> Result<Description, ParseError> {
    Parser::new(source)?.parse_description()
}

impl<'source> Lexer<'source> {
    fn new(source: &'source str) -> Self {
        Self { source, offset: 0 }
    }

    fn next(&mut self) -> Result<Token, ParseError> {
        self.skip_ignored();
        let offset = self.offset;
        let Some(character) = self.peek() else {
            return Ok(Token {
                kind: TokenKind::End,
                offset,
                end: offset,
            });
        };
        let kind = match character {
            '{' => {
                self.advance();
                TokenKind::LeftBrace
            }
            '}' => {
                self.advance();
                TokenKind::RightBrace
            }
            '[' => {
                self.advance();
                TokenKind::LeftBracket
            }
            ']' => {
                self.advance();
                TokenKind::RightBracket
            }
            '(' => {
                self.advance();
                TokenKind::LeftParen
            }
            ')' => {
                self.advance();
                TokenKind::RightParen
            }
            ':' => {
                self.advance();
                TokenKind::Colon
            }
            ',' => {
                self.advance();
                TokenKind::Comma
            }
            '=' => {
                self.advance();
                TokenKind::Equal
            }
            ';' => {
                self.advance();
                TokenKind::Semicolon
            }
            '-' if self.source[self.offset..].starts_with("->") => {
                self.offset += 2;
                TokenKind::Arrow
            }
            '"' => TokenKind::String(self.read_string()?),
            _ => TokenKind::Word(self.read_word()),
        };
        Ok(Token {
            kind,
            offset,
            end: self.offset,
        })
    }

    fn skip_ignored(&mut self) {
        loop {
            while self.peek().is_some_and(char::is_whitespace) {
                self.advance();
            }
            if self.peek() != Some('#') {
                return;
            }
            while self.peek().is_some_and(|character| character != '\n') {
                self.advance();
            }
        }
    }

    fn read_string(&mut self) -> Result<String, ParseError> {
        let start = self.offset;
        self.advance();
        let mut value = String::new();
        loop {
            let Some(character) = self.peek() else {
                return Err(ParseError {
                    offset: start,
                    message: "unterminated string".to_owned(),
                });
            };
            self.advance();
            match character {
                '"' => return Ok(value),
                '\\' => value.push(self.read_escape(start)?),
                character if character.is_control() => {
                    return Err(ParseError {
                        offset: self.offset.saturating_sub(character.len_utf8()),
                        message: "control character in string".to_owned(),
                    });
                }
                character => value.push(character),
            }
        }
    }

    fn read_escape(&mut self, start: usize) -> Result<char, ParseError> {
        let Some(escape) = self.peek() else {
            return Err(ParseError {
                offset: start,
                message: "unterminated escape".to_owned(),
            });
        };
        self.advance();
        match escape {
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            '/' => Ok('/'),
            'b' => Ok('\u{0008}'),
            'f' => Ok('\u{000c}'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            'u' => {
                let digits = self.take_hex_digits(4)?;
                let code = u32::from_str_radix(&digits, 16).map_err(|_| ParseError {
                    offset: self.offset.saturating_sub(4),
                    message: "invalid unicode escape".to_owned(),
                })?;
                char::from_u32(code).ok_or_else(|| ParseError {
                    offset: self.offset.saturating_sub(4),
                    message: "invalid unicode scalar".to_owned(),
                })
            }
            _ => Err(ParseError {
                offset: self.offset.saturating_sub(1),
                message: "invalid string escape".to_owned(),
            }),
        }
    }

    fn take_hex_digits(&mut self, count: usize) -> Result<String, ParseError> {
        let start = self.offset;
        let mut digits = String::new();
        for _ in 0..count {
            let Some(character) = self.peek() else {
                return Err(ParseError {
                    offset: start,
                    message: "short unicode escape".to_owned(),
                });
            };
            if !character.is_ascii_hexdigit() {
                return Err(ParseError {
                    offset: self.offset,
                    message: "invalid unicode escape".to_owned(),
                });
            }
            digits.push(character);
            self.advance();
        }
        Ok(digits)
    }

    fn read_word(&mut self) -> String {
        let start = self.offset;
        while self.peek().is_some_and(|character| {
            !character.is_whitespace()
                && !matches!(
                    character,
                    '{' | '}' | '[' | ']' | '(' | ')' | ':' | ',' | '=' | ';' | '"' | '#'
                )
        }) {
            if self.source[self.offset..].starts_with("->") {
                break;
            }
            self.advance();
        }
        self.source[start..self.offset].to_owned()
    }

    fn peek(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    fn advance(&mut self) {
        if let Some(character) = self.peek() {
            self.offset += character.len_utf8();
        }
    }
}

impl<'source> Parser<'source> {
    fn new(source: &'source str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(source);
        let current = lexer.next()?;
        Ok(Self { lexer, current })
    }

    fn parse_description(mut self) -> Result<Description, ParseError> {
        self.expect_word("grimoire")?;
        let _grammar_version = self.parse_version()?;
        self.expect_word("description")?;
        let address = self.parse_address()?;
        let label = self.optional_string()?;
        self.expect(TokenKind::LeftBrace)?;
        self.expect_word("core-spec")?;
        let core_spec = self.parse_version()?;
        self.expect(TokenKind::Semicolon)?;
        let extensions = self.optional_extensions()?;
        self.expect_word("core")?;
        let core = self.parse_core()?;
        self.expect(TokenKind::RightBrace)?;
        self.expect_end()?;
        let description = Description {
            address,
            label,
            core_spec,
            core,
            extensions,
            layers: Vec::new(),
        };
        ensure_unique_addresses(&description)?;
        Ok(description)
    }

    fn parse_core(&mut self) -> Result<CoreGraph, ParseError> {
        self.expect(TokenKind::LeftBrace)?;
        let mut core = CoreGraph::default();
        while self.current.kind != TokenKind::RightBrace {
            let keyword = self.take_word()?;
            match keyword.as_str() {
                "block" => {
                    let block = self.parse_block()?;
                    if core.blocks.insert(block.address.clone(), block).is_some() {
                        return self.error("duplicate block address");
                    }
                }
                "connection" => {
                    let connection = self.parse_connection()?;
                    if core
                        .connections
                        .insert(connection.address.clone(), connection)
                        .is_some()
                    {
                        return self.error("duplicate connection address");
                    }
                }
                "group" => {
                    let group = self.parse_group()?;
                    if core.groups.insert(group.address.clone(), group).is_some() {
                        return self.error("duplicate group address");
                    }
                }
                keyword => return self.error(format!("unexpected core keyword `{keyword}`")),
            }
        }
        self.expect(TokenKind::RightBrace)?;
        Ok(core)
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let address = self.parse_address()?;
        let Some(name) = self.take_string()? else {
            return self.error("block requires a human name");
        };
        self.expect(TokenKind::LeftBrace)?;
        let mut ports = std::collections::BTreeMap::new();
        while self.is_word("port") {
            self.expect_word("port")?;
            let port = self.parse_port()?;
            if ports.insert(port.address.clone(), port).is_some() {
                return self.error("duplicate port address");
            }
        }
        let extensions = self.optional_extensions()?;
        self.expect(TokenKind::RightBrace)?;
        Ok(Block {
            address,
            name,
            ports,
            extensions,
        })
    }

    fn parse_port(&mut self) -> Result<Port, ParseError> {
        let address = self.parse_address()?;
        let label = self.optional_string()?;
        let extensions = self.optional_extensions()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Port {
            address,
            label,
            extensions,
        })
    }

    fn parse_connection(&mut self) -> Result<Connection, ParseError> {
        let address = self.parse_address()?;
        let source = self.parse_address()?;
        self.expect(TokenKind::Arrow)?;
        let destination = self.parse_address()?;
        let extensions = self.optional_extensions()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Connection {
            address,
            label: None,
            source,
            destination,
            extensions,
        })
    }

    fn parse_group(&mut self) -> Result<Group, ParseError> {
        let address = self.parse_address()?;
        let label = self.optional_string()?;
        self.expect(TokenKind::LeftBrace)?;
        let mut members = Vec::new();
        if self.current.kind != TokenKind::RightBrace && !self.is_word("extensions") {
            members.push(self.parse_address()?);
            while self.current.kind == TokenKind::Comma {
                self.advance()?;
                members.push(self.parse_address()?);
            }
            self.expect(TokenKind::Semicolon)?;
        }
        let extensions = self.optional_extensions()?;
        self.expect(TokenKind::RightBrace)?;
        Ok(Group {
            address,
            label,
            members,
            extensions,
        })
    }

    fn optional_extensions(&mut self) -> Result<Vec<ExtensionParameter>, ParseError> {
        if self.is_word("extensions") {
            self.parse_extensions()
        } else {
            Ok(Vec::new())
        }
    }

    fn parse_extensions(&mut self) -> Result<Vec<ExtensionParameter>, ParseError> {
        self.expect_word("extensions")?;
        self.expect(TokenKind::LeftBrace)?;
        let mut extensions = Vec::new();
        while self.current.kind != TokenKind::RightBrace {
            let start = self.current.offset;
            self.expect_word("extension")?;
            let namespace = self.parse_namespace()?;
            let name = self.take_word()?;
            self.expect_word("schema")?;
            let schema = self.take_word()?;
            let version = self.parse_at_version()?;
            self.expect(TokenKind::Equal)?;
            let parsed_value = self.parse_value()?;
            let end = self.current.end;
            self.expect(TokenKind::Semicolon)?;
            let value = if is_known_namespace(&namespace) {
                ExtensionValue::Known(parsed_value)
            } else {
                ExtensionValue::Opaque(self.lexer.source.as_bytes()[start..end].to_vec())
            };
            extensions.push(ExtensionParameter {
                namespace,
                name,
                schema,
                version,
                value,
            });
        }
        self.expect(TokenKind::RightBrace)?;
        Ok(extensions)
    }

    fn parse_namespace(&mut self) -> Result<Namespace, ParseError> {
        let token = self.current.clone();
        let Some(value) = self.take_string()? else {
            return self.error("extension namespace requires a string");
        };
        Namespace::parse(&value).map_err(|error| ParseError {
            offset: token.offset,
            message: error.to_string(),
        })
    }

    fn parse_at_version(&mut self) -> Result<Version, ParseError> {
        let token = self.current.clone();
        let value = self.take_word()?;
        let Some(value) = value.strip_prefix('@') else {
            return Err(ParseError {
                offset: token.offset,
                message: "schema version requires `@`".to_owned(),
            });
        };
        Version::parse(value).map_err(|error| ParseError {
            offset: token.offset,
            message: error.to_string(),
        })
    }

    fn parse_value(&mut self) -> Result<Value, ParseError> {
        match self.current.kind.clone() {
            TokenKind::String(_) => {
                let Some(value) = self.take_string()? else {
                    return self.error("expected string value");
                };
                Ok(Value::Text(value))
            }
            TokenKind::Word(_) => self.parse_word_value(),
            TokenKind::LeftBrace => self.parse_product_value(),
            TokenKind::LeftBracket => self.parse_sequence_value(),
            _ => self.error("expected a value"),
        }
    }

    fn parse_word_value(&mut self) -> Result<Value, ParseError> {
        let token = self.current.clone();
        let word = self.take_word()?;
        match word.as_str() {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            "absent" => Ok(Value::Absent),
            "present" => {
                self.expect(TokenKind::LeftParen)?;
                let value = self.parse_value()?;
                self.expect(TokenKind::RightParen)?;
                Ok(Value::Present(Box::new(value)))
            }
            "ref" => {
                self.expect(TokenKind::LeftParen)?;
                let address = self.parse_address()?;
                self.expect(TokenKind::RightParen)?;
                Ok(Value::AddressReference(address))
            }
            _ if self.current.kind == TokenKind::LeftParen => {
                self.expect(TokenKind::LeftParen)?;
                let value = self.parse_value()?;
                self.expect(TokenKind::RightParen)?;
                Ok(Value::Tagged {
                    tag: word,
                    value: Box::new(value),
                })
            }
            _ => parse_scalar_word(&word).map_err(|message| ParseError {
                offset: token.offset,
                message,
            }),
        }
    }

    fn parse_product_value(&mut self) -> Result<Value, ParseError> {
        self.expect(TokenKind::LeftBrace)?;
        let mut fields = std::collections::BTreeMap::new();
        while self.current.kind != TokenKind::RightBrace {
            let name = self.take_word()?;
            self.expect(TokenKind::Colon)?;
            let value = self.parse_value()?;
            if fields.insert(name.clone(), value).is_some() {
                return self.error(format!("duplicate product field `{name}`"));
            }
            if self.current.kind == TokenKind::Comma {
                self.advance()?;
                if self.current.kind == TokenKind::RightBrace {
                    return self.error("trailing product comma");
                }
            } else {
                break;
            }
        }
        self.expect(TokenKind::RightBrace)?;
        Ok(Value::Product(fields))
    }

    fn parse_sequence_value(&mut self) -> Result<Value, ParseError> {
        self.expect(TokenKind::LeftBracket)?;
        let mut values = Vec::new();
        while self.current.kind != TokenKind::RightBracket {
            values.push(self.parse_value()?);
            if self.current.kind == TokenKind::Comma {
                self.advance()?;
                if self.current.kind == TokenKind::RightBracket {
                    return self.error("trailing sequence comma");
                }
            } else {
                break;
            }
        }
        self.expect(TokenKind::RightBracket)?;
        Ok(Value::Sequence(values))
    }

    fn parse_address(&mut self) -> Result<Address, ParseError> {
        let token = self.current.clone();
        let value = self.take_word()?;
        Address::parse(&value).map_err(|error| ParseError {
            offset: token.offset,
            message: error.to_string(),
        })
    }

    fn parse_version(&mut self) -> Result<Version, ParseError> {
        let token = self.current.clone();
        let value = self.take_word()?;
        Version::parse(&value).map_err(|error| ParseError {
            offset: token.offset,
            message: error.to_string(),
        })
    }

    fn optional_string(&mut self) -> Result<Option<String>, ParseError> {
        self.take_string()
    }

    fn take_string(&mut self) -> Result<Option<String>, ParseError> {
        let TokenKind::String(value) = &self.current.kind else {
            return Ok(None);
        };
        let value = value.clone();
        self.advance()?;
        Ok(Some(value))
    }

    fn expect_word(&mut self, expected: &str) -> Result<(), ParseError> {
        let actual = self.take_word()?;
        if actual == expected {
            Ok(())
        } else {
            self.error(format!("expected `{expected}`, got `{actual}`"))
        }
    }

    fn take_word(&mut self) -> Result<String, ParseError> {
        let TokenKind::Word(value) = &self.current.kind else {
            return self.error("expected a word");
        };
        let value = value.clone();
        self.advance()?;
        Ok(value)
    }

    fn is_word(&self, expected: &str) -> bool {
        matches!(&self.current.kind, TokenKind::Word(value) if value == expected)
    }

    fn expect(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        if self.current.kind == expected {
            self.advance()
        } else {
            self.error(format!(
                "expected {}, got {}",
                token_name(&expected),
                token_name(&self.current.kind)
            ))
        }
    }

    fn expect_end(&self) -> Result<(), ParseError> {
        if self.current.kind == TokenKind::End {
            Ok(())
        } else {
            Err(ParseError {
                offset: self.current.offset,
                message: "trailing input".to_owned(),
            })
        }
    }

    fn advance(&mut self) -> Result<(), ParseError> {
        self.current = self.lexer.next()?;
        Ok(())
    }

    fn error<T>(&self, message: impl Into<String>) -> Result<T, ParseError> {
        Err(ParseError {
            offset: self.current.offset,
            message: message.into(),
        })
    }
}

fn parse_scalar_word(word: &str) -> Result<Value, String> {
    if let Ok(value) = word.parse::<u64>() {
        if value > 0 {
            return Ok(Value::PositiveInteger(value));
        }
        return FiniteNumber::new(0.0)
            .map(Value::Number)
            .ok_or_else(|| "invalid finite number".to_owned());
    }
    if let Ok(value) = word.parse::<f64>() {
        return FiniteNumber::new(value)
            .map(Value::Number)
            .ok_or_else(|| "number must be finite".to_owned());
    }
    if is_identifier(word) {
        Ok(Value::Enum(word.to_owned()))
    } else {
        Err(format!("invalid scalar value `{word}`"))
    }
}

fn is_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
}

fn is_known_namespace(namespace: &Namespace) -> bool {
    namespace.as_str() == PROTOTYPE_NAMESPACE_ROOT
        || namespace
            .as_str()
            .strip_prefix(PROTOTYPE_NAMESPACE_ROOT)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn ensure_unique_addresses(description: &Description) -> Result<(), ParseError> {
    let mut addresses = std::collections::BTreeSet::new();
    for address in description.addresses() {
        if !addresses.insert(address) {
            return Err(ParseError {
                offset: 0,
                message: format!("duplicate address `{address}`"),
            });
        }
    }
    Ok(())
}

fn token_name(token: &TokenKind) -> &'static str {
    match token {
        TokenKind::Word(_) => "word",
        TokenKind::String(_) => "string",
        TokenKind::Arrow => "`->`",
        TokenKind::LeftBrace => "`{`",
        TokenKind::RightBrace => "`}`",
        TokenKind::LeftBracket => "`[`",
        TokenKind::RightBracket => "`]`",
        TokenKind::LeftParen => "`(`",
        TokenKind::RightParen => "`)`",
        TokenKind::Colon => "`:`",
        TokenKind::Comma => "`,`",
        TokenKind::Equal => "`=`",
        TokenKind::Semicolon => "`;`",
        TokenKind::End => "end of input",
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "at byte {}: {}", self.offset, self.message)
    }
}

impl std::error::Error for ParseError {}
