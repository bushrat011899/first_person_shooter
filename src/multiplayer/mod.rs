use bevy::{prelude::*, tasks::IoTaskPool};
use bevy_ggrs::Session;
use ggrs::{Config, PlayerHandle, SessionBuilder};
use matchbox_socket::{PeerId, PeerState, WebRtcSocket};

use crate::AppState;

#[derive(Resource)]
pub struct MatchConfiguration {
    pub room_id: String,
    pub players: usize,
}

#[derive(Default, Resource)]
pub struct SocketResource(Option<WebRtcSocket>);

#[derive(Debug)]
pub struct GGRSConfig;

impl Config for GGRSConfig {
    type Input = crate::input::PlayerInput;
    type State = u8;
    type Address = PeerId;
}

pub fn start_matchbox_socket(
    mut commands: Commands,
    config: Res<MatchConfiguration>,
    game_settings: Res<crate::config::Config>,
) {
    let room_url = format!("{}/{}", game_settings.matchmaking.server, config.room_id);

    info!("connecting to matchbox server: {:?}", room_url);
    let (socket, message_loop) = WebRtcSocket::new_ggrs(room_url);

    // The message loop needs to be awaited, or nothing will happen.
    // We do this here using bevy's task system.
    let task_pool = IoTaskPool::get();
    task_pool.spawn(message_loop).detach();

    commands.insert_resource(SocketResource(Some(socket)));
}

pub fn watch_for_connected_peers(mut socket: ResMut<SocketResource>) {
    // regularly call update_peers to update the list of connected peers
    for (peer, new_state) in socket.0.as_mut().unwrap().update_peers() {
        // you can also handle the specific dis(connections) as they occur:
        match new_state {
            PeerState::Connected => info!("peer {peer:?} connected"),
            PeerState::Disconnected => info!("peer {peer:?} disconnected"),
        }
    }
}

pub fn start_game_when_ready(
    mut socket: ResMut<SocketResource>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    config: Res<MatchConfiguration>,
    game_settings: Res<crate::config::Config>,
) {
    let connected_peers = socket.0.as_ref().unwrap().connected_peers().count();
    let remaining = config.players - (connected_peers + 1);

    info!("Waiting for {remaining} more player(s)",);

    if remaining > 0 {
        return;
    }

    info!("All peers have joined, going in-game");

    // consume the socket (currently required because ggrs takes ownership of its socket)
    let socket = socket.0.take().unwrap();

    // extract final player list
    let players = socket.players();

    let max_prediction = 12;

    // create a GGRS P2P session
    let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(config.players)
        .with_max_prediction_window(max_prediction)
        .with_input_delay(2)
        .with_fps(game_settings.matchmaking.tick_rate().into())
        .expect("invalid fps");

    for (i, player) in players.into_iter().enumerate() {
        sess_build = sess_build
            .add_player(player, i)
            .expect("failed to add player");
    }

    // start the GGRS session
    let sess = sess_build
        .start_p2p_session(socket)
        .expect("failed to start session");

    commands.insert_resource(Session::P2PSession(sess));

    // transition to in-game state
    next_state.set(AppState::InGame);
}
