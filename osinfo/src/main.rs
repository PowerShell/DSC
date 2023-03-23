mod config;

fn main() {
    let json = serde_json::to_string(&config::OsInfo::new()).unwrap();
    println!("{}", json);
}
