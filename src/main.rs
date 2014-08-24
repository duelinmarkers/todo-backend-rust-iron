extern crate http;
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
use typemap::Assoc;
use std::sync::{Arc, RWLock};
use serialize::json;

#[deriving(Show, Clone, Encodable)]
struct Todo {
    title: String,
    order: Option<f64>,
    completed: bool
}

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
    let todos : &Vec<Todo> = &*req.extensions.find::<TodoList, Arc<RWLock<Vec<Todo>>>>().unwrap().read();
    let _ = res.serve(::http::status::Ok, json::encode(&todos));
    Unwind
}

fn fresh_todo(s: &str) -> Result<Todo, String> {
    match json::from_str(s) {
        Ok(json) => {
            match json.find(&"title".to_string()) {
                Some(title) => Ok(Todo {
                                   title: title.as_string().unwrap().to_string(),
                                   order: match json.find(&"order".to_string()) {
                                       Some(j) => j.as_f64(),
                                       None => None
                                   },
                                   completed: false
                               }),
                _ => { println!("bad or missing title!"); Err("bad or missing title!".to_string()) }
            }
        },
        Err(builder_error) => Err(format!("{}", builder_error))
    }
}

#[test]
fn test_parse_complete_todo() {
    assert_eq!("a todo".to_string(), fresh_todo("{\"title\": \"a todo\", \"order\":100}").unwrap().title);
}
#[test]
fn test_parse_incomplete_todo() {
    assert_eq!("a todo".to_string(), fresh_todo("{\"title\": \"a todo\"}").unwrap().title);
}

fn create_todo(req: &mut Request, res: &mut Response) -> Status {
    println!("body: {}", req.body);
    content_type_json(res);
    match fresh_todo(req.body.as_slice()) {
        Ok(todo) => {
            match req.extensions.find::<TodoList, Arc<RWLock<Vec<Todo>>>>() {
                Some(lock) => {
                    let mut todos = lock.write();
                    (*todos).push(todo.clone());
                    let _ = res.serve(::http::status::Ok, json::encode(&todo));
                },
                None => println!("Got no persistent")
            }
        }
        Err(s) => {
            // TODO respond 4xx
            println!("{}", s)
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

fn main() {
    let mut server: Server = Iron::new();

    server.chain.link(logger::Logger::new(None));
    server.chain.link(FromFn::new(set_cors_headers));

    let todolist : Persistent<Vec<Todo>,TodoList> = Persistent::new(vec![]);
    server.chain.link(todolist);

    let mut router = router::Router::new();
    router.options("/", FromFn::new(empty_success));
    router.get("/", FromFn::new(list_todos));
    router.post("/", FromFn::new(create_todo));
    router.delete("/", FromFn::new(delete_todos));

    server.chain.link(router);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}
