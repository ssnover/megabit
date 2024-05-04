use megabit_runner_msgs::ConsoleMessage;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use yew::{
    function_component, hook, html, use_context, Callback, Children, ContextProvider, Html,
    Properties,
};

#[hook]
pub fn use_subscription_manager() -> SubscriptionManager {
    use_context().unwrap()
}

type SubscriptionKey = &'static str;
type MsgCallback = Callback<ConsoleMessage>;
type SubscriptionRegistry = HashMap<SubscriptionKey, Vec<(String, MsgCallback)>>;

#[derive(Clone, PartialEq)]
pub struct SubscriptionManager {
    registry: Rc<RefCell<SubscriptionRegistry>>,
}

impl SubscriptionManager {
    pub fn subscribe(&self, client: &str, message_kind: &'static str, callback: MsgCallback) {
        log::debug!("Adding subscription for kind {message_kind} for client {client}");
        let mut registry = self.registry.borrow_mut();
        match registry.get_mut(message_kind) {
            Some(subscriptions) => {
                if let Some((_, existing_cb)) = subscriptions
                    .iter_mut()
                    .find(|(sub_client, _)| client == sub_client.as_str())
                {
                    *existing_cb = callback;
                } else {
                    subscriptions.push((client.into(), callback));
                }
            }
            None => {
                let _ = registry.insert(message_kind, vec![(client.to_string(), callback)]);
            }
        }
    }

    pub fn handle_message(&self, msg: ConsoleMessage) -> bool {
        log::debug!("Handling message: {}", msg.as_ref());
        let registry = self.registry.borrow();
        if let Some(subscriptions) = registry.get(msg.as_ref()) {
            for (_, cb) in subscriptions {
                cb.emit(msg.clone());
            }
            true
        } else {
            false
        }
    }
}

#[function_component]
pub fn MsgSubscriberProvider(props: &Props) -> Html {
    let context = SubscriptionManager {
        registry: Rc::new(RefCell::new(HashMap::new())),
    };

    html! {
        <ContextProvider<SubscriptionManager> {context}>{props.children.clone()}</ContextProvider<SubscriptionManager>>
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub children: Children,
}
