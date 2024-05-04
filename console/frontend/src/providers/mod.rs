mod app_config_provider;
mod msg_subscriber_provider;
mod websocket_provider;
pub use app_config_provider::{use_app_config, AppConfigProvider};
pub use msg_subscriber_provider::{use_subscription_manager, MsgSubscriberProvider};
pub use websocket_provider::{use_websocket, WebsocketProvider};
use yew::{function_component, html, Children, Html, Properties};

#[function_component(ConsoleProviders)]
pub fn console_providers(props: &ConsoleProvidersProps) -> Html {
    html! {
        <MsgSubscriberProvider>
            <WebsocketProvider>
                <AppConfigProvider>
                    { props.children.clone() }
                </AppConfigProvider>
            </WebsocketProvider>
        </MsgSubscriberProvider>
    }
}

#[derive(PartialEq, Properties)]
pub struct ConsoleProvidersProps {
    pub children: Children,
}
