extern crate http;
extern crate uuid;
extern crate iron;
extern crate logger;
extern crate router;
extern crate persistent;
extern crate typemap;
extern crate serialize;

use iron::{Chain, ChainBuilder, Iron, IronResult, Plugin, Request, Response};
use persistent::State;
use router::{Router, Params};
use serialize::json;
use uuid::Uuid;
use todos::Todo;

mod todos;

struct TodoList; // "Phantom" type for iron/persistent.
impl ::typemap::Assoc<Vec<Todo>> for TodoList {}

fn main() {
    let mut router = Router::new();
    router
        .options("/", empty_success)
        .get("/", list_todos)
        .post("/", create_todo)
        .delete("/", delete_todos)
        .options("/:todoid", empty_success)
        .get("/:todoid", get_todo)
        .patch("/:todoid", update_todo)
        .delete("/:todoid", delete_todo);

    let mut chain = ChainBuilder::new(router);
    let (logger_before, logger_after) = logger::Logger::new(None);
    chain.link_before(logger_before);
    chain.link_before(State::<TodoList,Vec<Todo>>::one(vec![]));
    chain.link_after(set_cors_headers);
    chain.link_after(content_type_json);
    chain.link_after(logger_after);

    Iron::new(chain).listen(::std::io::net::ip::Ipv4Addr(127, 0, 0, 1), 3000);
    println!("Iron listening on http://localhost:3000/");
}

fn empty_success(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with(iron::status::Ok, ""))
}

fn list_todos(req: &mut Request) -> IronResult<Response> {
    let rwlock = req.get::<State<TodoList,Vec<Todo>>>().unwrap();
    let todos = rwlock.read();
    Ok(Response::with(::iron::status::Ok, json::encode(&*todos)))
}

fn get_todo(req: &mut Request) -> IronResult<Response> {
    let todoid = Uuid::parse_str(req.extensions.find::<Router, Params>().unwrap()["todoid"].as_slice()).unwrap();
    let rwlock = req.get::<State<TodoList,Vec<Todo>>>().unwrap();
    let todos = rwlock.read();
    match todos.iter().find(|todo| todo.id == todoid) {
        Some(todo) => Ok(Response::with(::iron::status::Ok, json::encode(todo))),
        None => Ok(Response::with(::iron::status::NotFound, ""))
    }
}

fn create_todo(req: &mut Request) -> IronResult<Response> {
    match Todo::new_from_json_str(req.body.as_slice(),
                                  format!("{}", req.url).as_slice()) {
        Ok(todo) => {
            let rwlock = req.get::<State<TodoList,Vec<Todo>>>().unwrap();
            let mut todos = rwlock.write();
            (*todos).push(todo.clone());
            Ok(Response::with(::iron::status::Ok, json::encode(&todo)))
        },
        Err(s) => Ok(Response::with(::iron::status::BadRequest, s))
    }
}

fn update_todo(req: &mut Request) -> IronResult<Response> {
    let todoid = Uuid::parse_str(req.extensions.find::<Router, Params>().unwrap()["todoid"].as_slice()).unwrap();
    let rwlock = req.get::<State<TodoList,Vec<Todo>>>().unwrap();
    let mut todos = rwlock.write();
    let idx = todos.iter().position(|todo| todo.id == todoid).unwrap();
    let todo = todos.deref_mut().get_mut(idx);
    match todo.update_from_json_str(req.body.as_slice()) {
        Ok(_) => Ok(Response::with(::iron::status::Ok, json::encode(todo))),
        Err(msg) => Ok(Response::with(::iron::status::BadRequest, msg))
    }
}

fn delete_todos(req: &mut Request) -> IronResult<Response> {
    let rwlock = req.get::<State<TodoList,Vec<Todo>>>().unwrap();
    let mut todos = rwlock.write();
    todos.clear();
    Ok(Response::with(::iron::status::Ok, ""))
}

fn delete_todo(req: &mut Request) -> IronResult<Response> {
    let todoid = Uuid::parse_str(req.extensions.find::<Router, Params>().unwrap()["todoid"].as_slice()).unwrap();
    let rwlock = req.get::<State<TodoList,Vec<Todo>>>().unwrap();
    let mut todos = rwlock.write();
    todos.retain(|todo| todo.id != todoid);
    Ok(Response::with(::iron::status::Ok, ""))
}

fn content_type_json(_: &mut Request, res: &mut Response) -> IronResult<()> {
    res.headers.content_type = Some(::http::headers::content_type::MediaType {
        type_: "application".to_string(),
        subtype: "json".to_string(),
        parameters: vec![]
    });
    Ok(())
}

fn set_cors_headers(req: &mut Request, res: &mut Response) -> IronResult<()> {
    let _ = res.headers.insert_raw("access-control-allow-origin".to_string(), b"*");
    if req.method == ::http::method::Options {
        let _ = res.headers.insert_raw("access-control-allow-headers".to_string(), b"accept, content-type");
        let _ = res.headers.insert_raw("access-control-allow-methods".to_string(), b"GET,POST,DELETE,OPTIONS,PATCH");
    }
    Ok(())
}
