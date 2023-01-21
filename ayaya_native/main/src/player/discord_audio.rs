use std::sync::Arc;

use once_cell::sync::OnceCell;
use serenity::{prelude::GatewayIntents, Client};
use songbird::input::Restartable;
use songbird::{Songbird, SerenityInit};

use crate::{TOKIO_RUNTIME, map_server::ServerOptions};
use crate::anyhow;

use super::player_context::{VideoPlayer, NativeCommunication, VideoData};

static DISCORD_CLIENT: OnceCell<DiscordClient> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct DiscordOptions {
    pub use_discord: bool,
    pub discord_token: String,
    pub guild_id: u64,
    pub channel_id: u64,
}

pub struct DiscordClient {
    options: DiscordOptions,
    songbird: Arc<Songbird>
}

impl DiscordClient {
    pub fn connect_and_play(&self, audio_path: String) -> anyhow::Result<()>{
        let songbird = self.songbird.clone();
        let options = self.options.clone();

        let handler_lock = songbird.get(self.options.guild_id);
        if handler_lock.is_some() {
            return Err(anyhow!("Discord client connected to a channel"));
        }

        TOKIO_RUNTIME.handle().clone().spawn(async move {
            let (handler_lock, join_result) = songbird.join(options.guild_id, options.channel_id).await;
            match join_result {
                Ok(_) => {},
                Err(err) => {
                    println!("[ProjectAyaya] Unable to connect to discord channel! Error: {:?}", err);
                }
            }

            let mut handler = handler_lock.lock().await;
           
            let source = Restartable::ffmpeg(audio_path, false).await;
            if let Err(err) = source {
                println!("Unable to create ffmpeg discord source! Err: {:?}", err);
                return;
            }
            
            handler.play_source(source.unwrap().into());
        });

        Ok(())
    }

    pub fn leave_channel(&self) -> anyhow::Result<()> {
        let songbird = self.songbird.clone();
        let options = self.options.clone();

        TOKIO_RUNTIME.handle().clone().spawn(async move {
            let handler_lock = songbird.get(options.guild_id);

            if handler_lock.is_none() {
                println!("[ProjectAyaya] Discord bot not connected! Cannot leave channel!");
                return;
            }

            if let Err(err) = songbird.remove(options.guild_id).await {
                println!("[ProjectAyaya] Cannot leave discord audio channel! Err: {:?}", err);
                return;
            }
            
        });

        Ok(())
    }
}

//We assume that caller checked if use_discord == true
pub fn init(options: &DiscordOptions) -> anyhow::Result<()>{
    let handle = TOKIO_RUNTIME.handle().clone();
    let options_clone = options.clone();

    handle.spawn(async move {
        println!("[ProjectAyaya] initializing discord bot!");
        let intents = GatewayIntents::non_privileged()
            | GatewayIntents::MESSAGE_CONTENT;

        let songbird = Songbird::serenity();

        let mut client = Client::builder(&options_clone.discord_token, intents)
            .register_songbird_with(songbird.clone())
            .await
            .expect("Err creating client");

        let discord_static_client = DiscordClient {
            songbird,
            options: options_clone.clone()
        };

        if let Err(_) = DISCORD_CLIENT.set(discord_static_client){
            println!("Unable to set static discord client!");
            return;
        }

        let _ = client.start().await.map_err(|why| println!("Discord client ended: {:?}", why));
    });

    Ok(())   
}

pub struct DiscordPlayer {
    inner: Box<dyn VideoPlayer>
}

impl VideoPlayer for DiscordPlayer {
    fn create(_file_name: String, _server_options: ServerOptions) -> anyhow::Result<Self>
    where
        Self: Sized {
        return Err(anyhow!("Please use the other init function!"))
    }

    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>> {
        self.inner.load_frame()
    }

    fn video_data(&self) -> anyhow::Result<VideoData> {
        self.inner.video_data()
    }

    fn handle_jvm_msg(&self, msg: NativeCommunication) -> anyhow::Result<()> {
        self.inner.handle_jvm_msg(msg)
    }

    fn destroy(&self) -> anyhow::Result<()> {
        if let Some(discord_client) = DISCORD_CLIENT.get(){
            discord_client.leave_channel()?;
        };

        self.inner.destroy()
    }
}

impl DiscordPlayer {
    pub fn create_with_discord(filename: String, player: Box<dyn VideoPlayer>) -> anyhow::Result<Self>{
        let discord_client = match DISCORD_CLIENT.get(){
            Some(val) => val,
            None => return Err(anyhow!("Discord client not initialized"))
        };

        discord_client.connect_and_play(filename)?;
        
        Ok(Self {
            inner: player
        })
    }
}
