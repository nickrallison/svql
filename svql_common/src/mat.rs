use crate::core::list::List;
use crate::core::string::CrateCString;
use serde::{Serialize, Deserialize};
use std::hash::{Hash};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MatchList {
    pub matches: Vec<Match>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Match {
    pub port_map: Vec<(String, String)>,
    pub cell_map: Vec<(CellData, CellData)>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct CellData {
    pub cell_name: String,
    pub cell_index: usize,
}

// ========== C-Compatible Structs ==========

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CCellData {
    pub cell_name: CrateCString,
    pub cell_index: usize,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CCellDataPair {
    pub first: CCellData,
    pub second: CCellData,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CCellDataPairList {
    pub items: List<CCellDataPair>,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CStringPair {
    pub first: CrateCString,
    pub second: CrateCString,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CStringPairList {
    pub items: List<CStringPair>,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CMatch {
    pub port_map: CStringPairList,
    pub cell_map: CCellDataPairList,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CMatchList {
    pub matches: List<CMatch>,
}

// Rust -> C
impl From<&CellData> for CCellData {
    fn from(cd: &CellData) -> Self {
        CCellData {
            cell_name: CrateCString::from(cd.cell_name.as_str()),
            cell_index: cd.cell_index,
        }
    }
}
impl From<&(CellData, CellData)> for CCellDataPair {
    fn from(pair: &(CellData, CellData)) -> Self {
        CCellDataPair {
            first: CCellData::from(&pair.0),
            second: CCellData::from(&pair.1),
        }
    }
}
impl From<&(String, String)> for CStringPair {
    fn from(pair: &(String, String)) -> Self {
        CStringPair {
            first: CrateCString::from(pair.0.as_str()),
            second: CrateCString::from(pair.1.as_str()),
        }
    }
}
impl From<&Match> for CMatch {
    fn from(m: &Match) -> Self {
        CMatch {
            port_map: CStringPairList {
                items: m.port_map.iter().map(CStringPair::from).collect(),
            },
            cell_map: CCellDataPairList {
                items: m.cell_map.iter().map(CCellDataPair::from).collect(),
            },
        }
    }
}
impl From<&MatchList> for CMatchList {
    fn from(ml: &MatchList) -> Self {
        CMatchList {
            matches: ml.matches.iter().map(CMatch::from).collect(),
        }
    }
}

// C -> Rust
impl From<&CCellData> for CellData {
    fn from(c: &CCellData) -> Self {
        CellData {
            cell_name: c.cell_name.as_str().to_string(),
            cell_index: c.cell_index,
        }
    }
}
impl From<&CCellDataPair> for (CellData, CellData) {
    fn from(pair: &CCellDataPair) -> Self {
        (CellData::from(&pair.first), CellData::from(&pair.second))
    }
}
impl From<&CStringPair> for (String, String) {
    fn from(pair: &CStringPair) -> Self {
        (pair.first.as_str().to_string(), pair.second.as_str().to_string())
    }
}
impl From<&CMatch> for Match {
    fn from(c: &CMatch) -> Self {
        Match {
            port_map: c.port_map.items.as_slice().iter().map(|p| p.into()).collect(),
            cell_map: c.cell_map.items.as_slice().iter().map(|p| p.into()).collect(),
        }
    }
}
impl From<&CMatchList> for MatchList {
    fn from(c: &CMatchList) -> Self {
        MatchList {
            matches: c.matches.as_slice().iter().map(|m| m.into()).collect(),
        }
    }
}

impl Default for CMatch {
    fn default() -> Self {
        CMatch {
            port_map: CStringPairList { items: List::new() },
            cell_map: CCellDataPairList { items: List::new() },
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ccelldata_new(string: CrateCString, index: usize) -> *mut CCellData {
    Box::into_raw(Box::new(CCellData {
        cell_name: string,
        cell_index: index,
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn ccelldata_destroy(ccelldata: *mut CCellData) {
    if !ccelldata.is_null() {
        unsafe { let _ = Box::from_raw(ccelldata); };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn match_add_celldata(cmatch: &mut CMatch, cell_data_1: CCellData, cell_data_2: CCellData)  {
    let cell_pair = CCellDataPair {
        first: cell_data_1,
        second: cell_data_2,
    };
    
    cmatch.cell_map.items.append(cell_pair);
}

#[unsafe(no_mangle)]
pub extern "C" fn match_add_portdata(cmatch: &mut CMatch, port_data_1: CrateCString, port_data_2: CrateCString)  {
    let port_pair = CStringPair {
        first: port_data_1,
        second: port_data_2,
    };
    
    cmatch.port_map.items.append(port_pair);
}

#[unsafe(no_mangle)]
pub extern "C" fn match_new() -> *mut CMatch {
    Box::into_raw(Box::new(CMatch::default()))
}

#[unsafe(no_mangle)]
pub extern "C" fn match_destroy(cmatch: *mut CMatch) {
    if !cmatch.is_null() {
        unsafe { let _ = Box::from_raw(cmatch); };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn match_list_new() -> *mut CMatchList {
    Box::into_raw(Box::new(CMatchList { matches: List::new() }))
}

#[unsafe(no_mangle)]
pub extern "C" fn append_match_to_matchlist(list: &mut CMatchList, match_data: CMatch) {
    list.matches.append(match_data);
}

#[unsafe(no_mangle)]
pub extern "C" fn match_list_clone(list: *mut CMatchList) -> *mut CMatchList {
    if list.is_null() {
        return std::ptr::null_mut();
    }
    let c_match_list_ref = unsafe { &*list };
    let r: CMatchList = c_match_list_ref.clone();
    Box::into_raw(Box::new(r))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn match_list_destroy(list: *mut CMatchList) {
    if !list.is_null() {
        unsafe { let _ = Box::from_raw(list); };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn match_list_eq(a: &CMatchList, b: &CMatchList) -> bool {
    a == b
}

#[unsafe(no_mangle)]
pub extern "C" fn match_list_debug_string(list: &CMatchList) -> *mut CrateCString {
    let rust = MatchList::from(list);
    let s = format!("{:?}", rust);
    
    Box::into_raw(Box::new(CrateCString::from(s.as_str())))
}

#[unsafe(no_mangle)]
pub extern "C" fn match_list_to_json(list: &CMatchList) -> *mut CrateCString {
    let rust = MatchList::from(list);
    match serde_json::to_string(&rust) {
        Ok(json) => Box::into_raw(Box::new(CrateCString::from(json.as_str()))),
        Err(e) => panic!("Failed to serialize to JSON: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use crate::core::string::crate_cstring_destroy;

    fn make_sample() -> MatchList {
        MatchList {
            matches: vec![
                Match {
                    port_map: vec![("a".to_string(), "b".to_string())],
                    cell_map: vec![
                        (
                            CellData { cell_name: "foo".to_string(), cell_index: 1 },
                            CellData { cell_name: "bar".to_string(), cell_index: 2 }
                        )
                    ],
                }
            ]
        }
    }

    #[test]
    fn test_roundtrip() {
        let orig = make_sample();
        let c = CMatchList::from(&orig);
        let back = MatchList::from(&c);
        assert_eq!(orig.matches.len(), back.matches.len());
        assert_eq!(orig.matches[0].port_map, back.matches[0].port_map);
        assert_eq!(orig.matches[0].cell_map, back.matches[0].cell_map);
    }

    #[test]
    fn test_clone_and_eq() {
        let c1 = CMatchList::from(&make_sample());
        let c2 = c1.clone();
        assert_eq!(c1, c2);
        assert_ne!(&c1 as *const _, &c2 as *const _);
    }

    #[test]
    fn test_hash() {
        let c1 = CMatchList::from(&make_sample());
        let c2 = c1.clone();
        let mut set = HashSet::new();
        set.insert(c1);
        assert!(set.contains(&c2));
    }

    #[test]
    fn test_debug_string() {
        let c = CMatchList::from(&make_sample());
        let dbg: *mut CrateCString = match_list_debug_string(&c);
        let s = unsafe { (&*dbg).as_str() };
        assert!(s.contains("cell_name"));
        crate_cstring_destroy(dbg);
    }

    #[test]
    fn test_ffi_lifecycle() {
        unsafe {
            let c = match_list_new();
            let c2 = match_list_clone(c);
            assert!(match_list_eq(&*c, &*c2));
        
            match_list_destroy(c2);
            match_list_destroy(c);
        }
    }
}