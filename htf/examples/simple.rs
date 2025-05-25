use cli_log::*;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use htf::test_runner::OperatorComms;

fn test1(comms: &mut OperatorComms) -> Result<()> {
    info!("Waiting");
    let value = comms.request_input("Test 1 Input:".to_string());
    info!("{:?}", value);
    Ok(())
}

fn test2(comms: &mut OperatorComms) -> Result<()> {
    info!("Waiting");
    let value = comms.request_input("Test 2 Input:".to_string());
    info!("{:?}", value);
    Err(eyre!("Err"))
}

fn test3(comms: &mut OperatorComms) -> Result<()> {
    info!("Waiting");
    let value = comms.request_input("Test 3 Input:".to_string());
    info!("{:?}", value);
    Ok(())
}

fn test4(comms: &mut OperatorComms) -> Result<()> {
    info!("Waiting");
    let value = comms.request_input("Test 4 Input:".to_string());
    info!("{:?}", value);
    Ok(())
}

fn test5(comms: &mut OperatorComms) -> Result<()> {
    info!("Waiting");
    let value = comms.request_input("Test 5 Input:".to_string());
    info!("{:?}", value);
    Ok(())
}

fn test6(comms: &mut OperatorComms) -> Result<()> {
    info!("Waiting");
    let value = comms.request_input("Test 6 Input:".to_string());
    info!("{:?}", value);
    Ok(())
}

fn main() -> Result<()> {
    let tests = htf::register_test!(test1, test2, test3, test4, test5, test6);
    htf::run_tests(tests)
}
