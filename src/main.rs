use crate::params::AppParams;

mod params;
mod util;

fn main() {
    // load AppParams file
    let params: AppParams = match AppParams::from_json() {
        Ok(val) => val,                 // load params from file
        Err(_) => AppParams::default(), // or create a new default param
    };

    println!("Hello, world!");
}
