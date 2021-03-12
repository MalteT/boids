use log::{error, info};
use wasm_bindgen::{prelude::*, JsCast};
use yew::{
    prelude::*,
    services::interval::{IntervalService, IntervalTask},
    utils::document,
    web_sys::{CanvasRenderingContext2d, MouseEvent},
};

use std::{f64, time::Duration};

use crate::{
    boids::Boids, debug, utils as util, BG_COLOR, BOID_COLOR, CANVAS_ID, QR_CODE_ID,
    QR_CODE_LOCATION,
};

pub struct Model {
    pub boids: Boids,
    pub last_update: f64,
    pub last_time_passed: f64,
    pub link: ComponentLink<Self>,
    pub settings_panel_shown: bool,
    pub special_mode: bool,
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
        let (width, height) = util::get_window_size();
        let settings_panel_shown = false;
        let special_mode = false;
        let mut boids = Boids::new(width, height);
        util::update_boids_from_url(&mut boids);
        util::update_url(&boids.to_url_suffix());
        Self {
            boids,
            last_update: util::performance().now(),
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
                let now = util::performance().now();
                self.last_time_passed = now - self.last_update;
                self.last_update = now;

                self.boids.size = util::get_window_size();
                util::update(&mut self.boids, self.last_time_passed);
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
            Msg::ChangeNrOfBoids(number) => util::change_number_of_boids(&mut self.boids, *number),
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
            | Msg::ChangeAngstFactor(_) => util::update_url(&self.boids.to_url_suffix()),
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
        let qrcode = util::get_qrcode();
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
