extern crate funky_logger;
#[macro_use] extern crate log;

mod one {
    pub fn deep() {
        trace!("one level deep!");
    }
}

fn main() {
    if let Err(e) = funky_logger::try_init() {
        eprintln!("Some custom msg {}", e);
        panic!("error!") // or whatever
    };

    info!("such information");
    warn!("o_O");
    error!("boom");
    debug!("deboogging");
    self::one::deep();
}
