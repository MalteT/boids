#![recursion_limit = "1024"]

use log::{error, info};
use wasm_bindgen::{prelude::*, JsCast};
use yew::{
    prelude::*,
    services::interval::{IntervalService, IntervalTask},
    utils::document,
    web_sys::{CanvasRenderingContext2d, HtmlImageElement, MouseEvent, Performance},
};

use std::{f64, time::Duration};

mod boid;
mod boids;
mod debug;

use boid::Boid;
use boids::Boids;

const CANVAS_ID: &str = "canvas";
const BOID_RADIUS: f64 = 5.0;
const QR_CODE_ID: &str = "qrcode";
const QR_CODE_SIZE: f64 = 30.0;
const QR_CODE_LOCATION: &str = "qrcode.png";
const BOID_COLOR: &str = "white";
const BG_COLOR: &str = "black";

pub struct Model {
    boids: Boids,
    last_update: f64,
    last_time_passed: f64,
    link: ComponentLink<Self>,
    settings_panel_shown: bool,
    display_as_qr: bool,
    _task: Box<IntervalTask>,
}

pub enum Msg {
    Tick,
    TogglePanel,
    ToggleDebugMode,
    ToggleQrDisplay,
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
        let display_as_qr = false;
        Self {
            boids: Boids::new(width, height),
            last_update: performance().now(),
            last_time_passed: 0.0,
            link,
            settings_panel_shown,
            display_as_qr,
            _task: Box::new(handle),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
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
            Msg::ChangeMaxSpeed(max_speed) => self.boids.max_speed = max_speed,
            Msg::ChangeMaxSteer(max_steer) => self.boids.max_steer = max_steer,
            Msg::ChangeAlignFactor(factor) => self.boids.align_factor = factor,
            Msg::ChangeCohesionFactor(factor) => self.boids.cohesion_factor = factor,
            Msg::ChangeSeperationFactor(factor) => self.boids.seperation_factor = factor,
            Msg::ChangeAngstFactor(factor) => self.boids.angst_factor = factor,
            Msg::ChangeNrOfBoids(number) => {
                while number > self.boids.boids.len() {
                    self.boids
                        .boids
                        .push(Boid::new(self.boids.size.0, self.boids.size.1));
                }
                while number < self.boids.boids.len() {
                    self.boids.boids.pop();
                }
            }
            Msg::ToggleDebugMode => self.boids.debug_mode = !self.boids.debug_mode,
            Msg::ScatterBoids => self.boids.scatter(),
            Msg::ToggleQrDisplay => self.display_as_qr = !self.display_as_qr,
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
                        <button id="toggle-qr"
                                onclick={click!(ToggleQrDisplay)}>
                            { "Toggle QR" }
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
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        let qrcode = get_qrcode();
        // Adjust the size
        canvas.set_width(boids.size.0 as u32);
        canvas.set_height(boids.size.1 as u32);

        context.set_fill_style(&JsValue::from_str(BG_COLOR));
        context.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

        context.begin_path();
        context.set_fill_style(&JsValue::from_str(BOID_COLOR));
        for boid in &boids.boids {
            let (xx, yy) = (boid.pos.x, boid.pos.y);
            if self.display_as_qr {
                context
                    .draw_image_with_html_image_element_and_dw_and_dh(
                        &qrcode,
                        xx - QR_CODE_SIZE / 2.0,
                        yy - QR_CODE_SIZE / 2.0,
                        QR_CODE_SIZE,
                        QR_CODE_SIZE,
                    )
                    .unwrap();
            } else {
                context.move_to(xx + BOID_RADIUS, yy);
                context
                    .arc(xx, yy, BOID_RADIUS, 0.0, 2.0 * f64::consts::PI)
                    .expect("Failed to draw boid");
            }
        }
        context.fill();

        if boids.debug_mode {
            debug::render_debug_info(&context, self);
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    App::<Model>::new().mount_to_body();
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
