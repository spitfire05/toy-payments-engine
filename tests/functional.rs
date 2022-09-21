use assert_cmd::prelude::OutputAssertExt;
use predicates::prelude::{predicate, PredicateBooleanExt};

macro_rules! test_output {
    ($name:ident, $path:expr, $($line:expr),*) => {
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let bin = escargot::CargoBuild::new()
                .bin("toy-payments-engine")
                .current_release()
                .current_target()
                .run()?;
            let mut cmd = bin.command();
            cmd.arg($path);
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
    "tests/data/simple.csv",
    "1,1.5000,0.0000,1.5000,false",
    "2,2.0000,0.0000,2.0000,false"
);

test_output!(
    dispute,
    "tests/data/dispute.csv",
    "1,2.0000,1.0000,3.0000,false",
    "2,2.0000,0.0000,2.0000,false"
);
