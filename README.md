# toy-payments-engine

## Usage

`cargo run -- INPUT`

or

`./toy-payments-engine INPUT`

where `INPUT` is the path to the CSV file with input.

## Tests

Can be run by `cargo test`. They are divided in 3 groups:

* Unit tests - defined in respective modules.
* Functional tests - defined in `tests/functional.rs` - test the whole application with prepared data sets.
* User Interaction - defined in `tests/ui.rs` - test the "unhappy path" feedback for the user.

## Notes, assumptions and considerations

* The type used to handle the transaction amounts is `f64`. In real application, probably something custom, less prone to rounding errors, should be used.
* Only deposit transactions can be disputed.
* "Locked" clients can not accept deposits nor withdrawals. They can, however, accept new disputes, resolves and chargebacks.
* The current implementation of `Repository` is not very multithread-friendly nor optimized, but it can be easily modified to be so by modifying `clients` field to be of type `HashMap<u16, Mutex<Client>>`, aquiring the lock in `register_transaction` method, and sharing the `Repository` object between threads via an `Arc`. This way we can aquire Mutex locks per client instead of on whole repository.