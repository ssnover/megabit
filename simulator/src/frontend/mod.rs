use std::cell::RefCell;
use yew::prelude::*;

mod debug_led;
use debug_led::DebugLed;
mod matrix;
use matrix::Canvas;
mod rgb_led;
use rgb_led::RgbLed;
mod user_button;
use user_button::UserButton;
mod websocket_provider;
use websocket_provider::WebsocketProvider;

#[function_component(App)]
pub fn app() -> Html {
    let led_state = use_state(|| false);
    let led_state_setter = led_state.setter();

    let rgb_state = use_state(|| (0, 0, 0));
    let rgb_state_setter = rgb_state.setter();

    let update_counter = use_state(|| 0);
    let update_counter_setter = update_counter.setter();

    let matrix_buffer =
        use_state(|| RefCell::new([0u8; (matrix::COLUMNS * matrix::ROWS) as usize]));

    let renderer_cb = {
        let matrix_buffer = matrix_buffer.clone();
        Callback::from(move |canvas| {
            matrix::draw(canvas, &matrix_buffer);
        })
    };

    let update_row_cb = {
        let matrix_buffer = matrix_buffer.clone();
        let update_counter = update_counter.clone();
        Callback::from(move |(row_number, data)| {
            update_counter_setter.set(*update_counter + 1);
            matrix::update_row(row_number, data, &matrix_buffer);
        })
    };

    html! {
        <WebsocketProvider set_led_state={led_state_setter} set_rgb_state={rgb_state_setter} {update_row_cb}>
            <h1>{ "Megabit Coproc Simulator" }</h1>
            <UserButton/>
            <DebugLed {led_state} />
            <RgbLed {rgb_state} />
            <Canvas renderer={renderer_cb} counter={update_counter}/>
        </WebsocketProvider>
    }
}
