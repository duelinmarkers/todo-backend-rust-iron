extern crate http;
extern crate uuid;
extern crate iron;
extern crate logger;
extern crate router;
extern crate persistent;
extern crate typemap;
extern crate serialize;

use std::io::net::ip::Ipv4Addr;
use http::method::Options;
use http::headers::content_type::MediaType;
use iron::{Iron, Chain, Request, Response, Server, Status, Continue, Unwind, FromFn};
use persistent::Persistent;
use router::{Router, Params};
use typemap::Assoc;
use std::sync::{Arc, RWLock};
use serialize::json;
use uuid::Uuid;
use todos::Todo;

mod todos;

struct TodoList; // "Phantom" type for iron/persistent.
impl Assoc<Arc<RWLock<Vec<Todo>>>> for TodoList {}

fn set_cors_headers(req: &mut Request, res: &mut Response) -> Status {
    let _ = res.headers.insert_raw("access-control-allow-origin".to_string(), b"*");
    if req.method == Options {
        let _ = res.headers.insert_raw("access-control-allow-headers".to_string(), b"accept, content-type");
        let _ = res.headers.insert_raw("access-control-allow-methods".to_string(), b"GET,POST,DELETE,OPTIONS,PATCH");
    }
    Continue
}

fn empty_success(_req: &mut Request, res: &mut Response) -> Status {
    let _ = res.serve(::http::status::Ok, "");
    Unwind
}

fn content_type_json(res: &mut Response) {
    res.headers.content_type = Some(MediaType {
        type_: "application".to_string(),
        subtype: "json".to_string(),
        parameters: vec![]
    });
}

fn list_todos(req: &mut Request, res: &mut Response) -> Status {
    content_type_json(res);
    let todos = req.extensions.find::<TodoList, Arc<RWLock<Vec<Todo>>>>().unwrap().read();
    let _ = res.serve(::http::status::Ok, json::encode(&*todos));
    Unwind
}

fn get_todo(req: &mut Request, res: &mut Response) -> Status {
    content_type_json(res);
    let todoid = Uuid::parse_string(req.extensions.find::<Router, Params>().unwrap()["todoid"].as_slice()).unwrap();
    let todos = req.extensions.find::<TodoList, Arc<RWLock<Vec<Todo>>>>().unwrap().read();
    match todos.iter().find(|todo| todo.id == todoid) {
        Some(todo) => { let _ = res.serve(::http::status::Ok, json::encode(todo)); },
        None => { let _ = res.serve(::http::status::NotFound, ""); }
    }
    Unwind
}

fn create_todo(req: &mut Request, res: &mut Response) -> Status {
    content_type_json(res);
    match Todo::new_from_json_str(req.body.as_slice()) {
        Ok(todo) => {
            let mut todos = req.extensions.find::<TodoList, Arc<RWLock<Vec<Todo>>>>().unwrap().write();
            (*todos).push(todo.clone());
            let _ = res.serve(::http::status::Ok, json::encode(&todo));
        }
        Err(s) => { let _ = res.serve(::http::status::BadRequest, s); }
    }
    Unwind
}

fn update_todo(req: &mut Request, res: &mut Response) -> Status {
    content_type_json(res);
    let todoid = Uuid::parse_string(req.extensions.find::<Router, Params>().unwrap()["todoid"].as_slice()).unwrap();
    let mut todos = req.extensions.find::<TodoList, Arc<RWLock<Vec<Todo>>>>().unwrap().write();
    let idx = todos.iter().position(|todo| todo.id == todoid).unwrap();
    let todo = todos.get_mut(idx);
    match todo.update_from_json_str(req.body.as_slice()) {
        Ok(_) => {
            let _ = res.serve(::http::status::Ok, json::encode(todo));
        }
        Err(msg) => {
            let _ = res.serve(::http::status::BadRequest, msg);
        }
    }
    Unwind
}

fn delete_todos(req: &mut Request, res: &mut Response) -> Status {
    let mut todos = req.extensions.find::<TodoList, Arc<RWLock<Vec<Todo>>>>().unwrap().write();
    todos.clear();
    let _ = res.serve(::http::status::Ok, "");
    Unwind
}

fn delete_todo(req: &mut Request, res: &mut Response) -> Status {
    let todoid = Uuid::parse_string(req.extensions.find::<Router, Params>().unwrap()["todoid"].as_slice()).unwrap();
    let mut todos = req.extensions.find::<TodoList, Arc<RWLock<Vec<Todo>>>>().unwrap().write();
    todos.retain(|todo| todo.id != todoid);
    let _ = res.serve(::http::status::Ok, "");
    Unwind
}

fn main() {
    let mut server: Server = Iron::new();

    server.chain.link(logger::Logger::new(None));
    server.chain.link(FromFn::new(set_cors_headers));

    let todolist : Persistent<Vec<Todo>,TodoList> = Persistent::new(vec![]);
    server.chain.link(todolist);

    let mut router = Router::new();

    router.options("/", FromFn::new(empty_success));
    router.get("/", FromFn::new(list_todos));
    router.post("/", FromFn::new(create_todo));
    router.delete("/", FromFn::new(delete_todos));

    router.options("/:todoid", FromFn::new(empty_success));
    router.get("/:todoid", FromFn::new(get_todo));
    router.patch("/:todoid", FromFn::new(update_todo));
    router.delete("/:todoid", FromFn::new(delete_todo));

    server.chain.link(router);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}
