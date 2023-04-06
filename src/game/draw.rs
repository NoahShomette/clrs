use crate::color_system::{PlayerColors, TileColor, TileColorStrength};
use crate::game::BORDER_PADDING_TOTAL;
use bevy::prelude::{Color, Query, Without};
use bevy_ascii_terminal::{ColorFormatter, Terminal, TileFormatter};
use bevy_ecs_tilemap::prelude::TilePos;
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::object::{Object, ObjectGridPosition, ObjectInfo, ObjectType};
use bevy_ggf::player::PlayerMarker;

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
            &TileColor,
            Option<&PlayerMarker>,
        ),
        Without<Object>,
    >,
) {
    let mut term = term_query.single_mut();
    term.clear();
    for (tile, tile_terrain_info, tile_pos, tile_color_strength, player_marker_option) in
        tile_queries.iter()
    {
        match tile_color_strength.tile_color_strength {
            TileColorStrength::Neutral => {
                let color: Color = match tile_terrain_info.terrain_type.name.as_str() {
                    "BasicColorable" => Color::DARK_GRAY,
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
                let player_color = PlayerColors::get_color(player_marker_option.unwrap().id());
                term.put_color(
                    [
                        tile_pos.x + BORDER_PADDING_TOTAL / 2,
                        tile_pos.y + BORDER_PADDING_TOTAL / 2,
                    ],
                    player_color.with_a(0.2).bg(),
                );
            }
            TileColorStrength::Two => {
                let player_color = PlayerColors::get_color(player_marker_option.unwrap().id());
                term.put_color(
                    [
                        tile_pos.x + BORDER_PADDING_TOTAL / 2,
                        tile_pos.y + BORDER_PADDING_TOTAL / 2,
                    ],
                    player_color.with_a(0.4).bg(),
                );
            }
            TileColorStrength::Three => {
                let player_color = PlayerColors::get_color(player_marker_option.unwrap().id());
                term.put_color(
                    [
                        tile_pos.x + BORDER_PADDING_TOTAL / 2,
                        tile_pos.y + BORDER_PADDING_TOTAL / 2,
                    ],
                    player_color.with_a(0.6).bg(),
                );
            }
            TileColorStrength::Four => {
                let player_color = PlayerColors::get_color(player_marker_option.unwrap().id());
                term.put_color(
                    [
                        tile_pos.x + BORDER_PADDING_TOTAL / 2,
                        tile_pos.y + BORDER_PADDING_TOTAL / 2,
                    ],
                    player_color.with_a(0.8).bg(),
                );
            }
            TileColorStrength::Five => {
                let player_color = PlayerColors::get_color(player_marker_option.unwrap().id());
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
            _ => {}
        }
    }
}
