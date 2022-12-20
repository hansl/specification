use crate::params::{Cbor, Identifier, Method};
use crate::world::World;
use cucumber::{given, when};
use many_protocol::RequestMessage;

#[given(expr = "a cbor {identifier} = {cbor}")]
fn cbor(world: &mut World, identifier: Identifier, definition: Cbor) {
    world.register_cbor(identifier, definition);
}

#[when(expr = "calling {method} with {identifier}")]
async fn send_cbor(world: &mut World, method: Method, cbor: Identifier) {
    let rendered = world.render(cbor);
    world
        .send(
            None,
            RequestMessage::default()
                .with_data(rendered)
                .with_method(method.into()),
        )
        .await;
}
