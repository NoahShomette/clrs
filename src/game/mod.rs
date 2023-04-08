﻿mod draw;
mod end_game;
mod state;

use crate::actions::Actions;
use crate::ai::run_ai;
use crate::buildings::line::simulate_lines;
use crate::buildings::pulser::{simulate_pulsers, Pulser};
use crate::buildings::scatter::simulate_scatterers;
use crate::buildings::{
    destroy_buildings, update_building_timers, Activate, Building, BuildingCooldown, BuildingMarker,
};
use crate::color_system::{
    handle_color_conflicts, update_color_conflicts, ColorConflictEvent, ColorConflicts, TileColor,
};
use crate::game::draw::{draw_game, draw_game_over};
use crate::game::end_game::{check_game_ended, update_game_end_state};
use crate::game::state::update_game_state;
use crate::map::MapCommandsExt;
use crate::player::{update_player_points, PlayerPoints};
use crate::GameState;
use bevy::app::App;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ascii_terminal::Terminal;
use bevy_ecs_tilemap::prelude::{TilePos, TilemapSize, TilemapTileSize, TilemapType};
use bevy_ggf::game_core::command::{GameCommand, GameCommands};
use bevy_ggf::game_core::runner::GameRunner;
use bevy_ggf::game_core::{Game, GameBuilder, GameRuntime};
use bevy_ggf::mapping::terrain::{TerrainClass, TerrainType};
use bevy_ggf::mapping::tiles::{
    ObjectStackingClass, StackingClass, TileObjectStacks, TileObjectStacksCount,
};
use bevy_ggf::mapping::{GameBuilderMappingExt, MapId};
use bevy_ggf::movement::{GameBuilderMovementExt, TileMovementCosts};
use bevy_ggf::object::{
    Object, ObjectClass, ObjectGridPosition, ObjectGroup, ObjectInfo, ObjectType,
};
use bevy_ggf::player::{Player, PlayerMarker};

pub struct GameCorePlugin;
impl Plugin for GameCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameBuildSettings>();
        app.add_system(start_game.in_schedule(OnEnter(GameState::Playing)));
        app.add_system(update_game_state.in_set(OnUpdate(GameState::Playing)));

        app.add_system(check_game_ended.in_set(OnUpdate(GameState::Playing)));
        app.add_system(check_game_ended.in_set(OnUpdate(GameState::Paused)));

        app.add_system(draw_game.in_set(OnUpdate(GameState::Playing)));
        app.add_system(draw_game.in_set(OnUpdate(GameState::Paused)));
        app.add_system(draw_game.in_set(OnUpdate(GameState::Ended)));

        app.add_system(draw_game_over.in_set(OnUpdate(GameState::Ended)));

        app.add_system(
            simulate_game
                .in_schedule(CoreSchedule::FixedUpdate)
                .run_if(in_state(GameState::Playing)),
        );
        app.insert_resource(FixedTime::new_from_secs(0.01));
    }
}

pub const BORDER_PADDING_TOTAL: u32 = 20;

fn simulate_game(world: &mut World) {
    world.resource_scope(|mut world, mut game: Mut<Game>| {
        world.resource_scope(|world, mut game_runtime: Mut<GameRuntime<TestRunner>>| {
            game.game_world
                .resource_scope(|world, mut time: Mut<Time>| {
                    time.update();
                });
            game_runtime.game_runner.simulate_game(&mut game.game_world);
            world.resource_scope(|world, mut game_commands: Mut<GameCommands>| {
                game_commands.execute_buffer(&mut game.game_world);
            });
        });
    });
}

#[derive(Clone, Eq, Debug, PartialEq, Resource)]
pub struct GameBuildSettings {
    pub map_size: u32,
    pub enemy_count: usize,
}

impl GameBuildSettings {
    pub fn decrease_enemy_count(&mut self) {
        self.enemy_count = self.enemy_count.saturating_sub(1);
        if self.enemy_count < 1 {
            self.enemy_count = 1
        }
    }

    pub fn increase_enemy_count(&mut self) {
        self.enemy_count = self.enemy_count.saturating_add(1);
        if self.enemy_count > 3 {
            self.enemy_count = 3
        }
    }

    pub fn increase_map_size(&mut self, modifier: bool) {
        let mut amount_to_change = 1;
        if modifier {
            amount_to_change = 10
        }
        self.map_size = self.map_size.saturating_add(amount_to_change);

        if self.map_size > 100 {
            self.map_size = 100
        }
    }

    pub fn decrease_map_size(&mut self, modifier: bool) {
        let mut amount_to_change = 1;
        if modifier {
            amount_to_change = 10
        }
        self.map_size = self.map_size.saturating_sub(amount_to_change);

        if self.map_size < 30 {
            self.map_size = 30
        }
    }
}

impl Default for GameBuildSettings {
    fn default() -> Self {
        Self {
            map_size: 30,
            enemy_count: 1,
        }
    }
}

#[derive(Default, Clone, Eq, Debug, PartialEq, Resource, Reflect, FromReflect)]
pub struct GameData {
    pub map_size_x: u32,
    pub map_size_y: u32,
    pub object_classes: HashMap<String, ObjectClass>,
    pub object_groups: HashMap<String, ObjectGroup>,
    pub object_types: HashMap<String, ObjectType>,
    pub terrain_classes: HashMap<String, TerrainClass>,
    pub terrain_types: HashMap<String, TerrainType>,
    pub stacking_classes: HashMap<String, StackingClass>,
}

pub fn start_game(world: &mut World) {
    let mut game_data = GameData::default();
    let Some(game_build_settings) = world.remove_resource::<GameBuildSettings>() else{
        return;
    };

    let stacking_class_building: StackingClass = StackingClass {
        name: String::from("Building"),
    };

    let stacking_class_abilities: StackingClass = StackingClass {
        name: String::from("Ability"),
    };

    game_data.stacking_classes.insert(
        stacking_class_building.name.clone(),
        stacking_class_building.clone(),
    );
    game_data.stacking_classes.insert(
        stacking_class_abilities.name.clone(),
        stacking_class_abilities.clone(),
    );

    let terrain_classes: Vec<TerrainClass> = vec![
        TerrainClass {
            name: String::from("Colorable"),
        },
        TerrainClass {
            name: String::from("NonColorable"),
        },
    ];
    game_data
        .terrain_classes
        .insert(terrain_classes[0].name.clone(), terrain_classes[0].clone());
    game_data
        .terrain_classes
        .insert(terrain_classes[1].name.clone(), terrain_classes[1].clone());

    let terrain_types: Vec<TerrainType> = vec![
        TerrainType {
            name: String::from("BasicColorable"),
            terrain_class: terrain_classes[0].clone(),
        },
        TerrainType {
            name: String::from("BasicNonColorable"),
            terrain_class: terrain_classes[1].clone(),
        },
    ];
    game_data
        .terrain_types
        .insert(terrain_types[0].name.clone(), terrain_types[0].clone());
    game_data
        .terrain_types
        .insert(terrain_types[1].name.clone(), terrain_types[1].clone());

    let object_class_building: ObjectClass = ObjectClass {
        name: String::from("Building"),
    };
    let object_group_colorers: ObjectGroup = ObjectGroup {
        name: String::from("Colorers"),
        object_class: object_class_building.clone(),
    };
    let object_type_pulser: ObjectType = ObjectType {
        name: String::from("Pulser"),
        object_group: object_group_colorers.clone(),
    };
    let object_type_line: ObjectType = ObjectType {
        name: String::from("Line"),
        object_group: object_group_colorers.clone(),
    };
    let object_type_scatter: ObjectType = ObjectType {
        name: String::from("Scatter"),
        object_group: object_group_colorers.clone(),
    };
    game_data.object_classes.insert(
        object_class_building.name.clone(),
        object_class_building.clone(),
    );
    game_data.object_groups.insert(
        object_group_colorers.name.clone(),
        object_group_colorers.clone(),
    );
    game_data
        .object_types
        .insert(object_type_pulser.name.clone(), object_type_pulser.clone());
    game_data
        .object_types
        .insert(object_type_line.name.clone(), object_type_line.clone());
    game_data.object_types.insert(
        object_type_scatter.name.clone(),
        object_type_scatter.clone(),
    );
    let tile_stack_rules = TileObjectStacks::new(vec![
        (
            stacking_class_building.clone(),
            TileObjectStacksCount {
                current_count: 0,
                max_count: 1,
            },
        ),
        (
            stacking_class_abilities.clone(),
            TileObjectStacksCount {
                current_count: 0,
                max_count: 1,
            },
        ),
    ]);

    let tile_movement_costs = vec![(
        TerrainType {
            name: String::from("Grassland"),
            terrain_class: terrain_classes[0].clone(),
        },
        TileMovementCosts {
            movement_type_cost: Default::default(),
        },
    )];

    let mut game_commands = GameCommands::new();

    let map_size = TilemapSize {
        x: game_build_settings.map_size,
        y: game_build_settings.map_size,
    };
    game_data.map_size_x = map_size.x;
    game_data.map_size_y = map_size.y;

    let tilemap_tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let tilemap_type = TilemapType::Square;
    let mut commands: Vec<Box<dyn GameCommand>> = vec![];

    commands.push(Box::new(game_commands.spawn_testing_map(
        map_size,
        tilemap_type,
        tilemap_tile_size,
        terrain_types,
        tile_stack_rules,
    )) as Box<dyn GameCommand>);

    let inset_count = game_build_settings.map_size / 4;

    for player_id in 0..=game_build_settings.enemy_count {
        let player_spawn_pos = match player_id {
            0 => TilePos {
                x: inset_count,
                y: inset_count,
            },
            1 => TilePos {
                x: game_build_settings.map_size - inset_count,
                y: game_build_settings.map_size - inset_count,
            },
            2 => TilePos {
                x: game_build_settings.map_size - inset_count,
                y: inset_count,
            },
            _ => TilePos {
                x: inset_count,
                y: game_build_settings.map_size - inset_count,
            },
        };
        commands.push(Box::new(game_commands.spawn_object(
            (
                ObjectGridPosition {
                    tile_position: player_spawn_pos,
                },
                ObjectStackingClass {
                    stack_class: stacking_class_building.clone(),
                },
                Object,
                ObjectInfo {
                    object_type: object_type_pulser.clone(),
                },
                Building {
                    building_type: Pulser {
                        strength: 7,
                        max_pulse_tiles: 3,
                    },
                },
                BuildingCooldown {
                    timer: Timer::from_seconds(0.0, TimerMode::Once),
                    timer_reset: 0.2,
                },
                BuildingMarker::default(),
            ),
            player_spawn_pos,
            MapId { id: 1 },
            player_id,
        )) as Box<dyn GameCommand>);
    }

    setup_game(tile_movement_costs, Some(commands), world, game_data);

    let mut term: Mut<Terminal> = world.query::<&mut Terminal>().single_mut(world);
    term.resize([
        map_size.x + BORDER_PADDING_TOTAL,
        map_size.y + BORDER_PADDING_TOTAL,
    ]);
}

#[derive(Default)]
pub struct TestRunner {
    schedule: Schedule,
}
impl GameRunner for TestRunner {
    fn simulate_game(&mut self, world: &mut World) {
        self.schedule.run(world);
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
#[system_set(base)]
pub enum GameSets {
    Pre,
    Core,
    Post,
}

pub fn setup_game(
    tile_movement_costs: Vec<(TerrainType, TileMovementCosts)>,
    commands: Option<Vec<Box<dyn GameCommand>>>,
    world: &mut World,
    game_data: GameData,
) {
    let mut schedule = Schedule::new();
    schedule.configure_sets((GameSets::Pre, GameSets::Core, GameSets::Post).chain());
    schedule.add_systems(
        (
            apply_system_buffers,
            update_building_timers,
            apply_system_buffers,
            simulate_pulsers,
            simulate_lines,
            simulate_scatterers,
            update_color_conflicts,
            run_ai,
            handle_color_conflicts,
            destroy_buildings,
            apply_system_buffers,
        )
            .chain()
            .in_base_set(GameSets::Core),
    );
    schedule.add_systems(
        (
            apply_system_buffers,
            update_player_points,
            apply_system_buffers,
            update_game_end_state,
        )
            .chain()
            .in_base_set(GameSets::Post),
    );
    let mut game = match commands {
        None => GameBuilder::<TestRunner>::new_game(TestRunner { schedule }),
        Some(commands) => {
            GameBuilder::<TestRunner>::new_game_with_commands(commands, TestRunner { schedule })
        }
    };

    game.setup_movement(tile_movement_costs);
    game.setup_mapping();

    let (player_id, entity_mut) = game.add_player(true);
    let entity = entity_mut.id();
    game.game_world.entity_mut(entity).insert(PlayerPoints {
        building_points: 50,
        ability_points: 0,
    });
    world
        .spawn_empty()
        .insert(Actions::default())
        .insert(PlayerMarker::new(player_id));

    let (player_id, entity_mut) = game.add_player(false);
    let entity = entity_mut.id();
    game.game_world
        .entity_mut(entity)
        .insert(PlayerPoints {
            building_points: 50,
            ability_points: 0,
        })
        .insert(Actions::default());

    let (player_id, entity_mut) = game.add_player(false);
    let entity = entity_mut.id();
    game.game_world
        .entity_mut(entity)
        .insert(PlayerPoints {
            building_points: 50,
            ability_points: 0,
        })
        .insert(Actions::default());

    let (player_id, entity_mut) = game.add_player(false);
    let entity = entity_mut.id();
    game.game_world
        .entity_mut(entity)
        .insert(PlayerPoints {
            building_points: 50,
            ability_points: 0,
        })
        .insert(Actions::default());

    game.game_world.init_resource::<ColorConflicts>();
    game.game_world
        .init_resource::<Events<ColorConflictEvent>>();
    game.game_world.init_resource::<Time>();

    game.register_component::<ObjectInfo>();

    game.register_component::<Building<Pulser>>();
    game.register_component::<Activate>();
    game.register_component::<BuildingCooldown>();

    game.register_component::<TileColor>();
    game.register_resource::<ColorConflicts>();

    game.register_component::<PlayerPoints>();
    game.register_component::<Player>();
    game.register_component::<Actions>();

    world.insert_resource(game_data.clone());
    game.game_world.insert_resource(game_data);

    game.build(world);
}
