use crate::params::parsers::FuzzGenerator;
use crate::params::Cbor;
use many_identity::{Address, Identity};
use many_identity_dsa::ecdsa::generate_random_ecdsa_cose_key;
use many_identity_dsa::CoseKeyIdentity;
use many_types::ledger::Symbol;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

/// A variable usable in a feature, e.g. an identity.
/// These should have an associated identifier, and each of these can be used
/// in a search and replace for CBOR, for example, or when matching values.
pub enum WorldVar {
    Identity(Arc<dyn Identity + Sync>),
    Address(Address),
    Symbol(Symbol),
    Cbor(Cbor),
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
        }
    }
}

impl WorldVar {
    pub fn identity() -> Self {
        let cose_key = generate_random_ecdsa_cose_key();
        let identity = CoseKeyIdentity::from_key(&cose_key)
            .expect("Should have generated a random cose key identity");

        Self::Identity(Box::new(identity))
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
        }
    }
}
