use std::vec;

use rpc_toy::Client;

pub fn main() {
    let mut client = Client::new("127.0.0.1:3001").unwrap();
    let one = serde_json::to_value(1u32).unwrap();
    let two = serde_json::to_value(2u32).unwrap();
    let args = vec![one, two];
    let res = client.call("Add", &args).unwrap();
    let three: u32 = serde_json::from_value(res.unwrap()).unwrap();
    assert_eq!(three, 3);
    let world = client.call("hello", &vec![]).unwrap();
    println!(
        "Hello, {}!",
        serde_json::from_value::<String>(world.unwrap()).unwrap()
    );
    let nothing = client.call("void_fn", &vec![]).unwrap();
    assert!(nothing.is_none());
}
