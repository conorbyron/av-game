//use js_sys::Function;
use specs::prelude::World;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{console, KeyboardEvent, MessageEvent, MouseEvent, WebSocket};
//use serde::{Deserialize, Serialize};
//use serde_derive;
//use serde_json::Result as JsonResult; // Is this necessary, or does it overlap with the serde feature of wasm-bindgen?

mod input;
use input::{InputEvent, InputEvents};

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

thread_local! { static GAME: Rc<RefCell<Option<Game>>> = Rc::new(RefCell::new(None)); }

#[derive(Eq, PartialEq, Hash)]
pub enum Key {
    Up,
    Down,
    Right,
    Left,
    Space,
}

impl Key {
    pub fn from(key_value: &str) -> Option<Key> {
        match key_value {
            "ArrowUp" => Some(Key::Up),
            "ArrowDown" => Some(Key::Down),
            "ArrowRight" => Some(Key::Right),
            "ArrowLeft" => Some(Key::Left),
            " " => Some(Key::Space),
            _ => None,
        }
    }
}

struct Game {
    world: World,
}

impl Game {
    pub fn update(&mut self) {}
    pub fn keydown(&mut self, e: KeyboardEvent) {
        console::log_1(&format!("keydown: {}", e.key()).into());
    }
    pub fn keyup(&mut self, e: KeyboardEvent) {
        console::log_1(&format!("keyup: {}", e.key()).into());
    }
    pub fn click(&mut self, e: MouseEvent) {
        console::log_1(&format!("click: {}, {}", e.screen_x(), e.screen_y()).into());
    }
    pub fn mousemove(&mut self, e: MouseEvent) {
        console::log_1(&format!("mousemove: {}, {}", e.movement_x(), e.movement_y()).into());
    }
    pub fn websocket_message(&mut self, e: MessageEvent) {
        let message: String = e.data().into_serde().unwrap();
        console::log_1(&format!("message: {}", message).into());
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
        GAME.with(|c| c.clone())
            .borrow_mut()
            .as_mut()
            .unwrap()
            .keydown(e);
    }) as Box<dyn FnMut(KeyboardEvent)>);
    window
        .add_event_listener_with_callback(&"keydown", keydown_callback.as_ref().unchecked_ref())?;
    keydown_callback.forget();

    let keyup_callback = Closure::wrap(Box::new(|e: KeyboardEvent| {
        GAME.with(|c| c.clone())
            .borrow_mut()
            .as_mut()
            .unwrap()
            .keyup(e);
    }) as Box<dyn FnMut(KeyboardEvent)>);
    window.add_event_listener_with_callback(&"keyup", keyup_callback.as_ref().unchecked_ref())?;
    keyup_callback.forget();

    let mousemove_callback = Closure::wrap(Box::new(|e: MouseEvent| {
        GAME.with(|c| c.clone())
            .borrow_mut()
            .as_mut()
            .unwrap()
            .mousemove(e);
    }) as Box<dyn FnMut(MouseEvent)>);
    window.add_event_listener_with_callback(
        &"mousemove",
        mousemove_callback.as_ref().unchecked_ref(),
    )?;
    mousemove_callback.forget();

    let click_callback = Closure::wrap(Box::new(|e: MouseEvent| {
        GAME.with(|c| c.clone())
            .borrow_mut()
            .as_mut()
            .unwrap()
            .click(e);
    }) as Box<dyn FnMut(MouseEvent)>);
    window.add_event_listener_with_callback(&"click", click_callback.as_ref().unchecked_ref())?;
    click_callback.forget();

    Ok(())
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    set_panic_hook();

    let mut world = World::new();
    world.add_resource(input::InputEvents::new());
    world.add_resource(input::InputState::new());

    let game = Game{ world };

    *(GAME.with(|c| c.clone())).borrow_mut() = Some(game);

    // websocket creation/setup
    let ws_callback = Closure::wrap(Box::new(|e: MessageEvent| {
        GAME.with(|c| c.clone())
            .borrow_mut()
            .as_mut()
            .unwrap()
            .websocket_message(e);
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
        GAME.with(|c| c.clone())
            .borrow_mut()
            .as_mut()
            .unwrap()
            .update();

        // ~~~ this code will be deleted ~~~
        i += 1;
        if i > 60 {
            i = 1;
        }
        let text = format!("requestAnimationFrame has been called {} times.", i);
        if ws.ready_state() == 1 {
            ws.send_with_str(&text).expect("Failed to send!");
        }
        body().set_text_content(Some(&text));
        // ~~~
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