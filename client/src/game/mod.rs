use bevy::prelude::*;

use crate::{
    despawn_screen,
    player::{
        move_player, spawn_players_according_to_backend, ConnectionInfo, Player, RenderedTrails,
    },
    BackendState, GameState,
};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum FrontendLobbyState {
    #[default]
    Loading,
    Countdown,
    Running,
    Finished,
}

#[derive(Resource)]
pub struct EndTimer {
    timer: Timer,
}

pub fn game_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Game), game_setup)
        .init_state::<FrontendLobbyState>()
        .add_systems(
            OnEnter(FrontendLobbyState::Countdown),
            (setup_countdown, setup_players),
        )
        .add_systems(OnEnter(FrontendLobbyState::Finished), setup_finished)
        .add_systems(
            Update,
            (update_countdown_text).run_if(in_state(FrontendLobbyState::Countdown)),
        )
        .add_systems(
            Update,
            (align_with_backend, move_player).run_if(in_state(FrontendLobbyState::Running)),
        )
        .add_systems(
            Update,
            check_quit.run_if(in_state(FrontendLobbyState::Finished)),
        )
        .add_systems(
            OnExit(FrontendLobbyState::Countdown),
            despawn_screen::<OnCountdown>,
        )
        .add_systems(OnExit(GameState::Game), despawn_screen::<OnGameScreen>);
}

#[derive(Component)]
pub struct OnGameScreen;

#[derive(Component)]
struct GameField;

#[derive(Component)]
struct OnCountdown;

#[derive(Component)]
struct CountdownText;

fn game_setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            OnGameScreen,
            Transform::from_xyz(0., 0., -1.),
        ))
        .with_children(|parent| {
            // Display the logo
            parent.spawn((
                Node {
                    width: Val::Px(1000.0),
                    height: Val::Px(1000.0),

                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.5)),
                GameField,
            ));
        });
}

fn setup_countdown(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            OnCountdown,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Text::new("Starts in: "),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    OnCountdown,
                ))
                .with_child((
                    TextSpan::default(),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    CountdownText,
                ));
        });
}

fn update_countdown_text(
    mut query: Query<&mut TextSpan, With<CountdownText>>,
    backend_state: Res<BackendState>,
) {
    for mut span in &mut query {
        **span = format!("{}", backend_state.countdown);
    }
}

fn setup_players(
    mut commands: Commands,
    backend_state: Res<BackendState>,
    connection_info: Res<ConnectionInfo>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(RenderedTrails { count: 0 });
    spawn_players_according_to_backend(commands, backend_state, connection_info, asset_server);
}

fn align_with_backend(
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut Player)>,
    backend_state: Res<BackendState>,
    mut rendered_trails: ResMut<RenderedTrails>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for backend_player in backend_state.players.iter() {
        for (i, &trail_segment) in backend_player.trail.iter().enumerate() {
            if (i as u32) < rendered_trails.count {
                continue; // skip already rendered
            }

            commands.spawn((
                Mesh2d(meshes.add(Circle::new(5.0))),
                MeshMaterial2d(materials.add(Color::srgb(0.9, 0.2, 0.5))),
                Transform::from_xyz(trail_segment.0, trail_segment.1, 0.0),
                OnGameScreen,
            ));
        }

        // update position of player
        for (mut player_pos, mut player) in &mut query {
            if player.uuid == backend_player.id {
                let quat = Quat::from_rotation_z(backend_player.direction);
                player.is_alive = backend_player.is_alive;
                player.rotation = quat;
                player_pos.translation.x = backend_player.position_x;
                player_pos.translation.y = backend_player.position_y;
            }
        }
    }

    if backend_state.players.len() > 0 {
        if let Some(first_player) = backend_state.players.get(0) {
            rendered_trails.count = first_player.trail.len() as u32;
        }
    }
}

fn setup_finished(mut commands: Commands) {
    commands.insert_resource(EndTimer {
        timer: Timer::from_seconds(3.0, TimerMode::Once),
    });

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            OnGameScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Text::new("Game finished!"),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ))
                .with_child((
                    TextSpan::default(),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
        });
}

fn check_quit(
    mut end_timer: ResMut<EndTimer>,
    time: Res<Time>,
    mut backend_state: ResMut<BackendState>,
    mut game_state: ResMut<NextState<GameState>>,
    mut lobby_state: ResMut<NextState<FrontendLobbyState>>,
    mut rendered_trails: ResMut<RenderedTrails>,
) {
    if end_timer.timer.tick(time.delta()).finished() {
        backend_state.countdown = 0;
        rendered_trails.count = 0;
        backend_state.players = Vec::new();
        lobby_state.set(FrontendLobbyState::Loading);
        game_state.set(GameState::Menu);
    }
}
