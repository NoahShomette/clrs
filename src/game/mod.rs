pub mod end_game;
pub mod game_difficulty;
pub mod restart_game;
pub mod state;

use crate::abilities::expand::{simulate_expand_from_cache, Expand};
use crate::abilities::fortify::{simulate_fortify_from_cache, Fortify};
use crate::abilities::nuke::{simulate_nuke_from_cache, Nuke};
use crate::abilities::{destroy_abilities, update_ability_timers, Ability};
use crate::actions::Actions;
use crate::ai::{run_ai_ability, run_ai_building};
use crate::buildings::building_pathfinding::{SimplePathfindMap, SimplePathfinder};
use crate::buildings::line::{simulate_lines_from_cache, Line, LinePathfindMap};
use crate::buildings::pulser::{simulate_pulsers_from_cache, Pulser};
use crate::buildings::scatter::{simulate_scatter_from_cache, Scatter};
use crate::buildings::{
    destroy_buildings, update_building_timers, Activate, Building, BuildingCooldown,
    BuildingMarker, Simulate,
};
use crate::color_system::{
    handle_color_conflict_guarantees, handle_color_conflicts, update_color_conflicts,
    ColorConflictEvent, ColorConflictGuarantees, ColorConflicts, PlayerTileChangedCount, TileColor,
};
use crate::game::end_game::{check_game_ended, cleanup_game, update_game_end_state};
use crate::game::state::update_main_world_game_state;
use crate::level_loader::{LevelHandle, Levels};
use crate::mapping::map::MapCommandsExt;
use crate::objects::{
    delete_pathfind_object_from_tile_index_cache, simulate_simple_pathfind_object_cache,
    update_objects_index, ObjectIndex, TileToObjectIndex,
};
use crate::player::{update_player_points, PlayerPoints};
use crate::{GamePausedState, GameState};

use bevy::app::App;
use bevy::prelude::CoreSet::Update;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_asset_loader::loading_state::LoadingStateAppExt;
use bevy_ecs_tilemap::prelude::{TilePos, TilemapSize};
use bevy_ggf::game_core::command::{GameCommand, GameCommands};
use bevy_ggf::game_core::runner::{GameRunner, GameRuntime};
use bevy_ggf::game_core::{Game, GameBuilder};
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

use self::end_game::GameEndConditions;
use self::game_difficulty::GameDifficulty;
use self::restart_game::RestartGamePlugin;

pub struct GameCorePlugin;

impl Plugin for GameCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RestartGamePlugin);
        app.init_resource_after_loading_state::<_, GameBuildSettings>(GameState::Loading);
        app.add_system(start_game.in_schedule(OnEnter(GameState::Playing)))
            .add_system(cleanup_game.in_schedule(OnEnter(GameState::Menu)))
            .add_system(
                check_game_ended
                    .in_base_set(Update)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_system(
                simulate_game
                    .run_if(
                        in_state(GameState::Playing).and_then(in_state(GamePausedState::NotPaused)),
                    )
                    .in_schedule(CoreSchedule::FixedUpdate),
            )
            .add_systems(
                (update_main_world_game_state, apply_system_buffers)
                    .distributive_run_if(in_state(GameState::Playing))
                    .after(simulate_game)
                    .in_schedule(CoreSchedule::FixedUpdate),
            );

        app.insert_resource(FixedTime::new_from_secs(0.03));

        app.register_type::<GameBuildSettings>();
    }
}

pub fn setup_game_resource(mut world: &mut World) {
    let game_build_settings = GameBuildSettings::from_world(&mut world);
    world.insert_resource(game_build_settings);
}

pub fn simulate_game(world: &mut World) {
    world.resource_scope(|world, mut game: Mut<Game>| {
        world.resource_scope(|world, mut game_runtime: Mut<GameRuntime<TestRunner>>| {
            game.game_world
                .resource_scope(|_world, mut time: Mut<Time>| {
                    time.update();
                });
            game_runtime.simulate(&mut game.game_world);
            world.resource_scope(|_world, mut game_commands: Mut<GameCommands>| {
                game_commands.execute_buffer(&mut game.game_world);
            });
        });
    });
}

#[derive(Reflect, Clone, Debug, PartialEq, Resource)]
pub struct GameBuildSettings {
    pub map_size: u32,
    pub enemy_count: usize,
    pub map_type: usize,
    pub max_map: usize,
    pub level_sizes: LevelsSizes,
    pub game_end_conditions: GameEndConditions,
    pub game_difficulty: GameDifficulty,
}

#[derive(Reflect, Clone, Eq, Debug, PartialEq)]
pub struct LevelsSizes {
    pub lists: HashMap<usize, (u32, u32)>,
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
            self.map_size = 100;
        }

        #[cfg(target_arch = "wasm32")]
        if self.map_size > 60 {
            self.map_size = 60;
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

    pub fn next_map(&mut self) {
        self.map_type = self.map_type.saturating_add(1);

        if self.map_type > self.max_map - 1 {
            self.map_type = self.max_map - 1;
        }

        if self.map_type > 0 {
            self.map_size = self.level_sizes.lists[&self.map_type].0;
        }
    }

    pub fn prev_map(&mut self) {
        self.map_type = self.map_type.saturating_sub(1);
        if self.map_type > 0 {
            self.map_size = self.level_sizes.lists[&self.map_type].0;
        }
    }
}

impl FromWorld for GameBuildSettings {
    fn from_world(world: &mut World) -> Self {
        world.resource_scope(|world, maps: Mut<LevelHandle>| {
            world.resource_scope(|_world, assets: Mut<Assets<Levels>>| {
                let mut levels_sizes = LevelsSizes {
                    lists: Default::default(),
                };

                for (i, level) in assets.get(&maps.levels).unwrap().levels.iter().enumerate() {
                    if i > 0 {
                        levels_sizes
                            .lists
                            .insert(i, (level.tiles[0].len() as u32, level.tiles.len() as u32));
                    }
                }

                return Self {
                    map_size: 30,
                    enemy_count: 1,
                    map_type: 0,
                    max_map: assets.get(&maps.levels).unwrap().levels.len(),
                    level_sizes: levels_sizes,
                    game_end_conditions: GameEndConditions::Percentage {
                        target_percentage: 0.8,
                    },
                    game_difficulty: GameDifficulty::Medium,
                };
            })
        })
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
    // basically checks to see if we are back in menu or not to prevent multiple games forming
    let Some(game_build_settings) = world.remove_resource::<GameBuildSettings>() else {
        return;
    };

    let mut game_data = GameData::default();

    let level_data = world.resource_scope(|world, mut level_handles: Mut<LevelHandle>| {
        return world.resource_scope(|_world, mut level_assets: Mut<Assets<Levels>>| {
            level_assets.get(&level_handles.levels).unwrap().levels[game_build_settings.map_type]
                .clone()
        });
    });

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

    let object_type_nuke: ObjectType = ObjectType {
        name: String::from("Nuke"),
        object_group: object_group_colorers.clone(),
    };
    let object_type_fortify: ObjectType = ObjectType {
        name: String::from("Fortify"),
        object_group: object_group_colorers.clone(),
    };
    let object_type_expand: ObjectType = ObjectType {
        name: String::from("Expand"),
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

    game_data
        .object_types
        .insert(object_type_nuke.name.clone(), object_type_nuke.clone());
    game_data.object_types.insert(
        object_type_fortify.name.clone(),
        object_type_fortify.clone(),
    );
    game_data
        .object_types
        .insert(object_type_expand.name.clone(), object_type_expand.clone());

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

    let noncolorable_tile_stack_rules = TileObjectStacks::new(vec![
        (
            stacking_class_building.clone(),
            TileObjectStacksCount {
                current_count: 0,
                max_count: 0,
            },
        ),
        (
            stacking_class_abilities.clone(),
            TileObjectStacksCount {
                current_count: 0,
                max_count: 0,
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

    let mut commands: Vec<Box<dyn GameCommand>> = vec![];

    match game_build_settings.map_type {
        0 => {
            commands.push(Box::new(game_commands.spawn_random_map(
                map_size,
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
                let tile_position = player_spawn_pos.into();
                commands.push(Box::new(game_commands.spawn_object(
                    (
                        ObjectGridPosition {
                            tile_position: tile_position,
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
                                max_pulse_tiles: 10,
                            },
                        },
                        BuildingCooldown {
                            timer: Timer::from_seconds(0.0, TimerMode::Once),
                            timer_reset: 0.75,
                        },
                        BuildingMarker::default(),
                        Simulate,
                    ),
                    player_spawn_pos,
                    MapId { id: 1 },
                    player_id,
                )) as Box<dyn GameCommand>);
            }
        }
        _ => {
            commands.push(Box::new(game_commands.spawn_map(
                terrain_types,
                level_data.clone(),
                tile_stack_rules,
                noncolorable_tile_stack_rules,
            )) as Box<dyn GameCommand>);

            for player_id in 0..=game_build_settings.enemy_count {
                let player_spawn_pos = TilePos::new(
                    level_data.spawn_points[player_id].0 as u32,
                    level_data.spawn_points[player_id].1 as u32,
                );
                commands.push(Box::new(game_commands.spawn_object(
                    (
                        ObjectGridPosition {
                            tile_position: player_spawn_pos.into(),
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
                                max_pulse_tiles: 10,
                            },
                        },
                        BuildingCooldown {
                            timer: Timer::from_seconds(0.0, TimerMode::Once),
                            timer_reset: 0.75,
                        },
                        BuildingMarker::default(),
                        Simulate,
                    ),
                    player_spawn_pos,
                    MapId { id: 1 },
                    player_id,
                )) as Box<dyn GameCommand>);
            }
        }
    }

    setup_game(
        tile_movement_costs,
        Some(commands),
        world,
        game_data,
        game_build_settings,
    );
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
    game_build_settings: GameBuildSettings,
) {
    let mut schedule = Schedule::new();
    schedule.configure_sets((GameSets::Pre, GameSets::Core, GameSets::Post).chain());
    schedule.add_systems(
        (
            update_objects_index,
            apply_system_buffers,
            update_building_timers,
            update_ability_timers,
            apply_system_buffers,
            simulate_simple_pathfind_object_cache::<
                Building<Pulser>,
                SimplePathfinder<Building<Pulser>>,
                SimplePathfindMap<Building<Pulser>>,
            >,
            simulate_simple_pathfind_object_cache::<
                Building<Line>,
                SimplePathfinder<Building<Line>>,
                LinePathfindMap,
            >,
            simulate_simple_pathfind_object_cache::<
                Building<Scatter>,
                SimplePathfinder<Building<Scatter>>,
                SimplePathfindMap<Building<Scatter>>,
            >,
            simulate_simple_pathfind_object_cache::<
                Ability<Nuke>,
                SimplePathfinder<Ability<Nuke>>,
                SimplePathfindMap<Ability<Nuke>>,
            >,
            simulate_simple_pathfind_object_cache::<
                Ability<Expand>,
                SimplePathfinder<Ability<Expand>>,
                SimplePathfindMap<Ability<Expand>>,
            >,
            simulate_simple_pathfind_object_cache::<
                Ability<Fortify>,
                SimplePathfinder<Ability<Fortify>>,
                SimplePathfindMap<Ability<Fortify>>,
            >,
            apply_system_buffers,
            simulate_pulsers_from_cache,
        )
            .chain()
            .in_base_set(GameSets::Core),
    );
    schedule.add_systems(
        (
            simulate_lines_from_cache.after(simulate_pulsers_from_cache),
            simulate_scatter_from_cache,
            simulate_nuke_from_cache,
            simulate_expand_from_cache,
            simulate_fortify_from_cache,
            update_color_conflicts,
            run_ai_building,
            run_ai_ability,
            handle_color_conflict_guarantees,
            handle_color_conflicts,
            apply_system_buffers,
            destroy_buildings,
            destroy_abilities,
            apply_system_buffers,
        )
            .chain()
            .in_base_set(GameSets::Core),
    );
    schedule.add_systems(
        (
            delete_pathfind_object_from_tile_index_cache::<
                Building<Pulser>,
                SimplePathfinder<Building<Pulser>>,
                SimplePathfindMap<Building<Pulser>>,
            >,
            delete_pathfind_object_from_tile_index_cache::<
                Building<Line>,
                SimplePathfinder<Building<Line>>,
                LinePathfindMap,
            >,
            delete_pathfind_object_from_tile_index_cache::<
                Building<Scatter>,
                SimplePathfinder<Building<Scatter>>,
                SimplePathfindMap<Building<Scatter>>,
            >,
            delete_pathfind_object_from_tile_index_cache::<
                Ability<Nuke>,
                SimplePathfinder<Ability<Nuke>>,
                SimplePathfindMap<Ability<Nuke>>,
            >,
            delete_pathfind_object_from_tile_index_cache::<
                Ability<Expand>,
                SimplePathfinder<Ability<Expand>>,
                SimplePathfindMap<Ability<Expand>>,
            >,
            delete_pathfind_object_from_tile_index_cache::<
                Ability<Fortify>,
                SimplePathfinder<Ability<Fortify>>,
                SimplePathfindMap<Ability<Fortify>>,
            >,
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
    game.default_components_track_changes();
    game.add_default_registrations();
    game.setup_movement(tile_movement_costs);
    game.setup_mapping();

    for player_id in 0..=game_build_settings.enemy_count {
        if player_id == 0 {
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
        } else {
            let (player_id, entity_mut) = game.add_player(false);
            let entity = entity_mut.id();
            game.game_world
                .entity_mut(entity)
                .insert(PlayerPoints {
                    building_points: 50,
                    ability_points: 0,
                })
                .insert(Actions::default());
        }
    }

    game.game_world.init_resource::<ColorConflicts>();
    game.game_world
        .init_resource::<Events<ColorConflictEvent>>();
    game.game_world
        .init_resource::<Events<ColorConflictGuarantees>>();
    game.game_world.init_resource::<Time>();
    game.game_world.init_resource::<TileToObjectIndex>();
    game.game_world.init_resource::<ObjectIndex>();

    game.register_component::<Player>();
    game.register_component::<ObjectInfo>();

    game.register_component::<Building<Pulser>>();
    game.register_component::<Building<Line>>();
    game.register_component::<Building<Scatter>>();

    game.register_component::<Activate>();
    game.register_component::<BuildingCooldown>();

    game.register_component::<TileColor>();

    game.register_component::<PlayerPoints>();
    game.register_component::<Actions>();

    world.insert_resource(game_data.clone());
    game.game_world.insert_resource(game_data);
    game.game_world.insert_resource(game_build_settings.clone());
    game.game_world
        .insert_resource(PlayerTileChangedCount::default());
    world.insert_resource(game_build_settings);
    game.build(world);
}
