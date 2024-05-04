use yew::{
    function_component, hook, html, use_context, Children, ContextProvider, Html, Properties,
};

const DEFAULT_WIDTH: u32 = 64;
const DEFAULT_HEIGHT: u32 = 32;

#[hook]
pub fn use_app_config() -> AppConfig {
    use_context().unwrap()
}

#[derive(Clone, PartialEq)]
pub struct AppConfig {
    display_width: u32,
    display_height: u32,
}

impl AppConfig {
    pub fn width(&self) -> u32 {
        self.display_width
    }

    pub fn height(&self) -> u32 {
        self.display_height
    }
}

#[function_component]
pub fn AppConfigProvider(props: &AppConfigProviderProps) -> Html {
    let context = AppConfig {
        display_width: DEFAULT_WIDTH,
        display_height: DEFAULT_HEIGHT,
    };

    html! {
        <ContextProvider<AppConfig> {context}>{props.children.clone()}</ContextProvider<AppConfig>>
    }
}

#[derive(Properties, PartialEq)]
pub struct AppConfigProviderProps {
    pub children: Children,
}
