use bevy::{app::AppExit, prelude::*};
use shared::models::network_message::NetworkMessage;

use crate::{networking::NetworkClient, player::ConnectionInfo, GameState};

use super::despawn_screen;

pub fn menu_plugin(app: &mut App) {
    app
        // Systems to handle the main menu screen
        .add_systems(OnEnter(GameState::Menu), main_menu_setup)
        .add_systems(
            Update,
            (update_connections_text, menu_action).run_if(in_state(GameState::Menu)),
        )
        .add_systems(OnExit(GameState::Menu), despawn_screen::<OnMainMenuScreen>);
}

#[derive(Component)]
struct OnMainMenuScreen;

#[derive(Component)]
struct ConnectionsText;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Quit,
}

fn main_menu_setup(mut commands: Commands) {
    let button_node = Node {
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_font = TextFont {
        font_size: 33.0,
        ..default()
    };

    // spawn main 'canvas'
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
            BackgroundColor(Color::srgb(0.31, 0.31, 0.31)),
            OnMainMenuScreen,
        ))
        .with_children(|parent| {
            // game name
            parent.spawn((
                Text::new("DataDrift"),
                TextFont {
                    font_size: 67.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // play button
            parent
                .spawn((
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Play,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("New Game"),
                        button_text_font.clone(),
                        TextColor(Color::WHITE),
                    ));
                });

            // quit button
            parent
                .spawn((
                    Button,
                    button_node,
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Quit,
                ))
                .with_children(|parent| {
                    parent.spawn((Text::new("Quit"), button_text_font, TextColor(Color::WHITE)));
                });

            // count of players connected text
            parent
                .spawn((
                    Text::new("Players connected: "),
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
                    ConnectionsText,
                ));
        });
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut game_state: ResMut<NextState<GameState>>,
    network_client: Res<NetworkClient>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Play => {
                    // send play request to backend - main will handle response
                    network_client.send_message(NetworkMessage::RequestStart(()));
                }

                MenuButtonAction::Quit => {
                    app_exit_events.send(AppExit::Success);
                }
            }
        }
    }
}

fn update_connections_text(
    mut query: Query<&mut TextSpan, With<ConnectionsText>>,
    connection_info: Res<ConnectionInfo>,
) {
    for mut span in &mut query {
        **span = format!("{}", connection_info.players_connected);
    }
}
