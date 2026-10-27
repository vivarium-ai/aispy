use libloading::{Library, Symbol};

use crate::{Error, Result};

type CuptiGetVersion = unsafe extern "C" fn(
    version: *mut u32,
) -> i32;

pub struct Cupti {
    library: Library,
}

impl Cupti {
    pub fn load() -> Result<Self> {
        let library = unsafe {
            Library::new("libcupti.so")
        }
        .map_err(|error| Error::CuptiLoad {
            message: error.to_string(),
        })?;

        Ok(Self { library })
    }

    pub fn version(&self) -> Result<u32> {
        let get_version: Symbol<'_, CuptiGetVersion> = unsafe {
            self.library.get(b"cuptiGetVersion\0")
        }
        .map_err(|error| Error::CuptiLoad {
            message: error.to_string(),
        })?;

        let mut version = 0;

        let result = unsafe {
            get_version(&mut version)
        };

        if result != 0 {
            return Err(Error::CuptiCall {
                function: "cuptiGetVersion",
                result,
            });
        }

        Ok(version)
    }
}
