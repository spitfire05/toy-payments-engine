// User Interaction tests

use assert_cmd::prelude::OutputAssertExt;
use predicates::prelude::predicate;

#[test]
fn fails_on_zero_arguments() -> Result<(), Box<dyn std::error::Error>> {
    let bin = escargot::CargoBuild::new()
        .bin("toy-payments-engine")
        .current_release()
        .current_target()
        .run()?;
    let mut cmd = bin.command();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Incorrect number of arguments"));

    Ok(())
}

#[test]
fn fails_on_two_arguments() -> Result<(), Box<dyn std::error::Error>> {
    let bin = escargot::CargoBuild::new()
        .bin("toy-payments-engine")
        .current_release()
        .current_target()
        .run()?;
    let mut cmd = bin.command();
    cmd.arg("foo").arg("bar");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Incorrect number of arguments"));

    Ok(())
}
