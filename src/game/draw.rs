use crate::abilities::Abilities;
use crate::actions::Actions;
use crate::buildings::BuildingTypes;
use crate::color_system::{PlayerColors, TileColor, TileColorStrength};
use crate::game::end_game::GameEnded;
use crate::game::{GameData, BORDER_PADDING_TOTAL};
use crate::player::PlayerPoints;
use crate::GameState;
use bevy::prelude::{Color, Query, Res, State, Without};
use bevy::utils::HashMap;
use bevy_ascii_terminal::{ColorFormatter, StringFormatter, Terminal, TileFormatter};
use bevy_ecs_tilemap::prelude::TilePos;
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::object::{Object, ObjectGridPosition, ObjectInfo, ObjectType};
use bevy_ggf::player::{Player, PlayerMarker};
use std::process::id;

pub fn draw_game_over(
    mut term_query: Query<&mut Terminal>,
    game_ended: Res<GameEnded>,
    player_colors: Res<PlayerColors>,
    game: Res<GameData>,
) {
    let mut term = term_query.single_mut();
    let term_size = term.size();

    for y in 0..20 {
        for x in 0..20 {
            term.clear_string([x + (term_size.x / 2) - 10, y + (term_size.y / 2) - 10], 1);
            term.put_color(
                [x + (term_size.x / 2) - 10, y + (term_size.y / 2) - 10],
                Color::BLACK.bg(),
            );
        }
    }

    term.put_string(
        [(term_size.x / 2) - 10 + 2, (term_size.y / 2) + 10 - 4],
        "!!! GAME OVER !!!".fg(player_colors.get_color(0)),
    );

    match game_ended.player_won {
        true => {
            term.put_string(
                [(term_size.x / 2) - 10 + 4, (term_size.y / 2) + 10 - 6],
                "YOU WON".fg(player_colors.get_color(0)),
            );
        }
        false => {
            let player_color = player_colors.get_color(game_ended.winning_id);
            let ai_color_string = match game_ended.winning_id {
                1 => "RED",
                2 => "GREEN",
                _ => "INDIGO",
            };
            term.put_string(
                [(term_size.x / 2) - 10 + 4, (term_size.y / 2) + 10 - 6],
                String::from(format!("LOST TO: {}", ai_color_string)).fg(player_color),
            );
        }
    }

    term.put_string(
        [(term_size.x / 2) - 10 + 2, (term_size.y / 2) + 10 - 10],
        "Space to return".fg(Color::WHITE),
    );
    term.put_string(
        [(term_size.x / 2) - 10 + 2, (term_size.y / 2) + 10 - 11],
        "to menu".fg(Color::WHITE),
    );
}

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
    player_colors: Res<PlayerColors>,
) {
    let mut term = term_query.single_mut();
    let term_size = term.size();
    term.clear();
    term.put_string(
        [
            game.map_size_x + (BORDER_PADDING_TOTAL / 2) - 10,
            game.map_size_y + (BORDER_PADDING_TOTAL / 2) + 3,
        ],
        "CLRS".fg(player_colors.get_color(0)),
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
            term.put_string([0, 6], "<".fg(Color::WHITE));
            term.put_string([1, 6], "P".fg(Color::WHITE));
            term.put_string([3, 6], "S".fg(Color::WHITE));
            term.put_string([5, 6], "L".fg(Color::WHITE));
            term.put_string([6, 6], ">".fg(Color::WHITE));

            match actions.selected_building {
                BuildingTypes::Pulser => {
                    term.put_string([1, 6], "P".fg(player_colors.get_color(0)));
                }
                BuildingTypes::Scatter => {
                    term.put_string([3, 6], "S".fg(player_colors.get_color(0)));
                }
                BuildingTypes::Line => {
                    term.put_string([5, 6], "L".fg(player_colors.get_color(0)));
                }
            }

            term.put_string([0, 10], "---------".fg(Color::WHITE));

            term.put_string([0, 12], "Abilities".fg(Color::WHITE));
            term.put_string([1, 14], "v".fg(Color::WHITE));
            term.put_string([1, 15], "N".fg(Color::WHITE));
            term.put_string([1, 17], "F".fg(Color::WHITE));
            term.put_string([1, 19], "E".fg(Color::WHITE));
            term.put_string([1, 20], "^".fg(Color::WHITE));

            match actions.selected_ability {
                Abilities::Nuke => {
                    term.put_string([1, 15], "N".fg(player_colors.get_color(0)));
                }
                Abilities::Fortify => {
                    term.put_string([1, 17], "F".fg(player_colors.get_color(0)));
                }
                Abilities::Expand => {
                    term.put_string([1, 19], "E".fg(player_colors.get_color(0)));
                }
            }

            term.put_string([0, 21], "---------".fg(Color::WHITE));
        }
    }

    term.put_string([10, 1], "|".fg(Color::WHITE));
    term.put_string([10, 2], "|".fg(Color::WHITE));
    term.put_string([10, 3], "|".fg(Color::WHITE));
    term.put_string([10, 4], "|".fg(Color::WHITE));
    term.put_string([10, 5], "|".fg(Color::WHITE));
    term.put_string([10, 6], "|".fg(Color::WHITE));
    term.put_string([10, 7], "|".fg(Color::WHITE));

    term.put_string([12, 1], "|".fg(Color::WHITE));
    term.put_string([12, 2], "|".fg(Color::WHITE));
    term.put_string([12, 3], "|".fg(Color::WHITE));
    term.put_string([12, 4], "|".fg(Color::WHITE));
    term.put_string([12, 5], "|".fg(Color::WHITE));
    term.put_string([12, 6], "|".fg(Color::WHITE));
    term.put_string([12, 7], "|".fg(Color::WHITE));

    let mut player_tile_count: HashMap<usize, i32> = HashMap::new();

    for (tile, tile_terrain_info, tile_pos, option) in tile_queries.iter() {
        match option {
            None => {
                let color: Color = match tile_terrain_info.terrain_type.name.as_str() {
                    "BasicColorable" => player_colors.get_colorable(),
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
                let count = player_tile_count.entry(player_marker.id()).or_insert(0);
                let count = *count;
                player_tile_count.insert(player_marker.id(), count.saturating_add(1));

                match tile_color_strength.tile_color_strength {
                    TileColorStrength::Neutral => {
                        let color: Color = match tile_terrain_info.terrain_type.name.as_str() {
                            "BasicColorable" => player_colors.get_colorable(),
                            "NonColorable" => player_colors.get_noncolorable(),
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
                        let player_color = player_colors.get_color(player_marker.id());
                        term.put_color(
                            [
                                tile_pos.x + BORDER_PADDING_TOTAL / 2,
                                tile_pos.y + BORDER_PADDING_TOTAL / 2,
                            ],
                            player_color.with_a(0.2).bg(),
                        );
                    }
                    TileColorStrength::Two => {
                        let player_color = player_colors.get_color(player_marker.id());
                        term.put_color(
                            [
                                tile_pos.x + BORDER_PADDING_TOTAL / 2,
                                tile_pos.y + BORDER_PADDING_TOTAL / 2,
                            ],
                            player_color.with_a(0.4).bg(),
                        );
                    }
                    TileColorStrength::Three => {
                        let player_color = player_colors.get_color(player_marker.id());
                        term.put_color(
                            [
                                tile_pos.x + BORDER_PADDING_TOTAL / 2,
                                tile_pos.y + BORDER_PADDING_TOTAL / 2,
                            ],
                            player_color.with_a(0.6).bg(),
                        );
                    }
                    TileColorStrength::Four => {
                        let player_color = player_colors.get_color(player_marker.id());
                        term.put_color(
                            [
                                tile_pos.x + BORDER_PADDING_TOTAL / 2,
                                tile_pos.y + BORDER_PADDING_TOTAL / 2,
                            ],
                            player_color.with_a(0.8).bg(),
                        );
                    }
                    TileColorStrength::Five => {
                        let player_color = player_colors.get_color(player_marker.id());
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

    if let Some(player_tile_count) = player_tile_count.get(&0) {
        let player_tile_count =
            *player_tile_count as f32 / (game.map_size_y as f32 * game.map_size_x as f32);
        if player_tile_count > 0.0 {
            term.put_color([11, 1], player_colors.get_color(0).bg());
        }
        if player_tile_count > 0.2 {
            term.put_color([11, 2], player_colors.get_color(0).bg());
        }
        if player_tile_count > 0.4 {
            term.put_color([11, 3], player_colors.get_color(0).bg());
        }
        if player_tile_count > 0.5 {
            term.put_color([11, 4], player_colors.get_color(0).bg());
        }
        if player_tile_count > 0.6 {
            term.put_color([11, 5], player_colors.get_color(0).bg());
        }
        if player_tile_count > 0.8 {
            term.put_color([11, 6], player_colors.get_color(0).bg());
        }
        if player_tile_count > 1.0 {
            term.put_color([11, 7], player_colors.get_color(0).bg());
        }
    }

    term.put_string(
        [14, 3],
        String::from(format!("{}", player_tile_count.get(&0).unwrap_or(&0)))
            .fg(player_colors.get_color(0)),
    );
    term.put_string([14, 2], "-------".fg(Color::WHITE));
    term.put_string(
        [14, 1],
        String::from(format!("{}", game.map_size_x * game.map_size_y)).fg(Color::WHITE),
    );

    for (id, count) in player_tile_count.iter() {
        if id == &0 {
            continue;
        }
        let player_color = player_colors.get_color(*id);
        let diff = match id {
            3 => 26,
            2 => 24,
            1 => 22,
            0 => 28,
            _ => 0,
        };
        let player_tile_count = *count as f32 / (game.map_size_y as f32 * game.map_size_x as f32);
        if player_tile_count > 0.0 {
            term.put_color([0, diff], player_color.bg());
        }
        if player_tile_count > 0.2 {
            term.put_color([1, diff], player_color.bg());
        }
        if player_tile_count > 0.4 {
            term.put_color([2, diff], player_color.bg());
        }
        if player_tile_count > 0.5 {
            term.put_color([3, diff], player_color.bg());
        }
        if player_tile_count > 0.6 {
            term.put_color([4, diff], player_color.bg());
        }
        if player_tile_count > 0.8 {
            term.put_color([5, diff], player_color.bg());
        }
        if player_tile_count > 1.0 {
            term.put_color([6, diff], player_color.bg());
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
            "Nuke" => {
                term.put_char(
                    [
                        object_pos.tile_position.x + BORDER_PADDING_TOTAL / 2,
                        object_pos.tile_position.y + BORDER_PADDING_TOTAL / 2,
                    ],
                    'N'.fg(Color::GRAY),
                );
            }
            "Fortify" => {
                term.put_char(
                    [
                        object_pos.tile_position.x + BORDER_PADDING_TOTAL / 2,
                        object_pos.tile_position.y + BORDER_PADDING_TOTAL / 2,
                    ],
                    'F'.fg(Color::GRAY),
                );
            }
            "Expand" => {
                term.put_char(
                    [
                        object_pos.tile_position.x + BORDER_PADDING_TOTAL / 2,
                        object_pos.tile_position.y + BORDER_PADDING_TOTAL / 2,
                    ],
                    'E'.fg(Color::GRAY),
                );
            }
            _ => {}
        }
    }
}
