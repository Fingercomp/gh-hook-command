extern crate actix_web;
extern crate crypto;
extern crate futures;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate toml;

mod app;
mod config;

use std::sync::Arc;
use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

use actix_web::{
    server,
    AsyncResponder,
    App,
    HttpMessage,
    HttpRequest,
    HttpResponse,
    Responder
};
use actix_web::dev::AsyncResult;
use actix_web::http::Method;
use crypto::mac::{Mac, MacResult};
use crypto::hmac::Hmac;
use crypto::sha1::Sha1;
use futures::Future;

use ::config::Config;
use ::app::State;

fn from_hex(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.len() % 2 == 1 {
        return None;
    }

    if !bytes.iter().all(|x| x.is_ascii_hexdigit()) {
        return None;
    }

    let nibbles: Vec<u8> = bytes.iter().map(|&x| {
        char::from(x).to_digit(16).unwrap() as u8
    }).collect();

    Some(nibbles.as_slice()
         .chunks(2)
         .map(|x| {
             let (n1, n2) = (x[0], x[1]);
             (n1 << 4 | n2) as u8
         })
         .collect())
}

fn run_command(command: &str, input: &[u8]) {
    match Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            match child.stdin.as_mut() {
                Some(stdin) => {
                    if let Err(e) = stdin.write_all(input) {
                        println!("Failed to write to stdin for {}: {}",
                                 command, e);
                    }
                }
                None => {
                    println!("Failed to open stdin for {}", command);
                }
            }

            if let Err(e) = child.wait() {
                println!("Failed to run {}: {}", command, e);
            }
        }
        Err(e) => {
            println!("Failed to run command {}: {}", command, e);
        }
    }
}

fn verify_signature<'a>(body: &'a [u8], secret: &'a [u8], signature: &'a [u8])
-> bool
{
    let mut hmac = Hmac::new(Sha1::new(), secret);
    hmac.input(body);
    hmac.result() == MacResult::new(signature)
}

fn hook(request: HttpRequest<State>) -> impl Responder
{
    let config = Arc::clone(&request.state().config);

    let (event, signature) = {
        let headers = request.headers();
        (headers.get("X-GitHub-Event").cloned(),
         headers.get("X-Hub-Signature").cloned())
    };

    if let (Some(event), Some(signature_value)) = (event, signature) {
        let event = String::from_utf8_lossy(event.as_bytes()).into_owned();

        let signature = if signature_value.as_bytes().starts_with(b"sha1=") {
            if let Some(slice) = signature_value.as_bytes().get(5..) {
                from_hex(slice)
            } else {
                from_hex(signature_value.as_bytes())
            }
        } else {
            from_hex(signature_value.as_bytes())
        };

        if let Some(signature) = signature {
            let secret = config.secret.clone();

            if let Some(command) = config.commands.get(&event).cloned() {
                return request.body().and_then(move |bytes| {

                    if verify_signature(&bytes,
                                        &secret.as_bytes(),
                                        &signature) {
                        run_command(&command, &bytes);
                    }
                    Ok(HttpResponse::NoContent())
                })
                .responder();
            }
        }
    }

    AsyncResult::ok(HttpResponse::NoContent()).responder()
}

fn main() {
    let config_path = env::var("GH_HOOK_CONFIG")
        .unwrap_or("./config.toml".to_string());

    let config = Config::load(config_path);
    let bind_addr = config.bind.clone();

    let state = State {
        config: Arc::new(config),
    };

    server::new(move || {
            App::with_state(state.clone())
                .resource("/hook", |r| r.method(Method::POST).f(::hook))
        })
        .bind(bind_addr).unwrap()
        .run();
}
