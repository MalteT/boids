#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rocket::{http::ContentType, response::content};
use rocket_contrib::serve::StaticFiles;

#[get("/")]
fn index() -> content::Html<&'static str> {
    content::Html(include_str!("../serve/index.html"))
}

#[get("/style.css")]
fn style() -> content::Css<&'static str> {
    content::Css(include_str!("../serve/style.css"))
}

#[get("/script.js")]
fn script() -> content::JavaScript<&'static str> {
    content::JavaScript(include_str!("../serve/script.js"))
}

#[get("/qrcode.png")]
fn qrcode() -> content::Content<&'static [u8]> {
    content::Content(ContentType::PNG, include_bytes!("../serve/qrcode.png"))
}

#[get("/wasm.js")]
fn wasm() -> content::JavaScript<&'static str> {
    content::JavaScript(include_str!("../../frontend/static/wasm.js"))
}

#[get("/wasm_bg.wasm")]
fn wasm_bg() -> content::Content<&'static [u8]> {
    content::Content(
        ContentType::WASM,
        include_bytes!("../../frontend/static/wasm_bg.wasm"),
    )
}

fn main() {
    rocket::ignite()
        .mount("/", StaticFiles::from("../frontend/static"))
        .mount("/", routes![index, style, script, qrcode, wasm])
        .launch();
}
