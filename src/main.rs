extern crate http;
extern crate iron;
extern crate logger;
extern crate router;

use std::io::net::ip::Ipv4Addr;
use http::method::Options;
use iron::{Iron, Chain, Request, Response, Server, Status, Continue, Unwind, FromFn};

fn set_cors_headers(req: &mut Request, res: &mut Response) -> Status {
    let _ = res.headers.insert_raw("access-control-allow-origin".to_string(), b"*");
    if req.method == Options {
        let _ = res.headers.insert_raw("access-control-allow-headers".to_string(), b"accept, content-type");
    }
    Continue
}

fn empty_success(_req: &mut Request, res: &mut Response) -> Status {
    let _ = res.serve(::http::status::Ok, "");
    Unwind
}

fn main() {
    let mut router = router::Router::new();

    router.options("/", FromFn::new(empty_success));
    router.get("/", FromFn::new(empty_success));

    let mut server: Server = Iron::new();
    server.chain.link(logger::Logger::new(None));
    server.chain.link(FromFn::new(set_cors_headers));
    server.chain.link(router);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}
