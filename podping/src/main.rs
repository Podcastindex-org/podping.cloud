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
use std::env;
use drop_root::set_user_group;

//Globals ----------------------------------------------------------------------------------------------------
const ZMQ_SOCKET_ADDR: &str = "tcp://127.0.0.1:9999";
mod handler;
mod router;
type Response = hyper::Response<hyper::Body>;
type Error = Box<dyn std::error::Error + Send + Sync + 'static>;


//Structs ----------------------------------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct AppState {
    pub state_thing: String,
    pub remote_ip: String
}

#[derive(Debug)]
pub struct Context {
    pub state: AppState,
    pub req: Request<Body>,
    pub params: Params,
    body_bytes: Option<hyper::body::Bytes>,
}


//Functions --------------------------------------------------------------------------------------------------
#[tokio::main]
async fn main() {
    
    //TODO: Allow command line args to give a single publisher auth token which will override the "auth.db" check
    //and just use that one each time.  This would be for single use inside a publisher where there would be no
    //other publishers using the system.  This param could be passed to docker with an env


    //ZMQ socket version
    thread::spawn(move || {
        let context = zmq::Context::new();
        let mut requester = context.socket(zmq::REQ).unwrap(); 

        //Set up and connect the socket
        //requester.set_rcvtimeo(500);
        if requester.set_linger(0).is_err() {
            eprintln!("  Failed to set zmq to zero linger.");
        }
        if requester.connect(ZMQ_SOCKET_ADDR).is_err() {
            eprintln!("  Failed to connect to the hive-writer socket.");
        }

        //Spawn a queue checker threader.  Every X seconds, get all the pings from the Queue and attempt to write them 
        //to the socket that the Hive-writer should be listening on
        loop {
            thread::sleep(time::Duration::from_secs(20));

            println!("\n");
            println!("Start tickcheck...");            

            //Get the most recent X number of pings from the queue database
            let pinglist = handler::get_pings_from_queue();
            match pinglist {
                Ok(pings) => {
                    println!("  Flushing the queue...");
                    if pings.len() > 0 {
                        println!("  Found items...");
                    }

                    //Iterate through the pings and send each one to the hive-writer through the socket
                    for ping in pings {

                        println!("  Sending {} over the socket.", ping.url.clone());

                        match requester.send(ping.url.as_str(), 0) {
                            Ok(_) => {                                    
                                match requester.recv_msg(0) {
                                    Ok(message) => {
                                        let status_msg = message.as_str().clone().unwrap();
                                        println!("  Received reply {}", status_msg);
    
                                        if status_msg == "OK" || status_msg == "ERR" {
                                            //If the write was successful, remove this url from the queue
                                            match handler::delete_ping_from_queue(ping.url.clone()) {
                                                Ok(_) => {
                                                    println!("  Removed {} from the queue.", ping.url.clone());
                                                },
                                                Err(_) => {
                                                    eprintln!("  Failed to remove {} from the queue.", ping.url.clone());                                            
                                                }
                                            }                                                    
                                        }
                                    },
                                    Err(_) => {
                                        eprintln!("  No reply. Waiting...");                                                   
                                    }
                                }
                            },
                            Err(e) => {                            
                                eprintln!("  {}", e);                                    
                                if requester.disconnect(ZMQ_SOCKET_ADDR).is_err() {
                                    eprintln!("  Failed to disconnect zmq socket.");
                                }
                                requester = context.socket(zmq::REQ).unwrap(); 
                                //requester.set_rcvtimeo(500);
                                if requester.set_linger(0).is_err() {
                                    eprintln!("  Failed to set zmq to zero linger.");
                                }
                                if requester.connect(ZMQ_SOCKET_ADDR).is_err() {
                                    eprintln!("  Failed to re-connect to the hive-writer socket.");
                                }
                                break;
                            }
                        }

                        println!("  Done sending and receiving.");
                        println!("  Sleeping...");
                        thread::sleep(time::Duration::from_millis(300));
                    }
                },
                Err(e) => {
                    println!("  Error: [{}] checking queue.", e);
                }
            }

            println!("  End tickcheck...");

            //eprintln!("Timer thread exiting.");
        }

        //println!("Queue checker thread exited.");
    });

 

    let some_state = "state".to_string();

    let mut router: Router = Router::new();
    router.get("/", Box::new(handler::ping));
    router.get("/publishers", Box::new(handler::publishers));

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

    //If a "run as" user is set in the "PODPING_RUN_AS" environment variable, then switch to that user
    //and drop root privileges after we've bound to the low range socket
    match env::var("PODPING_RUNAS_USER") {
        Ok(runas_user) => {
            match set_user_group(runas_user.as_str(), "nogroup") {
                Ok(_) => {
                    println!("RunAs: {}", runas_user.as_str());
                },
                Err(e) => {
                    eprintln!("RunAs Error: {} - Check that your PODPING_RUNAS_USER env var is set correctly.", e); 
                }
            }
        },
        Err(_) => {
            eprintln!("ALERT: Use the PODPING_RUNAS_USER env var to avoid running as root.");
        }
    }

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