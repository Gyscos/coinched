use super::game_manager::GameManager;

use std::sync::Arc;
use std::str::FromStr;

use rustc_serialize::json;

use iron;
use iron::prelude::*;

pub struct Server {
    port: u16,
    manager: Arc<GameManager>,
}

struct Router { manager: Arc<GameManager> }

#[derive(RustcEncodable)]
struct HelpAction {
    href: &'static str,
    help: &'static str,
    method: &'static str,
}

#[derive(RustcEncodable)]
struct HelpMessage {
    title: &'static str,
    actions: Vec<HelpAction>,
}

fn help_message() -> String {

    json::encode(&HelpMessage {
        title: "Help Page",
        actions: vec![
            HelpAction {
                href: "/join",
                method: "GET",
                help: "Join a new game.",
            },
            HelpAction {
                href: "/hand/[PLAYER_ID]",
                method: "GET",
                help: "Checks the current hand.",
            },
        ]
    }).unwrap()
}


fn help_resp() -> IronResult<Response> {
    let content_type: iron::mime::Mime = "application/json".parse::<iron::mime::Mime>().unwrap();
    return Ok(Response::with((content_type,
                              iron::status::NotFound,
                              help_message())));
}

fn err_resp(msg: &str) -> IronResult<Response> {
    let content_type: iron::mime::Mime = "application/json".parse::<iron::mime::Mime>().unwrap();

    #[derive(RustcEncodable)]
    struct Error<'a>{
        error: &'a str,
    }

    return Ok(Response::with((content_type,
                              iron::status::Ok,
                              json::encode(&Error { error: msg }).unwrap(),
                              )));
}

impl iron::Handler for Router {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {

        if req.url.path.is_empty() {
            // ?!?
            panic!("Empty request path should never happen.");
        }


        // Weird deref trick to go from &String to &str
        let action: &str = &*req.url.path[0];

        let content_type: iron::mime::Mime = "application/json".parse::<iron::mime::Mime>().unwrap();

        match req.method {
            iron::method::Options => {
                if ["hand", "trick", "contracts", "last_trick", "scores"].contains(&action) {
                    Ok(Response::with((iron::modifiers::Header(
                                           iron::headers::Allow(
                                               vec![
                                                   iron::method::Get,
                                                   iron::method::Options])),
                                       iron::status::Ok)))
                } else if ["pass", "coinche", "bid", "play", "join"].contains(&action) {
                    Ok(Response::with((iron::modifiers::Header(
                                           iron::headers::Allow(
                                               vec![
                                                   iron::method::Post,
                                                   iron::method::Options])),
                                       iron::status::Ok)))
                } else {
                    help_resp()
                }
            },
            iron::method::Get => {
                let response = match action {
                    "wait" => {
                        if req.url.path.len() != 3 {
                            return err_resp("missing parameters (Usage: /wait/[PID]/[EID])");
                        }
                        let player_id = match u32::from_str(&*req.url.path[1]) {
                            Ok(id) => id,
                            Err(e) => return err_resp(&format!("Invalid playerID: `{}` ({})", req.url.path[1], e)),
                        };
                        let event_id = match usize::from_str(&*req.url.path[2]) {
                            Ok(id) => id,
                            Err(e) => return err_resp(&format!("Invalid eventID: `{}` ({})", req.url.path[2], e)),
                        };
                        match self.manager.wait(player_id, event_id) {
                            Err(err) => return err_resp(&format!("{}", err)),
                            Ok(event) => json::encode(&event).unwrap(),
                        }
                    },
                    "hand" => "".to_string(),
                    "trick" => "".to_string(),
                    "contracts" => "".to_string(),
                    "last_trick" => "".to_string(),
                    "scores" => "".to_string(),
                    _ => return help_resp(),
                };

                Ok(Response::with((content_type, iron::status::Ok, response)))

            },
            iron::method::Post => {
                // Read the JSON body
                // ...

                let response = match action {
                    "join" => match self.manager.join() {
                        Err(msg) => return err_resp(&msg),
                        Ok(info) => json::encode(&info).unwrap(),
                    },
                    _ => return help_resp(),
                };

                Ok(Response::with((content_type, iron::status::Ok, response)))
            },
            _ => help_resp(),
        }
    }
}

impl Server {
    pub fn new(port: u16) -> Server {
        Server {
            port: port,
            manager: Arc::new(GameManager::new()),
        }
    }

    pub fn run(self) {
        let port = self.port;
        println!("Listening on port {}", port);

        let router = Router { manager: self.manager.clone() };

        Iron::new(router).http(("localhost", port)).unwrap();


    }
}