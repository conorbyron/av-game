//use js_sys::Function;
use specs::{
    Builder, Component, DispatcherBuilder, ReadStorage, System, VecStorage, World, WriteStorage,
};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{console, KeyboardEvent, MessageEvent, MouseEvent, WebSocket};
//use serde::{Deserialize, Serialize};
//use serde_derive;
//use serde_json::Result as JsonResult; // Is this necessary, or does it overlap with the serde feature of wasm-bindgen?

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct Game {
    specs_world: World,
}

pub struct UpdateDeltaTime {
    pub dt: f64,
}

impl Game {
    fn update(&mut self) {
        self.specs_world.maintain();
    }

    fn press(&mut self) {
        /*
        self.specs_world.write_resource::<InputEvents>().events
            .push_back(InputEvent::PressEvent(*args));

        // FIXME: Move to edit.rs
        if let &Button::Keyboard(Key::R) = args {
            saveload::ResetWorld.run_now(&mut self.specs_world.res);
            self.specs_world.maintain();
        }
        */
    }

    fn release(&mut self) {
        /*
        self.specs_world.write_resource::<InputEvents>().events
            .push_back(InputEvent::ReleaseEvent(*args));
        */
    }

    fn mouse_cursor(&mut self, x: f64, y: f64) {
        /*
        self.specs_world.write_resource::<InputEvents>().events
            .push_back(InputEvent::MotionEvent(x, y));
        */
    }
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn body() -> web_sys::HtmlElement {
    document().body().expect("document should have a body")
}

fn setup_mouse_and_keyboard_events() -> Result<(), JsValue> {
    let window = window();

    let keydown_callback = Closure::wrap(Box::new(|e: KeyboardEvent| {
        console::log_1(&(format!("keyDown: {:?}", e.key()).into()));
    }) as Box<dyn FnMut(KeyboardEvent)>);
    window
        .add_event_listener_with_callback(&"keydown", keydown_callback.as_ref().unchecked_ref())?;
    keydown_callback.forget();

    let keyup_callback = Closure::wrap(Box::new(|e: KeyboardEvent| {
        console::log_1(&(format!("keyUp: {:?}", e.key()).into()));
    }) as Box<dyn FnMut(KeyboardEvent)>);
    window.add_event_listener_with_callback(&"keyup", keyup_callback.as_ref().unchecked_ref())?;
    keyup_callback.forget();

    let mousemove_callback = Closure::wrap(Box::new(|e: MouseEvent| {
        console::log_1(&(format!("mouseMove: x: {:?} y: {:?}", e.screen_x(), e.screen_y()).into()));
    }) as Box<dyn FnMut(MouseEvent)>);
    window.add_event_listener_with_callback(
        &"mousemove",
        mousemove_callback.as_ref().unchecked_ref(),
    )?;
    mousemove_callback.forget();

    let click_callback = Closure::wrap(Box::new(|e: MouseEvent| {
        console::log_1(&(format!("click: x: {:?} y: {:?}", e.screen_x(), e.screen_y()).into()));
    }) as Box<dyn FnMut(MouseEvent)>);
    window.add_event_listener_with_callback(&"click", click_callback.as_ref().unchecked_ref())?;
    click_callback.forget();

    Ok(())
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    set_panic_hook();

    // websocket creation/setup
    let ws_callback = Closure::wrap(Box::new(|e: MessageEvent| {
        console::log_1(&(format!("received: {:?}", e.data()).into()));
    }) as Box<dyn Fn(MessageEvent)>);
    let ws = WebSocket::new(&"ws://localhost:3000").expect("Failed to connect!");
    ws.set_onmessage(Some(ws_callback.as_ref().unchecked_ref()));
    ws_callback.forget();

    setup_mouse_and_keyboard_events()?;

    // request_animation_frame setup
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    // Could this method be used for Websockets and their on_message closures?
    // On closing the connection, just do a "let _ = f.borrow_mut().take();"?

    let mut i = 0;
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        i += 1;
        let text = format!("requestAnimationFrame has been called {} times.", i);
        if ws.ready_state() == 1 {
            ws.send_with_str(&text).expect("Failed to send!");
        }
        body().set_text_content(Some(&text));
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
