use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
use std::sync::Mutex;

pub struct DiscordRPC {
    client: Mutex<Option<DiscordIpcClient>>,
    connected: Mutex<bool>,
}

impl DiscordRPC {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
            connected: Mutex::new(false),
        }
    }

    pub fn connect(&self, client_id: &str) {
        if *self.connected.lock().unwrap() {
            return;
        }
        match DiscordIpcClient::new(client_id) {
            Ok(mut client) => {
                if client.connect().is_ok() {
                    *self.client.lock().unwrap() = Some(client);
                    *self.connected.lock().unwrap() = true;
                }
            }
            Err(_) => {}
        }
    }

    pub fn disconnect(&self) {
        let mut guard = self.client.lock().unwrap();
        if let Some(client) = guard.as_mut() {
            let _ = client.close();
        }
        *guard = None;
        *self.connected.lock().unwrap() = false;
    }

    pub fn update(&self, state_str: &str, details: &str, start_time: Option<i64>, art_url: Option<&str>) {
        if !*self.connected.lock().unwrap() {
            return;
        }
        let mut guard = self.client.lock().unwrap();
        if let Some(client) = guard.as_mut() {
            let mut act = activity::Activity::new()
                .state(state_str)
                .details(details);
            if let Some(ts) = start_time {
                act = act.timestamps(activity::Timestamps::new().start(ts));
            }
            if let Some(url) = art_url {
                act = act.assets(
                    activity::Assets::new()
                        .large_text("Offline Music Player")
                        .large_image(url)
                );
            } else {
                act = act.assets(
                    activity::Assets::new()
                        .large_text("Offline Music Player")
                        .large_image("icon")
                );
            }
            let _ = client.set_activity(act);
        }
    }
}
