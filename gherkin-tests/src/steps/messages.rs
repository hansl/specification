use crate::params::cddl::CddlType;
use crate::params::{Cbor, Identifier, Method};
use crate::world::{World, WorldVar};
use cucumber::{given, then, when};
use many_protocol::RequestMessage;

#[given(expr = "a cbor {identifier} = {cbor}")]
fn cbor(world: &mut World, identifier: Identifier, definition: Cbor) {
    world.register_cbor(identifier, definition);
}

#[when(expr = "calling {method} with {identifier} into {identifier}")]
async fn send_cbor(world: &mut World, method: Method, message: Identifier, varname: Identifier) {
    let rendered = world.render(message);
    let response = world
        .send(
            "anonymous",
            RequestMessage::default()
                .with_data(rendered)
                .with_method(method.into()),
        )
        .await
        .expect("Error while sending message");
    world.insert_var(varname, WorldVar::Response(response))
}

// #[then(expr = "response {identifier} matches {cbor}")]
// fn response_matches(world: &mut World, identifier: Identifier, matcher: Cbor) {
//     let matcher = world.render_cbor_string(&matcher);
//     let matcher = crate::params::matchers::parse_diag(matcher).expect("Could not parse CBOR");
//     let cbor = world.response_cbor(identifier).unwrap();
//
//     assert!(cbor.matches(&matcher, true));
// }

#[then(expr = "response {identifier} matches cddl type {cddl-type}")]
fn response_matches_cddl(world: &mut World, identifier: Identifier, matcher: CddlType) {
    assert!(matcher.matches(&world.response_cbor(identifier).unwrap()));
}
