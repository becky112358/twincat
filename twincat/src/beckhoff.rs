#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::upper_case_acronyms)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

impl Default for AmsAddr {
    fn default() -> Self {
        Self {
            netId: AmsNetId_ { b: [0; 6] },
            port: 851,
        }
    }
}

impl Default for AdsSymbolUploadInfo2 {
    fn default() -> Self {
        Self {
            nSymbols: 0,
            nSymSize: 0,
            nDatatypes: 0,
            nDatatypeSize: 0,
            nMaxDynSymbols: 0,
            nUsedDynSymbols: 0,
        }
    }
}
