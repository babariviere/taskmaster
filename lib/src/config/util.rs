use serde::{Deserialize, Deserializer, Serializer};
use serde::de;
use std::fmt;

/// Apply str radix
pub trait IntStrRadix: Sized {
    /// String to number
    fn str_radix(s: &str, radix: u32) -> Result<Self, ::std::num::ParseIntError>;
}

macro_rules! impl_str_radix {
    ($($nb:ident)*) => {
        $(
            impl IntStrRadix for $nb {
                fn str_radix(s: &str, radix: u32) -> Result<Self, ::std::num::ParseIntError> {
                    $nb::from_str_radix(s, radix)
                }
            }
        )*
    }
}

impl_str_radix!(u8 u16 u32 u64 usize i8 i16 i32 i64 isize);

/// Human readable number
pub trait HumanNumber: Sized {
    /// Transform to human format
    fn to_human(&self) -> String;
    /// Transform from human format
    fn from_human(&str) -> Result<Self, String>;
}

fn letter_to_pow(c: char) -> Option<u32> {
    match c.to_lowercase().to_string().as_str() {
        "k" => Some(1),
        "m" => Some(2),
        "g" => Some(3),
        "t" => Some(4),
        "p" => Some(5),
        "e" => Some(6),
        "z" => Some(7),
        "y" => Some(8),
        _ => None,
    }
}

fn pow_to_letter(pow: u32) -> Option<char> {
    match pow {
        1 => Some('k'),
        2 => Some('M'),
        3 => Some('G'),
        4 => Some('T'),
        5 => Some('P'),
        6 => Some('E'),
        7 => Some('Z'),
        8 => Some('Y'),
        _ => None,
    }
}

enum HumanParsingState {
    Number,
    Unit,
    Prefix,
}

macro_rules! impl_hum_nb {
    ($($nb:ty)*) => {
        $(
            impl HumanNumber for $nb {
                fn to_human(&self) -> String {
                    let mut pow = 0;
                    let mut num = *self;
                    while num >= 10000 {
                        pow += 1;
                        num /= 1000;
                    }
                    match pow_to_letter(pow) {
                        Some(p) => format!("{}{}B", num, p),
                        None => format!("{}B", num),
                    }
                }

                fn from_human(s: &str) -> Result<Self, String> {
                    let mut num = String::new();
                    let mut state = HumanParsingState::Number;
                    let mut pow = 0;
                    let mut unit: $nb = 1000;
                    for c in s.chars() {
                        match state {
                            HumanParsingState::Number => {
                                if c.is_numeric() {
                                    num.push(c);
                                } else if c == 'B' {
                                    break;
                                } else {
                                    pow = match letter_to_pow(c) {
                                        Some(p) => p,
                                        None => {
                                            return Err(format!("invalid metric"));
                                        }
                                    };
                                    state = HumanParsingState::Unit;
                                }
                            }
                            HumanParsingState::Unit => {
                                if c == 'i' {
                                    state = HumanParsingState::Prefix;
                                    unit = 1024;
                                } else if c == 'B' {
                                    break;
                                } else {
                                    return Err(format!("unexpected char {}", c));
                                }
                            }
                            HumanParsingState::Prefix => {
                                if c == 'B' {
                                    break;
                                } else {
                                    return Err(format!("unexpected char {}", c));
                                }
                            }
                        }
                    }
                    if num.len() == 0 {
                        return Err(format!("invalid number"));
                    }
                    let num = num.parse().unwrap_or(0);
                    Ok(num * unit.pow(pow))
                }
            }
        )*
    }
}

impl_hum_nb!(u16 u32 u64 usize i16 i32 i64 isize);

/// Deserialize octal number
pub fn deserialize_octal<'de, D, T: IntStrRadix>(de: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(de).and_then(|s| {
        match &s[0..2] {
            "0o" => T::str_radix(&s[2..], 8),
            _ => T::str_radix(&s, 8),
        }.map_err(|e| de::Error::custom(e))
    })
}

/// Serialize octal number
pub fn serialize_octal<S, T: fmt::Octal>(val: &T, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_str(&format!("{:#o}", val))
}

/// Deserialize human number
pub fn deserialize_human<'de, D, T: HumanNumber>(de: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(de).and_then(|s| T::from_human(&s).map_err(|e| de::Error::custom(e)))
}

/// Serialize human number
pub fn serialize_human<S, T: HumanNumber>(val: &T, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_str(&val.to_human())
}
