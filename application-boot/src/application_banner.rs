use tracing::info;


const BANNER: &str = r###"
   ___     _ __    _ __     _       _                      _        _                               ___                     _
  /   \   | '_ \  | '_ \   | |     (_)     __     __ _    | |_     (_)     ___    _ _       o O O  | _ )    ___     ___    | |_
  | - |   | .__/  | .__/   | |     | |    / _|   / _` |   |  _|    | |    / _ \  | ' \     o       | _ \   / _ \   / _ \   |  _|
  |_|_|   |_|__   |_|__   _|_|_   _|_|_   \__|_  \__,_|   _\__|   _|_|_   \___/  |_||_|   TS__[O]  |___/   \___/   \___/   _\__|
_|"""""|_|"""""|_|"""""|_|"""""|_|"""""|_|"""""|_|"""""|_|"""""|_|"""""|_|"""""|_|"""""| {======|_|"""""|_|"""""|_|"""""|_|"""""|
"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'./o--000'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'
"###;

pub struct ApplicationBootBannerPrinter {

}

pub trait Banner {
    fn print(&self);
}

impl Banner for ApplicationBootBannerPrinter {
    fn print(&self) {
        info!("{}", BANNER);
    }
}
