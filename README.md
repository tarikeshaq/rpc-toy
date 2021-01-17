 ## A Toy RPC framework that is bug-prone, super slow, and hard to use
 ### But.. why?
 Well, I'm taking a class on distributed systems now and RPC was a topic so...
 What's best to understand a topic that to implement it?

 So that's where this came from. I repeat, this is toy. please no one
 use this

 ### General Information:
 This library is built upon two main protocols/concepts:
 - JSON serialization
 - TCP
 
 `JSON` is used for everything as a convinient (yet probably slow) way
 to serialize data types to strings. Then, the strings are converted to
 `UTF8` bytes and passed down.
 `TCP` is our choice of transport layer protocol. It makes moving the bytes
 from point A to point B easy.. So why not.
 Again, this is a toy. Not meant to be used for benchmarking or anything
 ### How to use:
 This library includes two main structs
 a `Client` struct and a `Server` struct.
 a server is meant to register RPC functions, and the client
 can then call them.

 ### Examples:
 #### Example Client
 ```rust
 use rpc_toy::Client;
 // You can create a new client using "new"
 let mut client = Client::new("127.0.0.1:3001").unwrap();
 // All arguments have to be passed in as a slice of `serde_json::Value`s
 let one = serde_json::to_value(1u32).unwrap();
 let two = serde_json::to_value(2u32).unwrap();
 let args = vec![one, two];
 // Use the `call` function to call remote procedures
 let res = client.call("Add", &args).unwrap();

 let three: u32 = serde_json::from_value(res.unwrap()).unwrap();
 assert_eq!(three, 3);
 ```
 #### Example Server
 ```rust
 use rpc_toy::Server;
 let mut server = Server::new();
 server.register("Add", |args| {
     let one = args.get(0).unwrap();
     let two = args.get(1).unwrap();
     let one = serde_json::from_value::<u32>(one.clone()).unwrap();
     let two = serde_json::from_value::<u32>(two.clone()).unwrap();
     let three = one + two;
     return Some(serde_json::to_value(three).unwrap());
 });
 server.listen("127.0.0.1:3001").unwrap();
 ```
 ### Message encodings:
 
 |  The client message encoding                        |
 | :-------------------------------------------------: |
 | 32 bits for the length of the function name         |
 | the name of the function                            |
 | The length of the argument, or zero for termination |
 | The argument encoded as JSON string utf8            |
 | second argument length, or zero for termination     |
 | Repeats until termination ...                       |
 ----------------------------------------------------- 

 |  The server message encoding                        |
 | :-------------------------------------------------: |
 | 32 bits for the length of the response              |
 | The response encoded as a JSON string utf8          |
 ----------------------------------------------------- 