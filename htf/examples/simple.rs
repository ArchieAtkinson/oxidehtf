use cli_log::*;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use htf::operator::Input;

fn test1() -> Result<()> {
    info!("Waiting");
    let value = Input::request("Test 1 Input:");
    info!("{:?}", value);
    Ok(())
}

fn test2() -> Result<()> {
    info!("Waiting");
    let value = Input::request("Test 2 Input:");
    info!("{:?}", value);
    Err(eyre!("Err"))
}

fn test3() -> Result<()> {
    info!("Waiting");
    let value = Input::request("Test 3 Input:");
    info!("{:?}", value);
    Ok(())
}

fn test4() -> Result<()> {
    info!("Waiting");
    let value = Input::request("Test 4 Input:");
    info!("{:?}", value);
    Ok(())
}

fn test5() -> Result<()> {
    info!("Waiting");
    let value = Input::request("Test 5 Input:");
    info!("{:?}", value);
    Ok(())
}

fn test6() -> Result<()> {
    info!("Waiting");
    let value = Input::request("Test 6 Input:");
    info!("{:?}", value);
    Ok(())
}

fn main() -> Result<()> {
    let tests = htf::register_test!(test1, test2, test3, test4, test5, test6);
    htf::run_tests(tests)
}
