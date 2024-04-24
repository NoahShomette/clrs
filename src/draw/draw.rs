use crate::color_system::TileColor;
use crate::draw::{DrawObject, DrawTile, MyColorLens};
use crate::game::state::OldTileState;
use crate::game::GameData;
use crate::loading::TextureAssets;
use crate::ui::PlayerColors;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilePos;
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::object::{ObjectGridPosition, ObjectInfo};
use bevy_ggf::player::PlayerMarker;
use bevy_tweening::lens::TransformScaleLens;
use bevy_tweening::{Animator, EaseFunction, RepeatCount, Tween};
use bevy_vector_shapes::prelude::{RectangleBundle, ShapeConfig, ThicknessType};
use bevy_vector_shapes::render::ShapePipelineType;
use std::time::Duration;

use super::UpdateTile;

pub const TILE_SIZE: f32 = 32.0;
pub const TILE_GAP: f32 = 0.0;
pub const TILE_OUTLINE: f32 = 2.0;

pub const OBJECT_SIZE: f32 = 24.0;

#[derive(Component)]
pub struct ChildGraphics;

#[derive(Component)]
pub struct ChildBackgroundGraphics;

pub fn draw_tile_backgrounds(
    game_info: Res<GameData>,
    tile_query: Query<(Entity, &TileTerrainInfo, &TilePos), (Added<UpdateTile>, Without<Children>)>,
    player_colors: Res<PlayerColors>,
    mut commands: Commands,
) {
    for (entity, tile_terrain_info, tile_pos) in tile_query.iter() {
        let card_x = (tile_pos.x as f32 * (TILE_SIZE + TILE_GAP))
            - ((game_info.map_size_x as f32 * (TILE_SIZE + TILE_GAP)) / 2.0);
        let card_y = (tile_pos.y as f32 * (TILE_SIZE + TILE_GAP))
            - ((game_info.map_size_y as f32 * (TILE_SIZE + TILE_GAP)) / 2.0);

        let child = commands
            .spawn((
                bevy_vector_shapes::shapes::ShapeBundle::rect(
                    &ShapeConfig {
                        transform: Transform {
                            translation: Vec3 {
                                x: card_x,
                                y: card_y,
                                z: 1.0,
                            },
                            rotation: Default::default(),
                            scale: Vec3 {
                                x: 1.0,
                                y: 1.0,
                                z: 1.0,
                            },
                        },
                        color: match tile_terrain_info.terrain_type.terrain_class.name.as_str() {
                            "NonColorable" => player_colors.get_noncolorable(),
                            _ => player_colors.get_colorable(),
                        },
                        hollow: false,
                        cap: Default::default(),
                        thickness: TILE_OUTLINE,
                        thickness_type: ThicknessType::World,
                        corner_radii: Default::default(),
                        render_layers: None,
                        alpha_mode: AlphaMode::Blend,
                        disable_laa: false,
                        instance_id: 0,
                        canvas: None,
                        texture: None,
                        alignment: Default::default(),
                        roundness: 0.0,
                        pipeline: ShapePipelineType::Shape2d,
                    },
                    Vec2 {
                        x: TILE_SIZE,
                        y: TILE_SIZE,
                    },
                ),
                ChildBackgroundGraphics,
            ))
            .id();

        commands.entity(entity).push_children(&[child]);
    }
}

/// Tiles always get [`UpdateTile`] when changed but only new tiles get [`DrawTile`]
pub fn draw_tiles(
    game_info: Res<GameData>,
    tile_query: Query<
        (
            Entity,
            &TileTerrainInfo,
            &TilePos,
            Option<&OldTileState>,
            Option<&DrawTile>,
            Option<&Children>,
            Option<(&TileColor, &PlayerMarker)>,
        ),
        Added<UpdateTile>,
    >,
    children_query: Query<(&ChildGraphics, &Transform)>,
    player_colors: Res<PlayerColors>,
    mut commands: Commands,
) {
    for (entity, tile_terrain_info, tile_pos, old_tile_state, opt_draw_tile, children, options) in
        tile_query.iter()
    {
        let card_x = (tile_pos.x as f32 * (TILE_SIZE + TILE_GAP))
            - ((game_info.map_size_x as f32 * (TILE_SIZE + TILE_GAP)) / 2.0);
        let card_y = (tile_pos.y as f32 * (TILE_SIZE + TILE_GAP))
            - ((game_info.map_size_y as f32 * (TILE_SIZE + TILE_GAP)) / 2.0);

        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(100),
            MyColorLens {
                start: match &old_tile_state.is_some() {
                    false => match tile_terrain_info.terrain_type.terrain_class.name.as_str() {
                        "NonColorable" => player_colors.get_noncolorable(),
                        _ => player_colors.get_colorable(),
                    },
                    true => match old_tile_state.unwrap().player_id.is_some() {
                        true => player_colors.get_color(old_tile_state.unwrap().player_id.unwrap()),
                        false => match tile_terrain_info.terrain_type.terrain_class.name.as_str() {
                            "NonColorable" => player_colors.get_noncolorable(),
                            _ => player_colors.get_colorable(),
                        },
                    },
                },
                end: match options {
                    None => match tile_terrain_info.terrain_type.terrain_class.name.as_str() {
                        "NonColorable" => player_colors.get_noncolorable(),
                        _ => player_colors.get_colorable(),
                    },
                    Some((_, player_marker)) => player_colors.get_color(player_marker.id()),
                },
            },
        )
        .with_repeat_count(RepeatCount::Finite(1));

        let tile_color_size = match children.is_some() {
            true => {
                let mut size = Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                };
                for child in children.unwrap().iter() {
                    if let Ok((_, transform)) = children_query.get(*child) {
                        size = transform.scale;
                    }
                }
                size
            }
            false => match old_tile_state.is_some() {
                true => match &old_tile_state.unwrap().tile_color {
                    Some(tile_color) => tile_color.get_scale(),
                    None => Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    },
                },
                false => Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
            },
        };

        let transform_tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(350),
            TransformScaleLens {
                start: tile_color_size,
                end: match options {
                    None => Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    },
                    Some((tile_color, _)) => tile_color.get_scale(),
                },
            },
        )
        .with_repeat_count(RepeatCount::Finite(1));

        let shape = bevy_vector_shapes::shapes::ShapeBundle::rect(
            &ShapeConfig {
                transform: Transform {
                    translation: Vec3 {
                        x: card_x,
                        y: card_y,
                        z: 2.0,
                    },
                    rotation: Default::default(),
                    scale: tile_color_size,
                },
                color: match options {
                    None => match tile_terrain_info.terrain_type.terrain_class.name.as_str() {
                        "NonColorable" => player_colors.get_noncolorable(),
                        _ => player_colors.get_colorable(),
                    },
                    Some((_, player_marker)) => player_colors.get_color(player_marker.id()),
                },
                hollow: false,
                cap: Default::default(),
                thickness: TILE_OUTLINE,
                thickness_type: ThicknessType::World,
                corner_radii: Default::default(),
                render_layers: None,
                alpha_mode: AlphaMode::Blend,
                disable_laa: false,
                instance_id: 0,
                canvas: None,
                texture: None,
                alignment: Default::default(),
                roundness: 0.0,
                pipeline: ShapePipelineType::Shape2d,
            },
            Vec2 {
                x: TILE_SIZE,
                y: TILE_SIZE,
            },
        );

        if opt_draw_tile.is_some() {
            let child = commands
                .spawn((
                    ChildGraphics,
                    shape,
                    Animator::new(tween),
                    Animator::new(transform_tween),
                ))
                .id();
            commands.entity(entity).push_children(&[child]);
            commands.entity(entity).remove::<DrawTile>();
        } else if children.is_some() {
            for child in children.unwrap().iter() {
                if let Ok(_) = children_query.get(*child) {
                    commands.entity(*child).insert((
                        shape,
                        Animator::new(tween),
                        Animator::new(transform_tween),
                    ));
                    break;
                }
            }
        }

        commands.entity(entity).remove::<OldTileState>();
        commands.entity(entity).remove::<UpdateTile>();
    }
}

pub fn draw_objects(
    game_info: Res<GameData>,
    tile_query: Query<(Entity, &DrawObject, &ObjectInfo, &ObjectGridPosition)>,
    mut commands: Commands,
    texture_assets: Res<TextureAssets>,
) {
    for (entity, _, object_info, tile_pos) in tile_query.iter() {
        let card_x = (tile_pos.tile_position.x as f32 * (TILE_SIZE + TILE_GAP))
            - ((game_info.map_size_x as f32 * (TILE_SIZE + TILE_GAP)) / 2.0);
        let card_y = (tile_pos.tile_position.y as f32 * (TILE_SIZE + TILE_GAP))
            - ((game_info.map_size_y as f32 * (TILE_SIZE + TILE_GAP)) / 2.0);

        let spawn_point: Vec3 = Vec3 {
            x: card_x,
            y: card_y,
            z: 3.0,
        };

        let child = commands
            .spawn(bevy_vector_shapes::shapes::ShapeBundle::rect(
                &ShapeConfig {
                    transform: Transform {
                        translation: spawn_point,
                        rotation: Default::default(),
                        scale: Vec3 {
                            x: 1.0,
                            y: 1.0,
                            z: 1.0,
                        },
                    },
                    color: Color::BLUE,
                    thickness: 0.0,
                    thickness_type: Default::default(),
                    alignment: Default::default(),
                    hollow: false,
                    cap: Default::default(),
                    roundness: 0.0,
                    corner_radii: Vec4::new(0.0, 0.0, 0.0, 0.0),
                    render_layers: None,
                    alpha_mode: AlphaMode::Blend,
                    disable_laa: false,
                    instance_id: 0,
                    canvas: None,
                    texture: match object_info.object_type.name.as_str() {
                        "Pulser" => Some(texture_assets.pulser.clone()),
                        "Scatter" => Some(texture_assets.scatter.clone()),
                        "Line" => Some(texture_assets.line.clone()),
                        "Nuke" => Some(texture_assets.nuke.clone()),
                        "Fortify" => Some(texture_assets.fortify.clone()),
                        "Expand" => Some(texture_assets.expand.clone()),
                        &_ => Some(texture_assets.pulser.clone()),
                    },
                    pipeline: ShapePipelineType::Shape2d,
                },
                Vec2 {
                    x: OBJECT_SIZE,
                    y: OBJECT_SIZE,
                },
            ))
            .id();
        commands.entity(entity).push_children(&[child]);
        commands.entity(entity).remove::<DrawObject>();
    }
}

/*
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

 */
