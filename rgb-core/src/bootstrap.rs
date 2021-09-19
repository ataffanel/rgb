use std::io;
use std::io::prelude::*;
use std::fs::File;

pub struct Bootstrap {
    bootstrap: Vec<u8>,
}

#[derive(Debug)]
pub struct BootstrapLoadError {
    pub error: String,
}

impl Bootstrap {
    pub fn load(path: &str) -> Result<Bootstrap, BootstrapLoadError> {
        let mut f = File::open(path)?;
        let mut buffer = Vec::new();

        f.read_to_end(&mut buffer)?;

        Ok(Bootstrap {
            bootstrap: buffer,
        })
    }

    pub fn create_from_slice(slice: &[u8]) -> Bootstrap {
        Bootstrap {
            bootstrap: slice.to_vec(),
        }
    }

    pub fn create_default() -> Bootstrap {
        Bootstrap {
            bootstrap: include_bytes!("../bootstrap/bootstrap.bin").to_vec(),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.bootstrap[address as usize]
    }
}


impl From<io::Error> for BootstrapLoadError {
    fn from(err: io::Error) -> BootstrapLoadError {
        BootstrapLoadError {
            error: format!("{:?}", err)
        }
    }
}
