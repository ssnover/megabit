use std::cell::RefCell;
use yew::prelude::*;

mod debug_led;
use debug_led::DebugLed;
mod matrix;
use matrix::Canvas;
mod recording_buttons;
use recording_buttons::{StartRecording, StopRecording};
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

    let is_rgb_display = use_state(|| false);
    let is_rgb_display_setter = is_rgb_display.setter();

    let matrix_buffer = use_state(|| {
        RefCell::new(vec![
            0u8;
            (matrix::simple_display::COLUMNS * matrix::simple_display::ROWS)
                as usize
        ])
    });
    let rgb_matrix_buffer = use_state(|| {
        RefCell::new(vec![
            0u16;
            (matrix::rgb_display::COLUMNS * matrix::rgb_display::ROWS)
                as usize
        ])
    });
    let working_matrix_buffer = use_state(|| {
        RefCell::new(vec![
            0u16;
            (matrix::rgb_display::COLUMNS * matrix::rgb_display::ROWS)
                as usize
        ])
    });

    let renderer_cb = {
        let matrix_buffer = matrix_buffer.clone();
        let rgb_matrix_buffer = rgb_matrix_buffer.clone();

        if *is_rgb_display {
            Callback::from(move |canvas| {
                log::debug!("Draw rgb display");
                matrix::rgb_display::draw(canvas, &*rgb_matrix_buffer);
            })
        } else {
            Callback::from(move |canvas| {
                log::debug!("Draw simple display");
                matrix::simple_display::draw(canvas, &*matrix_buffer);
            })
        }
    };

    let update_row_cb = {
        let matrix_buffer = matrix_buffer.clone();
        Callback::from(move |(row_number, data)| {
            matrix::simple_display::update_row(row_number, data, &matrix_buffer);
        })
    };
    let update_row_rgb_cb = {
        let matrix_buffer = working_matrix_buffer.clone();
        Callback::from(move |(row_number, data)| {
            log::debug!("Updating rgb callback");
            matrix::rgb_display::update_row(row_number, data, &matrix_buffer);
        })
    };
    let commit_render_cb = {
        let rgb_matrix = rgb_matrix_buffer.clone();
        let working_matrix = working_matrix_buffer.clone();
        Callback::from(move |()| {
            matrix::rgb_display::update_whole(&rgb_matrix, &working_matrix);
        })
    };

    html! {
        <WebsocketProvider set_led_state={led_state_setter} set_rgb_state={rgb_state_setter} {update_row_cb} {update_row_rgb_cb} {is_rgb_display_setter} {commit_render_cb}>
            <h1>{ "Megabit Coproc Simulator" }</h1>
            <div style="display:flex;">
                <UserButton/>
                <StartRecording/>
                <StopRecording/>
            </div>
            <div style="display:flex">
                <DebugLed {led_state} />
                <RgbLed {rgb_state} />
            </div>
            <Canvas renderer={renderer_cb} {matrix_buffer} {rgb_matrix_buffer} />
        </WebsocketProvider>
    }
}
