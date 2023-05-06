use feml::Value;

fn main() {
    let mut table = feml::parse_file("Cargo.toml").unwrap();
    table.insert("MyKey".to_string(), Value::Boolean(true));
    println!("{table:#?}");
}
