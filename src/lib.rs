//! ## A Toy RPC framework that is bug-prone, super slow, and hard to use
//! ### But.. why?
//! Well, I'm taking a class on distributed systems now and RPC was a topic so...
//! What's best to understand a topic that to implement it?
//!
//! So that's where this came from. I repeat, this is toy. please no one
//! use this
//!
//! ### General Information:
//! This library is built upon two main protocols/concepts:
//! - JSON serialization
//! - TCP
//!
//! `JSON` is used for everything as a convinient (yet probably slow) way
//! to serialize data types to strings. Then, the strings are converted to
//! `UTF8` bytes and passed down.
//! `TCP` is our choice of transport layer protocol. It makes moving the bytes
//! from point A to point B easy.. So why not.
//! Again, this is a toy. Not meant to be used for benchmarking or anything
//! ### How to use:
//! This library includes two main structs
//! a `Client` struct and a `Server` struct.
//! a server is meant to register RPC functions, and the client
//! can then call them.
//!
//! ### Examples:
//! #### Example Client
//! ```no_run
//! use rpc_toy::Client;
//! // You can create a new client using "new"
//! let mut client = Client::new("127.0.0.1:3001").unwrap();
//! // All arguments have to be passed in as a slice of `serde_json::Value`s
//! let one = serde_json::to_value(1u32).unwrap();
//! let two = serde_json::to_value(2u32).unwrap();
//! let args = vec![one, two];
//! // Use the `call` function to call remote procedures
//! let res = client.call("Add", &args).unwrap();
//!
//! let three: u32 = serde_json::from_value(res.unwrap()).unwrap();
//! assert_eq!(three, 3);
//! ```
//! #### Example Server
//! ```no_run
//! use rpc_toy::Server;
//! let mut server = Server::new();
//! server.register("Add", |args| {
//!     let one = args.get(0).unwrap();
//!     let two = args.get(1).unwrap();
//!     let one = serde_json::from_value::<u32>(one.clone()).unwrap();
//!     let two = serde_json::from_value::<u32>(two.clone()).unwrap();
//!     let three = one + two;
//!     return Some(serde_json::to_value(three).unwrap());
//! });
//! server.listen("127.0.0.1:3001").unwrap();
//! ```
//! ### Message encodings:
//!
//! |  The client message encoding                        |
//! | :-------------------------------------------------: |
//! | 32 bits for the length of the function name         |
//! | the name of the function                            |
//! | The length of the argument, or zero for termination |
//! | The argument encoded as JSON string utf8            |
//! | second argument length, or zero for termination     |
//! | Repeats until termination ...                       |
//! -----------------------------------------------------
//!
//! |  The server message encoding                        |
//! | :-------------------------------------------------: |
//! | 32 bits for the length of the response              |
//! | The response encoded as a JSON string utf8          |
//! -----------------------------------------------------
mod error;
pub use error::Error;
type Result<T, E = error::Error> = std::result::Result<T, E>;
use std::io::prelude::*;
use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
};

/// An RPC client
/// This is the main struct that should be used for
/// implementing an RPC client.
pub struct Client {
    stream: TcpStream,
}

impl Client {
    /// Creates a new client that connects to an RPC server
    /// # Arguments:
    /// - `addr` The address the TCP client should connect to
    ///     this should be in the form "host:port"
    /// # Example:
    /// ```rust
    /// use rpc_toy::Client;
    /// let client = Client::new("127.0.0.1:3001");
    /// ```
    pub fn new(addr: &str) -> Result<Self> {
        Ok(Client {
            stream: TcpStream::connect(addr)?,
        })
    }
    /// Invokes an RPC, this is the mechanism to "call" functions
    /// on a remote server
    ///
    /// # Arguments:
    /// - `fn_name`: The name of the function to call
    ///   NOTE: The server **MUST** have registered this function, otherwise
    ///   (currently) expect weird stuff to happen :)
    /// - `args` a slice of `serde_json::Value`s. This represents the arguments
    ///   that will be passed onto the server's functions
    /// # Returns:
    /// - a `Result<Option<serde_json::Value>>>`, which is `Ok` if nothing errored out
    ///   the `Option` will be `None` if this is a void function, otherwise it will be
    ///   `Some(value)` where `value` is a `serde_json::Value` representing the return value
    ///   of the function
    /// # Example:
    /// ```no_run
    /// use rpc_toy::Client;
    /// let mut client = Client::new("127.0.0.1:3001").unwrap();
    /// let one = serde_json::to_value(1u32).unwrap();
    /// let two = serde_json::to_value(2u32).unwrap();
    /// let args = vec![one, two];
    /// let res = client.call("Add", &args).unwrap();
    /// let three: u32 = serde_json::from_value(res.unwrap()).unwrap();
    /// assert_eq!(three, 3);
    /// ```
    pub fn call(
        &mut self,
        fn_name: &str,
        args: &[serde_json::Value],
    ) -> Result<Option<serde_json::Value>> {
        let mut bytes = Vec::new();
        let fn_name = fn_name.as_bytes();
        bytes.extend_from_slice(&(fn_name.len() as u32).to_be_bytes());
        bytes.extend_from_slice(fn_name);
        for arg in args {
            let arg = serde_json::to_string(&arg)?;
            let arg = arg.as_bytes();
            bytes.extend_from_slice(&(arg.len() as u32).to_be_bytes());
            bytes.extend_from_slice(arg);
        }
        bytes.extend_from_slice(&(0u32).to_be_bytes());
        self.stream.write_all(&bytes)?;
        let mut response_len = [0; 4];
        self.stream.read_exact(&mut response_len)?;
        let response_len = u32::from_be_bytes(response_len);
        if response_len == 0 {
            // void function
            return Ok(None);
        }
        let mut response = vec![0; response_len as usize];
        self.stream.read_exact(&mut response)?;
        let response = std::str::from_utf8(&response)?;
        Ok(Some(serde_json::from_str(response)?))
    }
}

use std::sync::Mutex;
type RPCFn = Box<dyn Fn(&[serde_json::Value]) -> Option<serde_json::Value> + Send>;

/// A struct representing an RPC server, this is to be
/// used to implement the server.
#[derive(Default)]
pub struct Server {
    // At the time this looks ugly and there is currently no
    // way to alias function traits :(
    fn_table: Mutex<HashMap<String, RPCFn>>,
}

impl Server {
    /// Creates a new RPC server
    pub fn new() -> Self {
        Self {
            fn_table: Mutex::new(HashMap::new()),
        }
    }

    /// Registers functions to be used in the RPC server
    /// only functions registered using this function can be
    /// called from the client
    ///
    /// # Arguments:
    /// - `fn_name` the name of the function to register, this **MUST**
    ///    be the same name the client expects to use
    /// - `function` the function to run once the RPC is invoked
    ///
    /// # Example:
    /// ```
    /// use rpc_toy::Server;
    /// let mut server = Server::new();
    /// server.register("Add", |args| {
    ///     let one = args.get(0).unwrap();
    ///     let two = args.get(1).unwrap();
    ///     let one = serde_json::from_value::<u32>(one.clone()).unwrap();
    ///     let two = serde_json::from_value::<u32>(two.clone()).unwrap();
    ///     let three = one + two;
    ///     return Some(serde_json::to_value(three).unwrap());
    /// });
    /// ```
    pub fn register<F>(&mut self, fn_name: &str, function: F)
    where
        F: Fn(&[serde_json::Value]) -> Option<serde_json::Value> + 'static + Send,
    {
        self.fn_table
            .lock()
            .unwrap()
            .insert(fn_name.to_string(), Box::new(function));
    }

    /// Listen to RPC connections
    /// This function must be run in order to start listening
    /// for calls over the network
    ///
    /// # Arguments:
    /// - `addr`: An address to bind to, must be in the form:
    ///     "host:port"
    /// # Examples:
    /// ```no_run
    /// use rpc_toy::Server;
    /// let mut server = Server::new();
    /// server.listen("127.0.0.1:3001").unwrap();
    /// ```
    pub fn listen(&self, addr: &str) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        for incoming in listener.incoming() {
            let stream = incoming?;
            crossbeam::thread::scope(move |s| {
                s.spawn(move |_| self.handle_client(stream));
            })
            .ok();
        }
        Ok(())
    }

    fn handle_client(&self, mut stream: TcpStream) -> Result<()> {
        // We first read the lenght of the name of the function
        // that should be encoded as a big endian 4 byte value
        let mut fn_name_len = [0; 4];
        while stream.read_exact(&mut fn_name_len).is_ok() {
            let fn_name_len = u32::from_be_bytes(fn_name_len);
            let mut fn_name = vec![0; fn_name_len as usize];
            // We then read the name of the function as a utf8 formatted
            // string
            stream.read_exact(&mut fn_name)?;
            let fn_name = std::str::from_utf8(&fn_name)?;
            // We check if the server has a function of that name
            // registered
            if self.fn_table.lock().unwrap().contains_key(fn_name) {
                // We read all the arguments
                let mut args = Vec::new();
                let mut arg_len = [0; 4];
                stream.read_exact(&mut arg_len)?;
                let mut arg_len = u32::from_be_bytes(arg_len);
                while arg_len != 0 {
                    let mut arg = vec![0; arg_len as usize];
                    stream.read_exact(&mut arg)?;
                    let arg_str = std::str::from_utf8(&arg)?;
                    args.push(serde_json::from_str(arg_str)?);
                    let mut arg_len_buff = [0; 4];
                    stream.read_exact(&mut arg_len_buff)?;
                    arg_len = u32::from_be_bytes(arg_len_buff);
                }
                // We call the function the server registered
                let res = (self.fn_table.lock().unwrap().get(fn_name).unwrap())(&args);
                match res {
                    Some(res) => {
                        let res_str = serde_json::to_string(&res)?;
                        let res = res_str.as_bytes();
                        stream.write_all(&(res.len() as u32).to_be_bytes())?;
                        stream.write_all(res)?;
                    }
                    None => {
                        stream.write_all(&(0 as u32).to_be_bytes())?;
                    }
                }
            } else {
                // TODO: Implement error handling
                // The server should send back an error to the client
                // letting it know that there is no function with that name
                break;
            }
        }
        Ok(())
    }
}
