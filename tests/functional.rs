use assert_cmd::prelude::OutputAssertExt;
use predicates::prelude::{predicate, PredicateBooleanExt};

macro_rules! test_output {
    ($name:ident, $($line:expr),*) => {
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let bin = escargot::CargoBuild::new()
                .bin("toy-payments-engine")
                .current_release()
                .current_target()
                .run()?;
            let mut cmd = bin.command();
            let name = stringify!($name);
            cmd.arg(format!("tests/data/{}.csv", name));
            cmd.assert().success().stdout(
                predicate::str::contains("client,available,held,total,locked")
                $(
                    .and(predicate::str::contains($line))
                )*,
            );

            Ok(())
        }
    };
}

test_output!(
    simple,
    "1,1.5000,0.0000,1.5000,false",
    "2,2.0000,0.0000,2.0000,false"
);

test_output!(
    dispute,
    "1,0.5000,1.0000,1.5000,false",
    "2,2.0000,0.0000,2.0000,false"
);

test_output!(
    dispute_negative,
    "1,-1.0000,1.0000,0.0000,false",
    "2,2.0000,0.0000,2.0000,false"
);

test_output!(
    resolve,
    "1,1.5000,0.0000,1.5000,false",
    "2,2.0000,0.0000,2.0000,false"
);

test_output!(
    chargeback,
    "1,0.5000,0.0000,0.5000,true",
    "2,2.0000,0.0000,2.0000,false"
);
