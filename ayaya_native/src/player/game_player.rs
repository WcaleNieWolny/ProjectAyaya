use crate::map_server::ServerOptions;

use super::player_context::VideoPlayer;

struct GamePlayer {
    width: i32,
    height: i32,
    fps: i32,
}

impl VideoPlayer for GamePlayer {
    fn create(
        file_name: String,
        server_options: ServerOptions,
    ) -> anyhow::Result<super::player_context::PlayerContext> {
        todo!()
    }

    fn load_frame(&mut self) -> anyhow::Result<Vec<i8>> {
        todo!()
    }

    fn video_data(&self) -> anyhow::Result<super::player_context::VideoData> {
        todo!()
    }

    fn handle_jvm_msg(
        &self,
        msg: super::player_context::NativeCommunication,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn destroy(self: Box<Self>) -> anyhow::Result<()> {
        todo!()
    }
}
