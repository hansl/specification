use cddl::ast::{Rule, TypeRule};
use cucumber::Parameter;
use lazy_static::lazy_static;
use std::str::FromStr;

const CDDL_STRING: &str = include_str!(concat!(env!("OUT_DIR"), "/all.cddl"));

lazy_static! {
    static ref CDDL: cddl::ast::CDDL<'static> =
        cddl::cddl_from_str(CDDL_STRING, true).expect("Could not parse CDDL.");
}

#[derive(Parameter, Debug, Clone)]
#[param(regex = r#"([a-zA-Z0-9@.-])*"#, name = "cddl-type")]
pub struct CddlType(String);

impl CddlType {
    pub fn matches(&self, cbor: &[u8]) -> bool {
        let result = cddl::validate_cbor_from_slice(
            &format!("start = {}\n{}", &self.0, CDDL_STRING),
            cbor,
            None,
        );

        if let Err(e) = result {
            eprintln!("{e}");
            unreachable!()
        } else {
            true
        }
    }
}

impl FromStr for CddlType {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        let cddl = cddl::cddl_from_str(CDDL_STRING, true).expect("Could not parse CDDL.");
        // Just make sure the rule exists.
        let rule: &Rule = cddl
            .rules
            .iter()
            .find(|rule| rule.name() == name)
            .ok_or("Could not find rule".to_string())?;

        Ok(Self(name.to_string()))
    }
}
