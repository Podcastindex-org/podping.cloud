//use bytes::Bytes;
use hyper::{
    body::to_bytes,
    service::{make_service_fn, service_fn},
    Body, Request, Server,
};
use route_recognizer::Params;
use router::Router;
use std::sync::Arc;
use hyper::server::conn::AddrStream;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;
use std::sync::mpsc;
use std::thread;
use handler::Ping;

use eventual::*;

mod handler;
mod router;

type Response = hyper::Response<hyper::Body>;
type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Clone, Debug)]
pub struct AppState {
    pub state_thing: String,
    pub remote_ip: String
}

#[tokio::main]
async fn main() {
    // First thread owns sender
    thread::spawn(move || {
        let timer = Timer::new();
        let ticks = timer.interval_ms(3000).iter();
        for _ in ticks {
            let pinglist = handler::get_pings_from_queue();
            match pinglist {
                Ok(pings) => {
                    if pings.len() > 0 {
                        println!("\nFlushing the queue...");
                    }
                    for ping in pings {
                        //Attempt to write the url to hive
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
                            }
                        }
                    }
                },
                Err(_) => {
                    println!("  Queue is empty.");
                }
            }

        }
    });

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

    //Create an inter-thread communications channel
    //let (sender, receiver) = channel();
}

// pub fn listen(tx: Sender<()>) {
//     let timer = Timer::new();
//     let ticks = timer.interval_ms(3000).iter();
//     for _ in ticks {
//         handler::get_pings_from_queue();
//     }
// }

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