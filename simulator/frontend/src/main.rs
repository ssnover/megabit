mod frontend;

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
    yew::Renderer::<frontend::App>::new().render();
}
