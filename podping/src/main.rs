use hyper::{
    body::to_bytes,
    service::{make_service_fn, service_fn},
    Body, Request, Server,
};
use route_recognizer::Params;
use router::Router;
use std::sync::Arc;
use hyper::server::conn::AddrStream;
use std::thread;
use std::time;
//use std::net::{TcpStream, Shutdown};
//use std::time::Duration;

mod handler;
mod router;

type Response = hyper::Response<hyper::Body>;
type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Clone, Debug)]
pub struct AppState {
    pub state_thing: String,
    pub remote_ip: String
}

//const ZMQ_SOCKET_ADDR: &str = "tcp://127.0.0.1:5555";

#[tokio::main]
async fn main() {

    thread::spawn(move || {


        // let mut stream: TcpStream;
        // loop {
        //     let newstream = TcpStream::connect("localhost:9999");
        //     if newstream.is_err() {
        //         eprintln!("Failed to connect to hive-writer socket");
        //     } else {
        //         stream = newstream.unwrap();
        //         break;
        //     }    
        //     thread::sleep(time::Duration::from_secs(1));
        // }

        loop {
            thread::sleep(time::Duration::from_secs(3));
            println!("Start tickcheck...");            
            let pinglist = handler::get_pings_from_queue();
            match pinglist {
                Ok(pings) => {
                    println!("  Flushing the queue...");
                    if pings.len() > 0 {
                        println!("  Found {} feeds...", pings.len());
                    }
                    for ping in pings {
                        //Attempt to write the url to hive
                        // match handler::hive_notify(&mut stream, ping.url.as_str()) {
                        match handler::hive_notify(ping.url.as_str()) {
                            Ok(result) => {
                                println!("  {}", result);
                                //If the write was successful, remove this url from the queue
                                match handler::delete_ping_from_queue(ping.url.clone()) {
                                    Ok(_) => {
                                        println!("  Removed {} from the queue.", ping.url.clone());
                                    },
                                    Err(_) => {
                                        eprintln!("  Failed to remove {} from the queue.", ping.url.clone());                                            
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("  {}", e);
                                break;
                            }
                        }

                        //Back off a bit
                        thread::sleep(time::Duration::from_secs(1));
                    }
                },
                Err(_) => {
                    println!("  Queue is empty.");
                }
            }
            println!("  End tickcheck...");            
        }
    });
    
    // let control_thread = thread::spawn(move || {
    //     //Spawn a queue checker threader.  Every 3 seconds, get all the pings from the Queue and attempt to write them 
    //     //to the socket that the Hive-writer should be listening on
    //     let timer_thread = thread::spawn(move || {
    //         let mut context = zmq::Context::new(); 
    //         let timer = Timer::new();
    //         let ticks = timer.interval_ms(3000).iter();

    //         //requester.setsockopt
    //         let mut requester = context.socket(zmq::REQ).unwrap(); 
    //         requester.set_rcvtimeo(500);
    //         requester.set_linger(0);
    //         if requester.connect(ZMQ_SOCKET_ADDR).is_err() {
    //             eprintln!("  Failed to connect to the hive-writer socket.");
    //         }

    //         //The main timer loop
    //         for _ in ticks {

    //             println!("Start tickcheck...");            


                
    //             //Get the most recent X number of pings from the queue database
    //             let pinglist = handler::get_pings_from_queue();
    //             match pinglist {
    //                 Ok(pings) => {
    //                     println!("  Flushing the queue...");
    //                     if pings.len() > 0 {
    //                         println!("  Found items...");
    //                     }

    //                     //Iterate through the pings and send each one to the hive-writer through the socket
    //                     for ping in pings {

    //                         println!("  Sending {} over the socket.", ping.url.clone());

    //                         match requester.send(ping.url.as_str(), 0) {
    //                             Ok(_) => {                                    
    //                                 match requester.recv_msg(0) {
    //                                     Ok(message) => {
    //                                         let status_msg = message.as_str().clone().unwrap();
    //                                         println!("  Received reply {}", status_msg);
        
    //                                         if status_msg == "OK" || status_msg == "ERR" {
    //                                             //If the write was successful, remove this url from the queue
    //                                             match handler::delete_ping_from_queue(ping.url.clone()) {
    //                                                 Ok(_) => {
    //                                                     println!("  Removed {} from the queue.", ping.url.clone());
    //                                                 },
    //                                                 Err(_) => {
    //                                                     eprintln!("  Failed to remove {} from the queue.", ping.url.clone());                                            
    //                                                 }
    //                                             }                                                    
    //                                         }
    //                                     },
    //                                     Err(_) => {
    //                                         eprintln!("  No reply. Waiting...");                                                   
    //                                     }
    //                                 }
    //                             },
    //                             Err(e) => {
    //                                 eprintln!("  {}", e);                                    
    //                                 requester.disconnect(ZMQ_SOCKET_ADDR);
    //                                 requester = context.socket(zmq::REQ).unwrap(); 
    //                                 requester.set_rcvtimeo(500);
    //                                 requester.set_linger(0);
    //                                 if requester.connect(ZMQ_SOCKET_ADDR).is_err() {
    //                                     eprintln!("  Failed to connect to the hive-writer socket.");
    //                                 }
    //                             }
    //                         }

    //                         println!("Done with socket.");
    //                     }
    //                 },
    //                 Err(e) => {
    //                     println!("  Error: [{}] checking queue.", e);
    //                 }
    //             }

    //             println!("  End tickcheck...");
    //         }

    //         eprintln!("Timer thread exiting.");
    //     });

    //     let res = timer_thread.join();
    //     println!("Timer thread finished: {:#?}", res);
    // });

 

    let some_state = "state".to_string();

    let mut router: Router = Router::new();
    router.get("/", Box::new(handler::ping));

    let shared_router = Arc::new(router);
    let new_service = make_service_fn(move |conn: &AddrStream| {
        let app_state = AppState {
            state_thing: some_state.clone(),
            remote_ip: conn.remote_addr().to_string().clone()
        };

        let router_capture = shared_router.clone();
        async {
            Ok::<_, Error>(service_fn(move |req| {
                route(router_capture.clone(), req, app_state.clone())
            }))
        }
    });

    let addr = "0.0.0.0:80".parse().expect("address creation works");
    let server = Server::bind(&addr).serve(new_service);
    println!("Listening on http://{}", addr);
    let _ = server.await;
}

async fn route(
    router: Arc<Router>,
    req: Request<hyper::Body>,
    app_state: AppState,
) -> Result<Response, Error> {
    let found_handler = router.route(req.uri().path(), req.method());
    let resp = found_handler
        .handler
        .invoke(Context::new(app_state, req, found_handler.params))
        .await;
    Ok(resp)
}

#[derive(Debug)]
pub struct Context {
    pub state: AppState,
    pub req: Request<Body>,
    pub params: Params,
    body_bytes: Option<hyper::body::Bytes>,
}

impl Context {
    pub fn new(state: AppState, req: Request<Body>, params: Params) -> Context {
        Context {
            state,
            req,
            params,
            body_bytes: None,
        }
    }

    pub async fn body_json<T: serde::de::DeserializeOwned>(&mut self) -> Result<T, Error> {
        let body_bytes = match self.body_bytes {
            Some(ref v) => v,
            _ => {
                let body = to_bytes(self.req.body_mut()).await?;
                self.body_bytes = Some(body);
                self.body_bytes.as_ref().expect("body_bytes was set above")
            }
        };
        Ok(serde_json::from_slice(&body_bytes)?)
    }
}