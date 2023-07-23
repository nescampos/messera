use linera_sdk::views::{CustomMapView, LogView, ViewStorageContext};
use linera_views::views::{GraphQLView, RootView};
use messera::{Key, MyMessage};

/// The application state.
#[derive(RootView, GraphQLView)]
#[view(context = "ViewStorageContext")]
pub struct Messera {
    /// Our posts.
    pub my_messages: LogView<MyMessage>,
    /// Posts we received from authors we subscribed to.
    pub received_messages: CustomMapView<Key, String>,
}