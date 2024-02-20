mod frontend;

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    yew::Renderer::<frontend::App>::new().render();
}
