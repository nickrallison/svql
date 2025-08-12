
use std::path::{Path, PathBuf};

use svql_query::{composite::SearchableComposite, netlist::SearchableNetlist, queries::security::access_control::locked_reg::{sync_en::SyncEnLockedReg, sync_mux::SyncMuxLockedReg}};
use svql_driver_handler::{proc::ProcDriver, Driver};
use svql_query::{Match, Search, WithPath};

use crate::and::AndAB;

mod and;

fn main() {
    // env logger

    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    // let design = PathBuf::from("examples/larger_designs/opentitan_otbn.il");
    // let module_name = "otbn_core".to_string();

    // let proc_driver = ProcDriver::new(design, module_name).expect("Failed to create proc driver");
    // let cmd = proc_driver.get_command();
    // println!("Command: {}", cmd);
    // let driver = Driver::from(proc_driver);

    let driver = Driver::new_net("localhost:9999".to_string());

    let sync_mux = SyncMuxLockedReg::<Search>::root("sync_mux".to_string());
    assert_eq!(sync_mux.path().inst_path(), "sync_mux");
    assert_eq!(sync_mux.data_in.path.inst_path(), "sync_mux.data_in");
    assert_eq!(sync_mux.clk.path.inst_path(), "sync_mux.clk");
    assert_eq!(sync_mux.resetn.path.inst_path(), "sync_mux.resetn");
    assert_eq!(sync_mux.write_en.path.inst_path(), "sync_mux.write_en");
    assert_eq!(sync_mux.data_out.path.inst_path(), "sync_mux.data_out");

    let sync_mux_search_result = SyncMuxLockedReg::<Search>::query(&driver, sync_mux.path());
    assert_eq!(
        sync_mux_search_result.len(),
        4,
        "Expected 4 matches for SyncMuxLockedReg, got {}",
        sync_mux_search_result.len()
    );

}
