#[derive(Debug, Clone, Copy)]
pub struct Site {
    pub kind: &'static str,
    pub function: &'static str,
    pub module: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug)]
pub struct Guard {
    site: Site,
}

#[must_use]
pub fn enter(site: Site) -> Guard {
    emit_begin(site);

    Guard { site }
}

impl Drop for Guard {
    fn drop(&mut self) {
        emit_end(self.site);
    }
}

fn emit_begin(_site: Site) {
    // TODO:
    // timestamp
    // instance_id
    // enqueue fixed-size observation record
}

fn emit_end(_site: Site) {
    // TODO:
    // timestamp
    // instance_id
    // enqueue fixed-size observation record
}
