use crate::params::parsers::{FuzzGenerator, Generator};
use anyhow::Error;
use cucumber::Parameter;
use either::Either;
use rand::Rng;
use regex::Regex;
use std::str::FromStr;

mod parsers;

#[derive(Parameter, Debug, Clone)]
#[param(regex = r#"(?:[^"\\]|\\.)*"#, name = "cbor")]
pub struct Cbor {
    parts: Vec<Either<String, Generator>>,
}

impl Cbor {
    pub fn render(&self, rng: &mut impl Rng) -> Result<Vec<u8>, Error> {
        let mut builder = String::with_capacity(1024);
        for part in &self.parts {
            match part {
                Either::Left(s) => builder.push_str(&s),
                Either::Right(g) => builder.push_str(&g.fuzz(rng)),
            }
        }

        cbor_diag::parse_diag(&builder)
            .map(|x| x.to_bytes())
            .map_err(|e| e.into())
    }
}

impl FromStr for Cbor {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut re = Regex::new(r"%%|%\(([^\)]*)\)")?;
        let mut parts = Vec::new();

        let mut prev_idx = 0;
        for (idx, part) in s.match_indices(&re) {
            parts.push(Either::Left(s[prev_idx..idx].to_string()));

            if part == "%%" {
                parts.push(Either::Left("%".to_string()));
            } else {
                let generator = parsers::fuzz_string::generator(part)?;
                parts.push(Either::Right(generator));
            }

            prev_idx = idx;
        }

        // Add the last part as a string.
        parts.push(Either::Left(s[prev_idx..].to_string()));

        Ok(Self { parts })
    }
}

#[derive(Parameter, Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Clone)]
#[param(regex = r"\w[\w\d.@]*", name = "identifier")]
pub struct Identifier(String);

impl FromStr for Identifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Identifier(s.to_string()))
    }
}

#[derive(Parameter, Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Clone)]
#[param(regex = r"\S+", name = "method")]
pub struct Method(String);

impl From<Method> for String {
    fn from(value: Method) -> Self {
        value.0
    }
}

impl FromStr for Method {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}
