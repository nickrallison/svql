
use std::path::{Path, PathBuf};

use svql_query::{composite::SearchableComposite, netlist::SearchableNetlist, queries::security::access_control::locked_reg::sync_en::SyncEnLockedReg};
use svql_driver_handler::{proc::ProcDriver, Driver};
use svql_query::{Match, Search, WithPath};

use crate::and::AndAB;

mod and;

fn main() {
    // env logger

    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let design = PathBuf::from("examples/patterns/security/access_control/locked_reg/rtlil/async_en.il");
    let module_name = "async_en".to_string();



    let proc_driver = ProcDriver::new(design, module_name).expect("Failed to create proc driver");
    let cmd = proc_driver.get_command();
    println!("Command: {}", cmd);
    let driver = Driver::from(proc_driver);


    let sync_en = SyncEnLockedReg::<Search>::root("sync_en".to_string());
    assert_eq!(sync_en.path().inst_path(), "sync_en");
    assert_eq!(sync_en.data_in.path.inst_path(), "sync_en.data_in");
    assert_eq!(sync_en.clk.path.inst_path(), "sync_en.clk");
    assert_eq!(sync_en.resetn.path.inst_path(), "sync_en.resetn");
    assert_eq!(sync_en.write_en.path.inst_path(), "sync_en.write_en");
    assert_eq!(sync_en.data_out.path.inst_path(), "sync_en.data_out");

    let sync_en_search_result = SyncEnLockedReg::<Search>::query(&driver, sync_en.path());
    assert_eq!(
        sync_en_search_result.len(),
        1,
        "Expected 1 match for SyncEnLockedReg, got {}",
        sync_en_search_result.len()
    );

}
