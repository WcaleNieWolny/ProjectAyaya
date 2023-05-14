#[cfg(feature = "discord")]
pub mod discord_audio;

pub mod game_player;
pub(crate) mod player_context;

#[cfg(feature = "ffmpeg")]
pub mod multi_video_player;
#[cfg(feature = "ffmpeg")]
pub mod single_video_player;
#[cfg(feature = "ffmpeg")]
pub mod x11_player;

#[cfg(all(target_os = "linux", feature = "ffmpeg"))]
pub mod blazing_linux_player;

#[cfg(all(feature = "external_player", feature = "ffmpeg"))]
//#[cfg(feature = "ffmpeg")]
pub mod external_player;
