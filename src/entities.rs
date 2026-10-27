use crate::Result;

pub trait EntityResolver {
    fn cpu(&mut self, cpu: u32) -> Result<u64>;
    fn thread(&mut self, tid: u32) -> Result<u64>;
}
