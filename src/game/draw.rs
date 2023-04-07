use crate::abilities::Abilities;
use crate::actions::Actions;
use crate::buildings::BuildingTypes;
use crate::color_system::{PlayerColors, TileColor, TileColorStrength};
use crate::game::{GameData, BORDER_PADDING_TOTAL};
use crate::player::PlayerPoints;
use bevy::prelude::{Color, Query, Res, Without};
use bevy_ascii_terminal::{ColorFormatter, StringFormatter, Terminal, TileFormatter};
use bevy_ecs_tilemap::prelude::TilePos;
use bevy_ggf::game_core::Game;
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::object::{Object, ObjectGridPosition, ObjectInfo, ObjectType};
use bevy_ggf::player::{Player, PlayerMarker};

pub fn draw_game(
    mut term_query: Query<&mut Terminal>,
    object_queries: Query<
        (&Object, &ObjectGridPosition, &ObjectInfo, &PlayerMarker),
        Without<Tile>,
    >,
    tile_queries: Query<
        (
            &Tile,
            &TileTerrainInfo,
            &TilePos,
            Option<(&TileColor, &PlayerMarker)>,
        ),
        Without<Object>,
    >,
    player_queries: Query<(&Player, &PlayerPoints), Without<PlayerMarker>>,
    action_queries: Query<(&PlayerMarker, &Actions), Without<Player>>,
    game: Res<GameData>,
) {
    let mut term = term_query.single_mut();
    term.clear();
    term.put_string(
        [
            game.map_size_x + (BORDER_PADDING_TOTAL / 2) - 10,
            game.map_size_y + (BORDER_PADDING_TOTAL / 2) + 3,
        ],
        "CLRS".fg(Color::GREEN),
    );

    for (player_query, player_points) in player_queries.iter() {
        if player_query.id() == 0 {
            term.put_string(
                [1, 3],
                String::from(format!("AP: {}", player_points.ability_points)).fg(Color::WHITE),
            );
            term.put_string(
                [1, 1],
                String::from(format!("BP: {}", player_points.building_points)).fg(Color::WHITE),
            );
        }
    }

    for (player_marker, actions) in action_queries.iter() {
        if player_marker.id() == 0 {
            term.put_string([0, 8], "Buildings".fg(Color::WHITE));
            term.put_string([1, 6], "P".fg(Color::WHITE));
            term.put_string([3, 6], "S".fg(Color::WHITE));
            term.put_string([5, 6], "L".fg(Color::WHITE));

            match actions.selected_building {
                BuildingTypes::Pulser => {
                    term.put_string([1, 6], "P".fg(Color::GREEN));
                }
                BuildingTypes::Scatter => {
                    term.put_string([3, 6], "S".fg(Color::GREEN));
                }
                BuildingTypes::Line => {
                    term.put_string([5, 6], "L".fg(Color::GREEN));
                }
            }

            term.put_string([0, 10], "---------".fg(Color::WHITE));

            term.put_string([0, 12], "Abilities".fg(Color::WHITE));
            term.put_string([1, 14], "N".fg(Color::WHITE));
            term.put_string([1, 16], "S".fg(Color::WHITE));
            term.put_string([1, 18], "F".fg(Color::WHITE));

            match actions.selected_ability {
                Abilities::Nuke => {
                    term.put_string([1, 14], "N".fg(Color::GREEN));
                }
                Abilities::Sacrifice => {
                    term.put_string([1, 16], "S".fg(Color::GREEN));
                }
                Abilities::Boost => {
                    term.put_string([1, 18], "F".fg(Color::GREEN));
                }
            }
        }
    }

    term.put_string([10, 1], "|".fg(Color::WHITE));
    term.put_string([10, 2], "|".fg(Color::WHITE));
    term.put_string([10, 3], "|".fg(Color::WHITE));
    term.put_string([10, 4], "|".fg(Color::WHITE));
    term.put_string([10, 5], "|".fg(Color::WHITE));
    term.put_string([10, 6], "|".fg(Color::WHITE));
    term.put_string([10, 7], "|".fg(Color::WHITE));

    for (tile, tile_terrain_info, tile_pos, option) in tile_queries.iter() {
        match option {
            None => {
                let color: Color = match tile_terrain_info.terrain_type.name.as_str() {
                    "BasicColorable" => Color::rgb(0.05, 0.05, 0.05),
                    "NonColorable" => Color::BLACK,
                    _ => Color::BLACK,
                };
                term.put_color(
                    [
                        tile_pos.x + BORDER_PADDING_TOTAL / 2,
                        tile_pos.y + BORDER_PADDING_TOTAL / 2,
                    ],
                    color.bg(),
                );
            }
            Some((tile_color_strength, player_marker)) => {
                match tile_color_strength.tile_color_strength {
                    TileColorStrength::Neutral => {
                        let color: Color = match tile_terrain_info.terrain_type.name.as_str() {
                            "BasicColorable" => Color::rgb(0.05, 0.05, 0.05),
                            "NonColorable" => Color::BLACK,
                            _ => Color::BLACK,
                        };
                        term.put_color(
                            [
                                tile_pos.x + BORDER_PADDING_TOTAL / 2,
                                tile_pos.y + BORDER_PADDING_TOTAL / 2,
                            ],
                            color.bg(),
                        );
                    }
                    TileColorStrength::One => {
                        let player_color = PlayerColors::get_color(player_marker.id());
                        term.put_color(
                            [
                                tile_pos.x + BORDER_PADDING_TOTAL / 2,
                                tile_pos.y + BORDER_PADDING_TOTAL / 2,
                            ],
                            player_color.with_a(0.2).bg(),
                        );
                    }
                    TileColorStrength::Two => {
                        let player_color = PlayerColors::get_color(player_marker.id());
                        term.put_color(
                            [
                                tile_pos.x + BORDER_PADDING_TOTAL / 2,
                                tile_pos.y + BORDER_PADDING_TOTAL / 2,
                            ],
                            player_color.with_a(0.4).bg(),
                        );
                    }
                    TileColorStrength::Three => {
                        let player_color = PlayerColors::get_color(player_marker.id());
                        term.put_color(
                            [
                                tile_pos.x + BORDER_PADDING_TOTAL / 2,
                                tile_pos.y + BORDER_PADDING_TOTAL / 2,
                            ],
                            player_color.with_a(0.6).bg(),
                        );
                    }
                    TileColorStrength::Four => {
                        let player_color = PlayerColors::get_color(player_marker.id());
                        term.put_color(
                            [
                                tile_pos.x + BORDER_PADDING_TOTAL / 2,
                                tile_pos.y + BORDER_PADDING_TOTAL / 2,
                            ],
                            player_color.with_a(0.8).bg(),
                        );
                    }
                    TileColorStrength::Five => {
                        let player_color = PlayerColors::get_color(player_marker.id());
                        term.put_color(
                            [
                                tile_pos.x + BORDER_PADDING_TOTAL / 2,
                                tile_pos.y + BORDER_PADDING_TOTAL / 2,
                            ],
                            player_color.with_a(1.0).bg(),
                        );
                    }
                }
            }
        }
    }
    for (object, object_pos, object_info, player_marker) in object_queries.iter() {
        match object_info.object_type.name.as_str() {
            "Pulser" => {
                term.put_char(
                    [
                        object_pos.tile_position.x + BORDER_PADDING_TOTAL / 2,
                        object_pos.tile_position.y + BORDER_PADDING_TOTAL / 2,
                    ],
                    'P'.fg(Color::WHITE),
                );
            }
            "Line" => {
                term.put_char(
                    [
                        object_pos.tile_position.x + BORDER_PADDING_TOTAL / 2,
                        object_pos.tile_position.y + BORDER_PADDING_TOTAL / 2,
                    ],
                    'L'.fg(Color::WHITE),
                );
            }
            "Scatter" => {
                term.put_char(
                    [
                        object_pos.tile_position.x + BORDER_PADDING_TOTAL / 2,
                        object_pos.tile_position.y + BORDER_PADDING_TOTAL / 2,
                    ],
                    'S'.fg(Color::WHITE),
                );
            }
            _ => {}
        }
    }
}
