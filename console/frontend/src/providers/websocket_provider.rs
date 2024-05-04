use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use gloo::{
    net::websocket::{futures::WebSocket, Message},
    utils::window,
};
use gloo_net::websocket::WebSocketError;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen_futures::spawn_local;
use yew::{
    function_component, hook, html, use_context, use_effect, use_state, Callback, Children,
    ContextProvider, Html, Properties, UseStateHandle,
};

use super::{msg_subscriber_provider::SubscriptionManager, use_subscription_manager};

#[hook]
pub fn use_websocket() -> WebsocketHandle {
    use_context().unwrap()
}

#[derive(Clone, PartialEq)]
pub struct WebsocketHandle {
    send_message: Callback<Vec<u8>>,
}

impl WebsocketHandle {
    pub fn send_message(&self, msg: Vec<u8>) {
        self.send_message.emit(msg);
    }
}

#[function_component]
pub fn WebsocketProvider(props: &WebsocketProviderProps) -> Html {
    let subscription_manager = use_subscription_manager();

    let connection = use_state(|| {
        let endpoint = if let (Ok(hostname), Ok(port)) =
            (window().location().hostname(), window().location().port())
        {
            format!("{hostname}:{port}")
        } else {
            log::error!("Failed to retrieve the hostname");
            String::new()
        };

        let ws = WebSocket::open(&format!("ws://{endpoint}/ws")).unwrap();
        let (writer, reader) = ws.split();

        (Rc::new(RefCell::new(writer)), Rc::new(RefCell::new(reader)))
    });

    use_effect({
        let connection = WebsocketConnection {
            inner: connection.clone(),
        };
        move || {
            spawn_local(async move {
                start_ws_context(connection, subscription_manager).await;
            });
        }
    });

    let send_message = {
        let connection = WebsocketConnection {
            inner: connection.clone(),
        };
        move |msg: Vec<u8>| {
            let connection = connection.clone();
            spawn_local(async move { connection.send(msg).await });
        }
    }
    .into();
    let context = WebsocketHandle { send_message };

    html! {
        <ContextProvider<WebsocketHandle> {context}>{props.children.clone()}</ContextProvider<WebsocketHandle>>
    }
}

#[derive(Properties, PartialEq)]
pub struct WebsocketProviderProps {
    pub children: Children,
}

#[derive(Clone)]
struct WebsocketConnection {
    inner: UseStateHandle<(
        Rc<RefCell<SplitSink<WebSocket, Message>>>,
        Rc<RefCell<SplitStream<WebSocket>>>,
    )>,
}

impl WebsocketConnection {
    pub async fn send(&self, msg: Vec<u8>) {
        if let Err(err) = self
            .inner
            .0
            .try_borrow_mut()
            .unwrap()
            .send(Message::Bytes(msg))
            .await
        {
            log::error!("Failed to send message: {err}");
        }
    }

    pub async fn read(&self) -> Option<Result<Message, WebSocketError>> {
        let mut reader = self.inner.1.try_borrow_mut().unwrap();
        reader.next().await
    }
}

async fn start_ws_context(
    connection: WebsocketConnection,
    subscription_manager: SubscriptionManager,
) {
    loop {
        if let Some(msg) = connection.read().await {
            match msg {
                Ok(Message::Bytes(msg)) => {
                    if let Ok(msg) = serde_json::from_slice(&msg[..]) {
                        subscription_manager.handle_message(msg);
                    }
                }
                Ok(_) => {
                    log::debug!("Got a non-bytes WebSocket message");
                }
                Err(err) => {
                    log::error!("Got error on the websocket: {err}");
                }
            }
        }
    }
}
