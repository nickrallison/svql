use svql_driver_handler::Driver;

use crate::{
    composite::{
        Composite, EnumComposite, MatchedComposite, MatchedEnumComposite, SearchableComposite, SearchableEnumComposite
    }, instance::Instance, netlist::SearchableNetlist, queries::security::access_control::locked_reg::{async_en::AsyncEnLockedReg, async_mux::AsyncMuxLockedReg, sync_en::SyncEnLockedReg, sync_mux::SyncMuxLockedReg}, Connection, Match, Search, State, WithPath
};

// Bring the trait that provides `query` into scope
pub mod async_en;
pub mod sync_en;
pub mod sync_mux;
pub mod async_mux;

pub enum LockedReg<S> 
where 
    S: State,
{
    AsyncEn(async_en::AsyncEnLockedReg<S>),
    SyncEn(sync_en::SyncEnLockedReg<S>),
    AsyncMux(async_mux::AsyncMuxLockedReg<S>),
    SyncMux(sync_mux::SyncMuxLockedReg<S>),
}

impl<S> EnumComposite<S> for LockedReg<S>
where
    S: State,
{}

impl SearchableEnumComposite for LockedReg<Search> {
    type Hit = LockedReg<Match>;
    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
        let async_en_search_result: Vec<LockedReg<Match>> =
            AsyncEnLockedReg::<Search>::query(driver, path.child("async_en".to_string()))
            .into_iter()
            .map(LockedReg::AsyncEn)
            .collect();
        let sync_en_search_result: Vec<LockedReg<Match>> =
            SyncEnLockedReg::<Search>::query(driver, path.child("sync_en".to_string()))
            .into_iter()
            .map(LockedReg::SyncEn)
            .collect(); 
        let async_mux_search_result: Vec<LockedReg<Match>> =
            AsyncMuxLockedReg::<Search>::query(driver, path.child("async_mux".to_string()))
            .into_iter()
            .map(LockedReg::AsyncMux)
            .collect();
        let sync_mux_search_result: Vec<LockedReg<Match>> =
            SyncMuxLockedReg::<Search>::query(driver, path.child("sync_mux".to_string()))
            .into_iter()
            .map(LockedReg::SyncMux)
            .collect();
    
        let mut results = Vec::new();
        results.extend(async_en_search_result);
        results.extend(sync_en_search_result);
        results.extend(async_mux_search_result);
        results.extend(sync_mux_search_result);
        
        results
    }
}

impl MatchedEnumComposite for LockedReg<Match> {
    fn other_filters(&self) -> Vec<Box<dyn Fn(&Self) -> bool>> {
        vec![]
    }
}