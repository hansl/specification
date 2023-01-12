use std::cmp::Ordering;

use cucumber::{given, then, when};
use many_client::client::ledger::{SendArgs, TokenAmount};
use num_bigint::BigUint;

use crate::params::Identifier;
use crate::world::World;

#[given(expr = "an identity {identifier}")]
fn setup_identity(world: &mut World, id: Identifier) {
    world.new_identity(id);
}

#[given(expr = "a symbol {word}")]
fn setup_symbol(world: &mut World, symbol: String) {
    assert!(world.symbol(&symbol).is_some());
}

#[given(expr = "{identifier} has {int} {word}")]
async fn id_has_x_symbols(world: &mut World, id: Identifier, amount: BigUint, symbol: String) {
    let amount: TokenAmount = amount.into();
    let symbol = *world.symbol(&symbol).unwrap();
    let current_balance = world.balance(&id, symbol).await;
    let faucet_balance = world.balance("faucet", symbol).await;

    assert!((current_balance.clone() + faucet_balance.clone()) > amount);

    match amount.cmp(&current_balance) {
        Ordering::Greater => {
            world
                .ledger_client("faucet")
                .send(SendArgs {
                    from: None,
                    to: world.address_of(&id).unwrap(),
                    amount: amount.clone() - current_balance,
                    symbol,
                })
                .await
                .unwrap();
        }
        Ordering::Less => {
            world
                .ledger_client(&id)
                .send(SendArgs {
                    from: world.address_of(&id),
                    to: world.address_of("faucet").unwrap(),
                    amount: current_balance - amount.clone(),
                    symbol,
                })
                .await
                .unwrap();
        }
        Ordering::Equal => {}
    }

    let new_balance = world.balance(&id, symbol).await;
    assert_eq!(new_balance, amount);
}

#[when(expr = "{identifier} sends {int} {word} to {identifier}")]
async fn send_symbol(
    world: &mut World,
    sender_id: Identifier,
    amount: u32,
    symbol: String,
    receiver_id: Identifier,
) {
    let symbol = *world.symbol(&symbol).unwrap();
    world
        .ledger_client(&sender_id)
        .send(SendArgs {
            from: None,
            to: world
                .address_of(receiver_id)
                .expect("Could not get address."),
            amount: amount.into(),
            symbol,
        })
        .await
        .unwrap();
}

#[then(expr = "the balance of {identifier} should be {int} {word}")]
async fn balance_should_be(world: &mut World, id: Identifier, amount: BigUint, symbol: String) {
    let symbol = *world.symbol(&symbol).unwrap();
    let balance = world.balance(id, symbol).await;
    assert_eq!(balance, TokenAmount::from(amount));
}
