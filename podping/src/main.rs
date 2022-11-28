//Uses -----------------------------------------------------------------------------------------------------------------
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
use std::panic;
use capnp::data::Reader;
use drop_root::set_user_group;
use hyper::body::Buf;
use zmq::Message;
use dbif::{Reason, Medium};


//Globals --------------------------------------------------------------------------------------------------------------
const ZMQ_SOCKET_ADDR: &str = "127.0.0.1:9999";
const ZMQ_RECV_TIMEOUT: i32 = 10;
//const LOOP_TIMER_SECONDS: u64 = 1;
const LOOP_TIMER_MILLISECONDS: u64 = 500;
mod handler;
mod router;
type Response = hyper::Response<hyper::Body>;
type Error = Box<dyn std::error::Error + Send + Sync + 'static>;


//Structs --------------------------------------------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct AppState {
    pub state_thing: String,
    pub remote_ip: String,
    pub version: String,
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


//Capnproto ------------------------------------------------------------------------------------------------------------
pub mod plexo_message_capnp {
    include!("../plexo-schemas/built/dev/plexo/plexo_message_capnp.rs");
}
pub mod podping_capnp {
    include!("../podping-schemas/built/org/podcastindex/podping/podping_capnp.rs");
}
pub mod podping_reason_capnp {
    include!("../podping-schemas/built/org/podcastindex/podping/podping_reason_capnp.rs");
}
pub mod podping_medium_capnp {
    include!("../podping-schemas/built/org/podcastindex/podping/podping_medium_capnp.rs");
}
pub mod podping_write_capnp {
    include!("../podping-schemas/built/org/podcastindex/podping/podping_write_capnp.rs");
}
pub mod podping_write_error_capnp {
    include!("../podping-schemas/built/org/podcastindex/podping/podping_write_error_capnp.rs");
}
pub mod podping_hive_transaction_capnp {
    include!("../podping-schemas/built/org/podcastindex/podping/hivewriter/podping_hive_transaction_capnp.rs");
}
#[allow(unused_imports)]
use crate::podping_capnp::{podping};
use crate::plexo_message_capnp::{plexo_message};
use crate::podping_write_capnp::{podping_write};
use crate::podping_write_error_capnp::{podping_write_error};
use crate::podping_hive_transaction_capnp::{podping_hive_transaction};


//Main -----------------------------------------------------------------------------------------------------------------
#[tokio::main]
async fn main() {

    //TODO: Allow command line args to give a single publisher auth token which will override the "auth.db" check
    //and just use that one each time.  This would be for single use inside a publisher where there would be no
    //other publishers using the system.  This param could be passed to docker with an env

    //Get what version we are
    let version = env!("CARGO_PKG_VERSION");
    println!("Version: {}", version);
    println!("--------------------");

    //ZMQ socket version
    thread::spawn(move || {

        //Get the socket address to connect to
        println!("\nDiscovering ZMQ socket address...");
        let zmq_address;
        let env_zmq_socket_url = std::env::var("ZMQ_SOCKET_ADDR");
        if env_zmq_socket_url.is_ok() {
            zmq_address = "tcp://".to_owned() + env_zmq_socket_url.unwrap().as_str();
            println!(" - Trying environment var(ZMQ_SOCKET_ADDR): [{}]", zmq_address);
        } else {
            zmq_address = "tcp://".to_owned() + String::from(ZMQ_SOCKET_ADDR).as_str();
            println!(" - Trying localhost default: [{}].", zmq_address);
        }

        //Set up and connect the socket
        let context = zmq::Context::new();
        let mut requester = context.socket(zmq::PAIR).unwrap();
        if requester.set_rcvtimeo(ZMQ_RECV_TIMEOUT).is_err() {
            eprintln!("  Failed to set zmq receive timeout.");
        }
        if requester.set_linger(0).is_err() {
            eprintln!("  Failed to set zmq to zero linger.");
        }
        if requester.connect(&zmq_address).is_err() {
            eprintln!("  Failed to connect to the podping writer socket.");
        }

        println!("ZMQ socket: [{}] connected.", zmq_address);

        //Spawn a queue checker threader.  Every X seconds, get all the pings from the Queue and attempt to write them
        //to the socket that the Hive-writer should be listening on
        loop {
            let mut sent = 0;

            //Reset old inflight pings that may have never been sent
            if dbif::reset_pings_in_flight().is_err() {
                eprintln!("  Failed to reset old in-flight pings.");
            }

            //We always want to try and receive any waiting socket messages before moving on to sending
            receive_messages(&requester);

            //Get the most recent X number of pings from the queue database
            let pinglist = dbif::get_pings_from_queue(false);
            match pinglist {
                Ok(pings) => {
                    if pings.len() > 0 {
                        println!("  Sending: [{}] items to writer...", pings.len());
                    }

                    //Send any outstanding pings to the writer(s)
                    for ping in pings {
                        println!("    --Sending: [{}]...", ping.url.clone());

                        //Construct the capnp buffer
                        let mut podping_message = ::capnp::message::Builder::new_default();
                        let mut podping_write = podping_message.init_root::<podping_write::Builder>();
                        podping_write.set_iri(ping.url.as_str());

                        //Set the proper reason code (maps an internal enum to a capnp enum)
                        let pp_reason;
                        match ping.reason {
                            Reason::Live => pp_reason = podping_reason_capnp::PodpingReason::Live,
                            Reason::LiveEnd => pp_reason = podping_reason_capnp::PodpingReason::LiveEnd,
                            Reason::Update => pp_reason = podping_reason_capnp::PodpingReason::Update,
                        }
                        podping_write.set_reason(pp_reason);

                        //Set the proper medium code (maps an internal enum to a capnp enum)
                        let pp_medium;
                        match ping.medium {
                            Medium::Podcast => pp_medium = podping_medium_capnp::PodpingMedium::Podcast,
                            Medium::PodcastL => pp_medium = podping_medium_capnp::PodpingMedium::PodcastL,
                            Medium::Music => pp_medium = podping_medium_capnp::PodpingMedium::Music,
                            Medium::MusicL => pp_medium = podping_medium_capnp::PodpingMedium::MusicL,
                            Medium::Video => pp_medium = podping_medium_capnp::PodpingMedium::Video,
                            Medium::VideoL => pp_medium = podping_medium_capnp::PodpingMedium::VideoL,
                            Medium::Film => pp_medium = podping_medium_capnp::PodpingMedium::Film,
                            Medium::FilmL => pp_medium = podping_medium_capnp::PodpingMedium::FilmL,
                            Medium::Audiobook => pp_medium = podping_medium_capnp::PodpingMedium::Audiobook,
                            Medium::AudiobookL => pp_medium = podping_medium_capnp::PodpingMedium::AudiobookL,
                            Medium::Newsletter => pp_medium = podping_medium_capnp::PodpingMedium::Newsletter,
                            Medium::NewsletterL => pp_medium = podping_medium_capnp::PodpingMedium::NewsletterL,
                            Medium::Blog => pp_medium = podping_medium_capnp::PodpingMedium::Blog,
                            Medium::BlogL => pp_medium = podping_medium_capnp::PodpingMedium::BlogL,
                        }
                        podping_write.set_medium(pp_medium);

                        //Create a raw buffer that will hold the plexo wrapper
                        let mut write_message_buffer = Vec::new();
                        capnp::serialize::write_message(&mut write_message_buffer, &podping_message).unwrap();

                        //Write a podping_write message into the plexo wrapper
                        let mut message = ::capnp::message::Builder::new_default();
                        let mut plexo_message = message.init_root::<plexo_message::Builder>();
                        plexo_message.set_type_name("org.podcastindex.podping.hivewriter.PodpingWrite.capnp");
                        let podping_write_reader = Reader::from(write_message_buffer.as_slice());
                        plexo_message.set_payload(podping_write_reader);

                        //Attempt to send the message over the ZMQ socket
                        let mut send_buffer = Vec::new();
                        capnp::serialize::write_message(&mut send_buffer, &message).unwrap();
                        match requester.send(send_buffer, 0) {
                            Ok(_) => {
                                println!("      IRI sent.");
                                //If the write was successful, mark this ping as "in flight"
                                match dbif::set_ping_as_inflight(&ping) {
                                    Ok(_) => {
                                        println!("      Marked: [{}|{}|{}|{}] as in flight.",
                                             ping.url.clone(),
                                             ping.time,
                                             ping.reason,
                                             ping.medium
                                        );
                                    },
                                    Err(_) => {
                                        eprintln!("      Failed to mark: [{}|{}|{}|{}] as in flight.",
                                                 ping.url.clone(),
                                                 ping.time,
                                                 ping.reason,
                                                 ping.medium
                                        );
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("      {}", e);
                                if requester.disconnect(&zmq_address).is_err() {
                                    eprintln!("      Failed to disconnect zmq socket.");
                                }
                                requester = context.socket(zmq::PAIR).unwrap();
                                if requester.set_rcvtimeo(ZMQ_RECV_TIMEOUT).is_err() {
                                    eprintln!("      Failed to set zmq receive timeout.");
                                }
                                if requester.set_linger(0).is_err() {
                                    eprintln!("      Failed to set zmq to zero linger.");
                                }
                                if requester.connect(&zmq_address).is_err() {
                                    eprintln!("      Failed to re-connect to the hive-writer socket.");
                                }
                                break;
                            }
                        }

                        //Again, try to receive any messages waiting on the socket so that we effectively
                        //interleave the receives and sends to speed things up and not have one "block" the other
                        receive_messages(&requester);
                        sent += 1;
                    }
                },
                Err(e) => {
                    println!("  Error: [{}] checking queue.", e);
                }
            }

            if sent < 5 {
                thread::sleep(time::Duration::from_millis(LOOP_TIMER_MILLISECONDS));
            }
        }
    });

    // We want a thread panic on the ZMQ thread to exit the whole process
    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // invoke the default handler and exit the process
        orig_hook(panic_info);
        std::process::exit(1);
    }));

    let some_state = "state".to_string();

    let mut router: Router = Router::new();
    router.get("/", Box::new(handler::ping));
    router.get("/publishers", Box::new(handler::publishers));

    let shared_router = Arc::new(router);
    let new_service = make_service_fn(move |conn: &AddrStream| {
        let app_state = AppState {
            state_thing: some_state.clone(),
            remote_ip: conn.remote_addr().to_string().clone(),
            version: version.to_string(),
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


//Functions ------------------------------------------------------------------------------------------------------------
fn receive_messages(requester: &zmq::Socket) -> bool {

    //Receive any messages from the writer(s)
    //TODO: handle error scenario here where hive writer returns a write error by marking the url not inflight
    let mut response =  Message::new();
    match requester.recv(&mut response, 0) {
        Ok(_) => {
            println!("  Incoming writer message...");

            //Read the plexo message from the ZMQ socket
            let message_reader = capnp::serialize::read_message(
                response.reader(),
                ::capnp::message::ReaderOptions::new()
            ).unwrap();
            let plexo_message = message_reader.get_root::<plexo_message::Reader>().unwrap();
            let plexo_payload_type = plexo_message.get_type_name().unwrap();
            println!("    --Plexo payload: [{:#?}]", plexo_payload_type);

            //Was this a writer error?
            if plexo_payload_type == "org.podcastindex.podping.hivewriter.PodpingWriteError.capnp" {
                //Extract the write error from the plexo message
                let hivetx_reader = capnp::serialize::read_message(
                    plexo_message.get_payload().unwrap(),
                    ::capnp::message::ReaderOptions::new()
                ).unwrap();
                let hive_write_error = hivetx_reader.get_root::<podping_write_error::Reader>().unwrap();

                //Does it have a valid podping_write message attached?
                if hive_write_error.has_podping_write() {
                    //Extract the podping write
                    // let podping_write_reader = capnp::serialize::read_message(
                    //     hive_write_error.get_podping_write().unwrap(),
                    //     ::capnp::message::ReaderOptions::new()
                    // ).unwrap();
                    let podping_write_failure = hive_write_error.get_podping_write().unwrap();

                    let iri_to_remove = podping_write_failure.get_iri().unwrap();
                    println!("    --Removing: [{:#?}] from queue...", iri_to_remove);
                    if dbif::delete_ping_from_queue(iri_to_remove.to_string()).is_err() {
                        eprintln!("Error removing ping: [{}] from queue.", iri_to_remove);
                    }
                }
            }

            //Extract the hive_transaction from the plexo message
            let hivetx_reader = capnp::serialize::read_message(
                plexo_message.get_payload().unwrap(),
                ::capnp::message::ReaderOptions::new()
            ).unwrap();
            let hive_transaction = hivetx_reader.get_root::<podping_hive_transaction::Reader>().unwrap();

            //If this reply message has podpings in it, remove them from the queue
            if hive_transaction.has_podpings() {
                println!("    --Hive tx id: [{:#?}]", hive_transaction.get_hive_tx_id().unwrap());
                println!("    --Hive td details: [https://hive.ausbit.dev/tx/{}]",
                         hive_transaction.get_hive_tx_id().unwrap()
                );
                println!("    --Hive block num: [{:#?}]", hive_transaction.get_hive_block_num());

                if hive_transaction.get_hive_block_num() == 0 {
                    return false;
                }

                let podpings_written = hive_transaction.get_podpings().unwrap();
                for podping_written in podpings_written {
                    let podping_iris = podping_written.get_iris().unwrap();
                    for podping_iri in podping_iris {
                        let iri_to_remove = podping_iri.unwrap();
                        println!("    --Removing: [{:#?}] from queue...", iri_to_remove);
                        if dbif::delete_ping_from_queue(iri_to_remove.to_string()).is_err() {
                            eprintln!("Error removing ping: [{}] from queue.", iri_to_remove);
                        }
                    }
                    println!("    --Removed: [{}] iri's from the queue.", podping_iris.len());
                }
            }

            true
        },
        Err(_) => {
            //eprintln!("  No reply. Waiting...");
            false
        }
    }
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

#[allow(dead_code)]
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}