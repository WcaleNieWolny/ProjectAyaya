use std::sync::Arc;

use once_cell::sync::OnceCell;
use serenity::{prelude::GatewayIntents, Client};
use songbird::input::Restartable;
use songbird::tracks::TrackHandle;
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
    pub fn connect_and_play(&self, audio_path: String, use_map_server: bool) -> anyhow::Result<TrackHandle>{
        let songbird = self.songbird.clone();
        let options = self.options.clone();

        let handler_lock = songbird.get(self.options.guild_id);
        if handler_lock.is_some() {
            return Err(anyhow!("Discord client connected to a channel"));
        }

        let join_handle: anyhow::Result<TrackHandle> = TOKIO_RUNTIME.handle().clone().block_on(async move {
            let (handler_lock, join_result) = songbird.join(options.guild_id, options.channel_id).await;
            join_result?;

            let mut handler = handler_lock.lock().await;
           
            let source = Restartable::ffmpeg(audio_path, false).await?;
            
            let track = handler.play_source(source.into());
            if use_map_server {
                track.pause()?;
            }
            Ok(track)
        });
        Ok(join_handle?)
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

    pub fn is_used() -> anyhow::Result<bool> {
        let client = DISCORD_CLIENT.get();

        let client = match client {
            Some(val) => val,
            None => return Err(anyhow!("Discord client not initialized"))
        };

        return Ok(client.songbird.get(client.options.guild_id).is_some());
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
    inner: Box<dyn VideoPlayer>,
    track_handle: TrackHandle,
    use_map_server: bool
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
        match msg {
            NativeCommunication::StartRendering { .. } => {
                self.track_handle.play()?;
                if !self.use_map_server {
                    return Ok(());
                }
            }
            NativeCommunication::StopRendering { .. } => {
                self.track_handle.pause()?;
                if !self.use_map_server {
                    return Ok(());
                }
            }
            _ => {}
       };

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
    pub fn create_with_discord(filename: String, player: Box<dyn VideoPlayer>, use_map_server: bool) -> anyhow::Result<Self>{
        let discord_client = match DISCORD_CLIENT.get(){
            Some(val) => val,
            None => return Err(anyhow!("Discord client not initialized"))
        };

        let track_handle = discord_client.connect_and_play(filename, use_map_server)?;
        
        Ok(Self {
            inner: player,
            track_handle,
            use_map_server
        })
    }
}
