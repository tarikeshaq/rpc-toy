use rpc_toy::Server;

pub fn main() {
    let mut server = Server::new();
    server.register("Add", |args| {
        let one = args.get(0).unwrap();
        let two = args.get(1).unwrap();
        let one = serde_json::from_value::<u32>(one.clone()).unwrap();
        let two = serde_json::from_value::<u32>(two.clone()).unwrap();
        let three = one + two;
        return Some(serde_json::to_value(three).unwrap());
    });

    server.register("hello", |args| {
        assert_eq!(args.len(), 0);
        return Some(serde_json::to_value("world").unwrap());
    });

    server.register("void_fn", |_| {
        return None;
    });

    server.listen("127.0.0.1:3001").unwrap();
}
