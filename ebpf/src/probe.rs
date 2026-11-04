macro_rules! probe {
    ($name:ident, $try_name:ident) => {
        pub fn $name(ctx: BtfTracePointContext) -> u32 {
            match unsafe { $try_name(ctx) } {
                Ok(()) => 0,
                Err(error) => error as u32,
            }
        }
    };
}
