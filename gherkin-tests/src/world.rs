use crate::params::{Cbor, Identifier};
use crate::{cose::new_identity, opts::SpecConfig};
use async_trait::async_trait;
use cucumber::WorldInit;
use many_client::client::base::BaseClient;
use many_client::client::ledger::{BalanceArgs, LedgerClient, Symbol, TokenAmount};
use many_client::client::send_envelope;
use many_client::ManyClient;
use many_error::ManyError;
use many_identity::verifiers::AnonymousVerifier;
use many_identity::{Address, AnonymousIdentity, Identity};
use many_identity_dsa::{CoseKeyIdentity, CoseKeyVerifier};
use many_protocol::{
    decode_response_from_cose_sign1, encode_cose_sign1_from_request, RequestMessage,
    ResponseMessage,
};
use minicbor::Encode;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::{collections::BTreeMap, convert::Infallible, sync::Arc};
use url::Url;

mod variables;

pub use variables::WorldVar;

#[derive(Debug, WorldInit)]
pub struct World {
    messages: BTreeMap<Identifier, Cbor>,

    spec_config: Option<Arc<SpecConfig>>,

    variables: BTreeMap<String, WorldVar>,

    server_url: Option<Url>,

    rand: StdRng,
    latest_response: Option<ResponseMessage>,
}

impl World {
    pub fn seed(&mut self, seed: u64) {
        self.rand = StdRng::seed_from_u64(seed);
    }

    pub fn rng(&mut self) -> &mut StdRng {
        &mut self.rand
    }

    async fn send_(
        &self,
        identity: impl AsRef<str>,
        request: RequestMessage,
    ) -> Result<ResponseMessage, ManyError> {
        let cose_sign1 = encode_cose_sign1_from_request(
            request,
            self.identity(identity)
                .expect("Invalid identity name")
                .as_ref(),
        )?;

        let envelope =
            send_envelope(self.server_url.as_ref().unwrap().to_string(), cose_sign1).await?;

        decode_response_from_cose_sign1(&envelope, None, &(AnonymousVerifier, CoseKeyVerifier))
    }

    pub async fn send(&mut self, identity: impl AsRef<str>, request: RequestMessage) {
        let response = self
            .send_(identity, request)
            .await
            .expect("Could not send message");
        self.latest_response = Some(response);
    }

    pub async fn call(
        &mut self,
        identity: impl AsRef<str>,
        method: impl ToString,
        args: impl Encode<()>,
    ) {
        let message = RequestMessage::default()
            .with_method(method.to_string())
            .with_data(minicbor::to_vec(args).expect("Could not serialize argument"));

        self.send(identity, message).await;
    }

    async fn call_(
        &self,
        identity: impl AsRef<str>,
        method: impl ToString,
        args: impl Encode<()>,
    ) -> Result<ResponseMessage, ManyError> {
        let message = RequestMessage::default()
            .with_method(method.to_string())
            .with_data(minicbor::to_vec(args).expect("Could not serialize argument"));

        self.send_(identity, message).await
    }

    pub fn address_of(&self, identity: impl AsRef<str>) -> Option<Address> {
        match self.variables.get(identity.as_ref()) {
            Some(WorldVar::Symbol(s)) => Some(*s),
            Some(WorldVar::Address(a)) => Some(*a),
            Some(WorldVar::Identity(id)) => Some(id.address()),
            _ => None,
        }
    }

    pub fn register_cbor(&mut self, id: Identifier, cbor: Cbor) {
        self.messages.insert(id, cbor);
    }

    pub fn render(&mut self, id: Identifier) -> Vec<u8> {
        let cbor = self.messages.get(&id).expect("Could not find CBOR.");
        cbor.render(&mut self.rand, &self.variables)
            .expect("Could not render CBOR")
    }

    pub async fn init_config(&mut self, spec_config: Arc<SpecConfig>) {
        // Some predefined constants.
        self.insert_var("anonymous", WorldVar::Identity(Arc::new(AnonymousIdentity)));
        self.insert_var("illegal", WorldVar::Address(Address::illegal()));

        self.spec_config = Some(spec_config);

        let faucet_identity = self.spec_config().faucet_identity.clone();
        self.insert_var("faucet", WorldVar::Identity(Arc::new(faucet_identity)));

        self.server_url = Some(self.spec_config().server_url.clone());

        let info = self.ledger_client("faucet").info().await.unwrap();
        for (address, name) in info.local_names.into_iter() {
            self.insert_var(name, WorldVar::Symbol(address));
        }
    }

    pub fn insert_var(&mut self, name: impl ToString, var: WorldVar) {
        if let Some(v) = self.variables.insert(name.to_string(), var) {
            panic!("Var {} already exists with value {v:?}", name.to_string())
        }
    }

    pub fn spec_config(&self) -> &SpecConfig {
        self.spec_config.as_ref().unwrap()
    }

    pub fn symbol(&self, symbol: &str) -> Option<&Symbol> {
        match self.variables.get(symbol) {
            Some(WorldVar::Symbol(s)) => Some(s),
            _ => None,
        }
    }

    pub fn insert_identity(&mut self, id: Identifier) {
        let identity = new_identity();
        self.insert_var(id.to_string(), WorldVar::identity());

        let many_client = ManyClient::new(
            self.spec_config().server_url.clone(),
            AnonymousIdentity.address(),
            identity.clone(),
        )
        .unwrap();
    }

    pub fn identity(&self, id: impl AsRef<str>) -> Option<Arc<dyn Identity>> {
        self.variables.get(id.as_ref()).and_then(|var| match var {
            WorldVar::Identity(id) => Some(id.clone()),
            _ => None,
        })
    }

    pub fn ledger_client(&self, id: impl AsRef<str>) -> LedgerClient<CoseKeyIdentity> {
        let id = self.identity(id).expect("Could not find identity");
        LedgerClient::new(
            ManyClient::new(self.server_url.clone().unwrap(), Address::anonymous(), id).unwrap(),
        )
    }

    pub fn base_client(&self, id: impl AsRef<str>) -> &BaseClient<CoseKeyIdentity> {
        BaseClient::new(ManyClient::new(
            self.server_url.clone().unwrap(),
            Address::anonymous(),
            self.identity(id).expect("Could not find identity."),
        ))
    }

    pub async fn balance(&self, identity: impl AsRef<str>, symbol: Symbol) -> TokenAmount {
        let mut response = self
            .ledger_client(&identity)
            .balance(BalanceArgs {
                account: self.address_of(&identity),
                symbols: Some(vec![symbol].into()),
            })
            .await
            .expect("Could not send message");

        response
            .balances
            // Remove gets by ownership
            .remove(&symbol)
            .unwrap_or_default()
    }
}

#[async_trait(?Send)]
impl cucumber::World for World {
    type Error = Infallible;

    async fn new() -> Result<Self, Self::Error> {
        Ok(World {
            spec_config: None,
            rand: StdRng::seed_from_u64(0),
            server_url: None,
            latest_response: None,
            messages: Default::default(),
            variables: Default::default(),
            client: None,
        })
    }
}

impl Drop for World {
    fn drop(&mut self) {}
}
