//! Module to parse configuration

use parser::Parser;
use super::*;

/// Ini value
#[derive(Clone, Debug, PartialEq)]
pub enum IniValue {
    /// Ini key
    Key(String, String),
    /// Ini section
    Section(String, Vec<IniValue>),
}

/// Ini parser
pub struct IniParser<'a> {
    parser: Parser<'a>,
}

impl<'a> IniParser<'a> {
    /// Create parser
    pub fn new(buf: &'a str) -> IniParser {
        IniParser {
            parser: Parser::new(buf),
        }
    }

    /// Parse everything
    pub fn parse(mut self) -> Vec<IniValue> {
        let mut values = Vec::new();
        while let Some(val) = self.parse_value() {
            values.push(val);
        }
        values
    }

    /// Parse next value
    pub fn parse_value(&mut self) -> Option<IniValue> {
        while let Some(c) = self.parser.next_char() {
            if !c.is_whitespace() && c != ';' {
                break;
            }
            self.parser.eat_line();
        }
        match self.parser.next_char() {
            Some('[') => Some(self.parse_section()),
            Some(_) => Some(self.parse_key()),
            _ => None,
        }
    }

    /// Parse key
    pub fn parse_key(&mut self) -> IniValue {
        let name = self.parser.eat_while(|c| c != '=').trim().to_string();
        self.parser.eat_char();
        let value = self.parser
            .eat_while_esc(|c| c != ';' && c != '\n', '\\')
            .trim()
            .to_string();
        IniValue::Key(name, value)
    }

    /// Parse section
    pub fn parse_section(&mut self) -> IniValue {
        assert_eq!(self.parser.eat_char(), Some('['));
        let sec = self.parser.eat_while(|c| c != ']');
        assert_eq!(self.parser.eat_char(), Some(']'));
        self.parser.eat_line();
        let mut keys = Vec::new();
        loop {
            while let Some(c) = self.parser.next_char() {
                if !c.is_whitespace() && c != ';' {
                    break;
                }
                self.parser.eat_line();
            }
            match self.parser.next_char() {
                Some('[') | None => break,
                _ => {}
            }
            keys.push(self.parse_key());
        }
        IniValue::Section(sec, keys)
    }
}
