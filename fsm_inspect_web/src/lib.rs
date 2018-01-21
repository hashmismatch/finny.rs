#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate websocket;
extern crate rocket;
extern crate handlebars;
extern crate fsm;

extern crate serde;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

pub mod data;
pub mod server_state;
pub mod server_web;
pub mod server_ws;
