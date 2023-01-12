use crate::params::parsers::FuzzGenerator;
use crate::params::Cbor;
use many_identity::{Address, Identity};
use many_identity_dsa::ed25519::generate_random_ed25519_identity;
use many_protocol::ResponseMessage;
use many_types::ledger::Symbol;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

/// A variable usable in a feature, e.g. an identity.
/// These should have an associated identifier, and each of these can be used
/// in a search and replace for CBOR, for example, or when matching values.
pub enum WorldVar {
    Identity(Arc<dyn Identity>),
    Address(Address),
    Symbol(Symbol),
    Cbor(Cbor),
    Response(ResponseMessage),
}

impl Debug for WorldVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WorldVar::Identity(id) => f
                .debug_tuple("WorldVar::Identity")
                .field(&id.address())
                .finish(),
            WorldVar::Address(id) => f.debug_tuple("WorldVar::Address").field(id).finish(),
            WorldVar::Symbol(id) => f.debug_tuple("WorldVar::Symbol").field(id).finish(),
            WorldVar::Cbor(cbor) => f.debug_tuple("WorldVar::Cbor").field(cbor).finish(),
            WorldVar::Response(response) => f
                .debug_tuple("WorldVar::Response")
                .field(&response)
                .finish(),
        }
    }
}

impl WorldVar {
    pub fn identity() -> Self {
        Self::Identity(Arc::new(generate_random_ed25519_identity()))
    }
}

impl FuzzGenerator for WorldVar {
    fn fuzz<Rng: rand::Rng>(
        &self,
        rng: &mut Rng,
        variables: &BTreeMap<String, WorldVar>,
    ) -> String {
        use WorldVar::*;

        match self {
            Identity(id) => {
                format!("h'{}'", hex::encode(id.address().to_vec()))
            }
            Address(id) | Symbol(id) => {
                format!("h'{}'", hex::encode((*id).to_vec()))
            }
            Cbor(content) => content
                .render_string(rng, variables)
                .expect("Could not render"),
            Response(_) => panic!("Cannot render response."),
        }
    }
}
