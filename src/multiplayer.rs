use bevy::{prelude::*, tasks::IoTaskPool};
use ggrs::{Config, PlayerHandle};
use bevy_ggrs::{PlayerInputs, Rollback, RollbackIdProvider, Session};
use matchbox_socket::{WebRtcSocket, PeerState};
use std::{hash::Hash, net::SocketAddr};
use bytemuck::{Pod, Zeroable};

#[derive(Default, Resource)]
struct SocketResource(Option<WebRtcSocket>);

#[derive(Debug)]
pub struct GGRSConfig;

impl Config for GGRSConfig {
    type Input = PlayerInput;
    type State = u8;
    type Address = SocketAddr;
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Pod, Zeroable, Default)]
pub struct PlayerInput {
    pub inp: u8,
}

impl PlayerInput {
    const INPUT_UP: u8 = 1 << 0;
    const INPUT_DOWN: u8 = 1 << 1;
    const INPUT_LEFT: u8 = 1 << 2;
    const INPUT_RIGHT: u8 = 1 << 3;

    fn get(&self, mask: u8) -> bool {
        self.inp & mask == 1
    }

    fn set(&mut self, state: bool, mask: u8) {
        if state {
            self.inp |= mask;
        } else {
            self.inp &= !mask;
        }
    }

    pub fn get_up(&self) -> bool {
        self.get(Self::INPUT_UP)
    }

    pub fn set_up(&mut self, state: bool) {
        self.set(state, Self::INPUT_UP);
    }

    pub fn get_down(&self) -> bool {
        self.get(Self::INPUT_DOWN)
    }

    pub fn set_down(&mut self, state: bool) {
        self.set(state, Self::INPUT_DOWN);
    }

    pub fn get_left(&self) -> bool {
        self.get(Self::INPUT_LEFT)
    }

    pub fn set_left(&mut self, state: bool) {
        self.set(state, Self::INPUT_LEFT);
    }

    pub fn get_right(&self) -> bool {
        self.get(Self::INPUT_RIGHT)
    }

    pub fn set_right(&mut self, state: bool) {
        self.set(state, Self::INPUT_RIGHT);
    }
}

pub fn input(_handle: In<PlayerHandle>, keyboard_input: Res<Input<KeyCode>>) -> PlayerInput {
    let mut input = PlayerInput::default();

    if keyboard_input.pressed(KeyCode::W) {
        input.set_up(true);
    }
    if keyboard_input.pressed(KeyCode::A) {
        input.set_left(true);
    }
    if keyboard_input.pressed(KeyCode::S) {
        input.set_down(true);
    }
    if keyboard_input.pressed(KeyCode::D) {
        input.set_right(true);
    }

    input
}

pub fn start_matchbox_socket(mut commands: Commands) {
    let room_id ="bevy_ggrs?next=1";

    let room_url = format!("localhost:44444/{}", room_id);
    info!("connecting to matchbox server: {:?}", room_url);
    let (socket, message_loop) = WebRtcSocket::new_ggrs(room_url);

    // The message loop needs to be awaited, or nothing will happen.
    // We do this here using bevy's task system.
    let task_pool = IoTaskPool::get();
    task_pool.spawn(message_loop).detach();

    commands.insert_resource(SocketResource(Some(socket)));
}

/*
fn lobby_system(
    mut app_state: ResMut<State<crate::AppState>>,
    mut socket: ResMut<SocketResource>,
    mut commands: Commands,
) {
    // regularly call update_peers to update the list of connected peers
    for (peer, new_state) in socket.0.as_mut().unwrap().update_peers() {
        // you can also handle the specific dis(connections) as they occur:
        match new_state {
            PeerState::Connected => info!("peer {peer:?} connected"),
            PeerState::Disconnected => info!("peer {peer:?} disconnected"),
        }
    }

    let connected_peers = socket.0.as_ref().unwrap().connected_peers().count();
    let remaining = args.players - (connected_peers + 1);
    query.single_mut().sections[0].value = format!("Waiting for {remaining} more player(s)",);
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
        .with_num_players(args.players)
        .with_max_prediction_window(max_prediction)
        .with_input_delay(2)
        .with_fps(FPS)
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
    app_state
        .set(AppState::InGame)
        .expect("Tried to go in-game while already in-game");
}
 */