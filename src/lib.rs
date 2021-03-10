#![recursion_limit = "1024"]

use log::{error, info, warn};
use wasm_bindgen::{prelude::*, JsCast};
use yew::{
    prelude::*,
    services::interval::{IntervalService, IntervalTask},
    utils::{document, window},
    web_sys::{CanvasRenderingContext2d, HtmlImageElement, MouseEvent, Performance, Url},
};

use std::{f64, time::Duration};

mod boid;
mod boids;
mod debug;

use boid::Boid;
use boids::Boids;

const CANVAS_ID: &str = "canvas";
const QR_CODE_ID: &str = "qrcode";
const QR_CODE_LOCATION: &str = "qrcode.png";
const BOID_COLOR: &str = "#bf616a";
const BG_COLOR: &str = "#d8dee9";

pub struct Model {
    boids: Boids,
    last_update: f64,
    last_time_passed: f64,
    link: ComponentLink<Self>,
    settings_panel_shown: bool,
    special_mode: bool,
    _task: Box<IntervalTask>,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Tick,
    TogglePanel,
    ToggleDebugMode,
    ToggleSpecialMode,
    ChangeAlignRadius(f64),
    ChangeCohesionRadius(f64),
    ChangeSeperationRadius(f64),
    ChangeAngstRadius(f64),
    ChangeNrOfBoids(usize),
    ChangeMaxSpeed(f64),
    ChangeMaxSteer(f64),
    ChangeAlignFactor(f64),
    ChangeCohesionFactor(f64),
    ChangeSeperationFactor(f64),
    ChangeAngstFactor(f64),
    MouseMoved(MouseEvent),
    ScatterBoids,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|_| Msg::Tick);
        let handle = IntervalService::spawn(Duration::from_millis(10), callback);
        let (width, height) = get_window_size();
        let settings_panel_shown = false;
        let special_mode = false;
        let mut boids = Boids::new(width, height);
        update_boids_from_url(&mut boids);
        update_url(&boids.to_url_suffix());
        Self {
            boids,
            last_update: performance().now(),
            last_time_passed: 0.0,
            link,
            settings_panel_shown,
            special_mode,
            _task: Box::new(handle),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match &msg {
            Msg::Tick => {
                let now = performance().now();
                self.last_time_passed = now - self.last_update;
                self.last_update = now;

                self.boids.size = get_window_size();
                update(&mut self.boids, self.last_time_passed);
            }
            Msg::MouseMoved(me) => {
                self.boids.predator.x = me.client_x() as f64;
                self.boids.predator.y = me.client_y() as f64;
            }
            Msg::TogglePanel => {
                info!("Toggled panel");
                self.settings_panel_shown = !self.settings_panel_shown;
            }
            Msg::ChangeAlignRadius(radius) => self.boids.align_radius_squared = radius.powf(2.0),
            Msg::ChangeCohesionRadius(radius) => {
                self.boids.cohesion_radius_squared = radius.powf(2.0)
            }
            Msg::ChangeSeperationRadius(radius) => {
                self.boids.seperation_radius_squared = radius.powf(2.0)
            }
            Msg::ChangeAngstRadius(radius) => self.boids.angst_radius_squared = radius.powf(2.0),
            Msg::ChangeMaxSpeed(max_speed) => self.boids.max_speed = *max_speed,
            Msg::ChangeMaxSteer(max_steer) => self.boids.max_steer = *max_steer,
            Msg::ChangeAlignFactor(factor) => self.boids.align_factor = *factor,
            Msg::ChangeCohesionFactor(factor) => self.boids.cohesion_factor = *factor,
            Msg::ChangeSeperationFactor(factor) => self.boids.seperation_factor = *factor,
            Msg::ChangeAngstFactor(factor) => self.boids.angst_factor = *factor,
            Msg::ChangeNrOfBoids(number) => change_number_of_boids(&mut self.boids, *number),
            Msg::ToggleDebugMode => self.boids.debug_mode = !self.boids.debug_mode,
            Msg::ScatterBoids => self.boids.scatter(),
            Msg::ToggleSpecialMode => self.special_mode = !self.special_mode,
        }
        match msg {
            Msg::Tick
            | Msg::TogglePanel
            | Msg::ToggleDebugMode
            | Msg::ToggleSpecialMode
            | Msg::MouseMoved(_)
            | Msg::ScatterBoids => {}
            Msg::ChangeAlignRadius(_)
            | Msg::ChangeCohesionRadius(_)
            | Msg::ChangeSeperationRadius(_)
            | Msg::ChangeAngstRadius(_)
            | Msg::ChangeNrOfBoids(_)
            | Msg::ChangeMaxSpeed(_)
            | Msg::ChangeMaxSteer(_)
            | Msg::ChangeAlignFactor(_)
            | Msg::ChangeCohesionFactor(_)
            | Msg::ChangeSeperationFactor(_)
            | Msg::ChangeAngstFactor(_) => update_url(&self.boids.to_url_suffix()),
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <canvas id=CANVAS_ID onmousemove=self.link.callback(Msg::MouseMoved)>
                </canvas>
                <button id="toggle-panel" onclick=self.link.callback(|_| Msg::TogglePanel)>
                </button>
                { self.display_settings_panel() }
                <img id=QR_CODE_ID src=QR_CODE_LOCATION />
            </>
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        self.render();
    }
}

impl Model {
    fn display_settings_panel(&self) -> Html {
        macro_rules! change {
            ($msg:ident) => {
                self.link.callback(|raw: ChangeData| {
                    if let ChangeData::Value(raw) = raw {
                        Msg::$msg(raw.parse().unwrap())
                    } else {
                        error!("ChangeData sucks...");
                        panic!("ChangeData sucks...");
                    }
                })
            };
        }
        macro_rules! click {
            ($msg:ident) => {
                self.link.callback(|_| Msg::$msg)
            };
        }
        if self.settings_panel_shown {
            html! {
                <div id="settings-panel">
                    <div>
                        <label for="nr-of-boids">{ "Nr of Boids" }</label>
                        <input type="range"
                               id="nr-of-boids"
                               name="nr-of-boids"
                               min="0" max="4000"
                               value={self.boids.boids.len()}
                               onchange={change!(ChangeNrOfBoids)}
                        />
                    </div>
                    <div>
                        <label for="align-radius">{ "Align Radius" }</label>
                        <input type="range"
                               id="align-radius"
                               name="align-radius"
                               min="0" max="400"
                               value={self.boids.align_radius_squared.sqrt()}
                               onchange={change!(ChangeAlignRadius)}
                        />
                    </div>
                    <div>
                        <label for="cohesion-radius">{ "Cohesion Radius" }</label>
                        <input type="range"
                               id="cohesion-radius"
                               name="cohesion-radius"
                               min="0" max="400"
                               value={self.boids.cohesion_radius_squared.sqrt()}
                               onchange={change!(ChangeCohesionRadius)}
                        />
                    </div>
                    <div>
                        <label for="seperation-radius">{ "Seperation Radius" }</label>
                        <input type="range"
                               id="seperation-radius"
                               name="seperation-radius"
                               min="0" max="400"
                               value={self.boids.seperation_radius_squared.sqrt()}
                               onchange={change!(ChangeSeperationRadius)}
                        />
                    </div>
                    <div>
                        <label for="angst-radius">{ "Angst Radius" }</label>
                        <input type="range"
                               id="angst-radius"
                               name="angst-radius"
                               min="0" max="400"
                               value={self.boids.angst_radius_squared.sqrt()}
                               onchange={change!(ChangeAngstRadius)}
                        />
                    </div>
                    <div>
                        <label for="align-factor">{ "Align Factor" }</label>
                        <input type="range"
                               id="align-factor"
                               name="align-factor"
                               min="0" max="10" step="0.1"
                               value={self.boids.align_factor}
                               onchange={change!(ChangeAlignFactor)}
                        />
                    </div>
                    <div>
                        <label for="cohesion-factor">{ "Cohesion Factor" }</label>
                        <input type="range"
                               id="cohesion-factor"
                               name="cohesion-factor"
                               min="0" max="10" step="0.1"
                               value={self.boids.cohesion_factor}
                               onchange={change!(ChangeCohesionFactor)}
                        />
                    </div>
                    <div>
                        <label for="seperation-factor">{ "Seperation Factor" }</label>
                        <input type="range"
                               id="seperation-factor"
                               name="seperation-factor"
                               min="0" max="10" step="0.1"
                               value={self.boids.seperation_factor}
                               onchange={change!(ChangeSeperationFactor)}
                        />
                    </div>
                    <div>
                        <label for="angst-factor">{ "Angst Factor" }</label>
                        <input type="range"
                               id="angst-factor"
                               name="angst-factor"
                               min="0" max="20000"
                               value={self.boids.angst_factor}
                               onchange={change!(ChangeAngstFactor)}
                        />
                    </div>
                    <div>
                        <label for="max-steer">{ "Acceleration Limit" }</label>
                        <input type="range"
                               id="max-steer"
                               name="max-steer"
                               min="0" max="50" step="0.1"
                               value={self.boids.max_steer}
                               onchange={change!(ChangeMaxSteer)}
                        />
                    </div>
                    <div>
                        <label for="max-speed">{ "Speed Limit" }</label>
                        <input type="range"
                               id="max-speed"
                               name="max-speed"
                               min="0" max="500"
                               value={self.boids.max_speed}
                               onchange={change!(ChangeMaxSpeed)}
                        />
                    </div>
                    <div>
                        <button id="toggle-debug"
                                onclick={click!(ToggleDebugMode)}>
                            { "Toggle Debug" }
                        </button>
                    </div>
                    <div>
                        <button id="toggle-special-mode"
                                onclick={click!(ToggleSpecialMode)}>
                            { "Special Mode" }
                        </button>
                    </div>
                    <div>
                        <button id="scatter"
                                onclick={click!(ScatterBoids)}>
                            { "Scatter!" }
                        </button>
                    </div>
                </div>
            }
        } else {
            html! {}
        }
    }
    pub fn render(&self) {
        let boids = &self.boids;
        let canvas = document().get_element_by_id(CANVAS_ID).unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();
        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        let qrcode = get_qrcode();
        // Adjust the size
        canvas.set_width(boids.size.0 as u32);
        canvas.set_height(boids.size.1 as u32);
        // Draw the background
        if self.special_mode {
            ctx.set_fill_style(&JsValue::from_str("black"));
        } else {
            ctx.set_fill_style(&JsValue::from_str(BG_COLOR));
        }
        ctx.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
        // Draw all the boids
        ctx.begin_path();
        ctx.set_fill_style(&JsValue::from_str(BOID_COLOR));
        for boid in &boids.boids {
            boid.render(self, &ctx, &qrcode);
        }
        ctx.fill();
        // Draw debug info if necessary
        if boids.debug_mode {
            debug::render_debug_info(&ctx, self);
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    App::<Model>::new().mount_to_body();
}

fn update_boids_from_url(boids: &mut Boids) {
    let raw_url = document().url().expect("Failed to get URL");
    let url = Url::new(&raw_url).expect("Failed to create url from string");
    let params = url.search_params();
    macro_rules! parse {
        ($key:literal, $callback:expr) => {
            if let Some(val) = params.get($key) {
                if let Ok(val) = val.parse() {
                    $callback(val)
                } else {
                    warn!("Invalid value for key {} in url", stringify!($key));
                }
            }
        };
    }
    parse!("nr-of-boids", |number: usize| change_number_of_boids(
        boids, number
    ));
    parse!("align-radius", |radius: f64| boids.align_radius_squared =
        radius.powf(2.0));
    parse!("cohesion-radius", |radius: f64| boids
        .cohesion_radius_squared =
        radius.powf(2.0));
    parse!("seperation-radius", |radius: f64| boids
        .seperation_radius_squared =
        radius.powf(2.0));
    parse!("angst-radius", |radius: f64| boids.angst_radius_squared =
        radius.powf(2.0));
    parse!("max-speed", |speed: f64| boids.max_speed = speed);
    parse!("max-steer", |steer: f64| boids.max_steer = steer);
    parse!("align-factor", |fac: f64| boids.align_factor = fac);
    parse!("cohesion-factor", |fac: f64| boids.cohesion_factor = fac);
    parse!("seperation-factor", |fac: f64| boids.seperation_factor =
        fac);
    parse!("angst-factor", |fac: f64| boids.angst_factor = fac);
}

fn update_url(url_suffix: &str) {
    window()
        .history()
        .expect("History not found")
        .replace_state_with_url(&JsValue::NULL, "", Some(url_suffix))
        .expect("Replacing state failed");
}

fn performance() -> Performance {
    web_sys::window()
        .expect("Could not get window object")
        .performance()
        .expect("Could not get performance object")
}

fn update(boids: &mut Boids, time_passed: f64) {
    let secs = time_passed / 1000.0;
    // Iterate over all boid indices
    for idx in 0..boids.boids.len() {
        Boid::update(idx, boids, secs);
    }
}

fn change_number_of_boids(boids: &mut Boids, number: usize) {
    while number > boids.boids.len() {
        boids.boids.push(Boid::new(boids.size.0, boids.size.1));
    }
    while number < boids.boids.len() {
        boids.boids.pop();
    }
}

pub fn get_window_size() -> (f64, f64) {
    let window = web_sys::window().expect("Could not get window object");
    let width = window
        .inner_width()
        .expect("Could not get window width")
        .as_f64()
        .expect("Width is not a number");
    let height = window
        .inner_height()
        .expect("Could not get window height")
        .as_f64()
        .expect("Height is not a number");
    (width, height)
}

pub fn get_qrcode() -> HtmlImageElement {
    document()
        .get_element_by_id(QR_CODE_ID)
        .unwrap()
        .dyn_into()
        .unwrap()
}
