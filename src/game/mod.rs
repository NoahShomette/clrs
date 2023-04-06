mod draw;
mod state;

use bevy::app::App;

use crate::buildings::pulser::simulate_pulsers;
use crate::buildings::{Building, Pulser};
use crate::color_system::TileColor;
use crate::game::draw::draw_game;
use crate::game::state::update_game_state;
use crate::map::MapCommandsExt;
use crate::GameState;
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
use bevy_ggf::movement::defaults::SquareMovementCalculator;
use bevy_ggf::movement::{GameBuilderMovementExt, TileMovementCosts};
use bevy_ggf::object::{
    Object, ObjectClass, ObjectGridPosition, ObjectGroup, ObjectInfo, ObjectType,
};

pub struct GameCorePlugin;
impl Plugin for GameCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(start_game.in_schedule(OnEnter(GameState::Playing)));
        app.add_system(update_game_state.in_set(OnUpdate(GameState::Playing)));
        app.add_system(draw_game.in_set(OnUpdate(GameState::Playing)));
        app.add_system(
            simulate_game
                .in_schedule(CoreSchedule::FixedUpdate)
                .run_if(in_state(GameState::Playing)),
        );
        app.insert_resource(FixedTime::new_from_secs(1.0));
    }
}

pub const BORDER_PADDING_TOTAL: u32 = 20;

fn simulate_game(world: &mut World) {
    world.resource_scope(|mut world, mut game: Mut<Game>| {
        world.resource_scope(|world, mut game_runtime: Mut<GameRuntime<TestRunner>>| {
            game_runtime.game_runner.simulate_game(&mut game.game_world);
            world.resource_scope(|world, mut game_commands: Mut<GameCommands>| {
                game_commands.execute_buffer(&mut game.game_world);
            });
        });
    });
}

#[derive(Default, Clone, Eq, Debug, PartialEq, Resource, Reflect, FromReflect)]
pub struct GameData {
    pub object_classes: HashMap<String, ObjectClass>,
    pub object_groups: HashMap<String, ObjectGroup>,
    pub object_types: HashMap<String, ObjectType>,
    pub terrain_classes: HashMap<String, TerrainClass>,
    pub terrain_types: HashMap<String, TerrainType>,
    pub stacking_classes: HashMap<String, StackingClass>,
}

pub fn start_game(world: &mut World) {
    let mut game_data = GameData::default();
    
    let stacking_class_building: StackingClass = StackingClass {
        name: String::from("Building"),
    };

    game_data.stacking_classes.insert(
        stacking_class_building.name.clone(),
        stacking_class_building.clone(),
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

    let tile_stack_rules = TileObjectStacks::new(vec![(
        stacking_class_building.clone(),
        TileObjectStacksCount {
            current_count: 0,
            max_count: 1,
        },
    )]);

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

    let map_size = TilemapSize { x: 30, y: 30 };
    let tilemap_tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let tilemap_type = TilemapType::Square;

    let spawn_map_command = game_commands.spawn_testing_map(
        map_size,
        tilemap_type,
        tilemap_tile_size,
        terrain_types,
        tile_stack_rules,
    );

    let player_spawn_pos = TilePos { x: 3, y: 3 };

    let spawn_object = game_commands.spawn_object(
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
                building_type: Pulser { strength: 5 },
            },
        ),
        player_spawn_pos,
        MapId { id: 1 },
        0,
    );

    let player_spawn_pos = TilePos { x: 15, y: 15 };

    let spawn_object_3 = game_commands.spawn_object(
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
                building_type: Pulser { strength: 5 },
            },
        ),
        player_spawn_pos,
        MapId { id: 1 },
        0,
    );

    let player_spawn_pos = TilePos { x: 5, y: 5 };

    let spawn_object_2 = game_commands.spawn_object(
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
                building_type: Pulser { strength: 5 },
            },
        ),
        player_spawn_pos,
        MapId { id: 1 },
        1,
    );

    setup_game(
        tile_movement_costs,
        Some(vec![
            Box::new(spawn_map_command) as Box<dyn GameCommand>,
            Box::new(spawn_object) as Box<dyn GameCommand>,
            Box::new(spawn_object_2) as Box<dyn GameCommand>,
            Box::new(spawn_object_3) as Box<dyn GameCommand>,
        ]),
        world,
    );

    world.insert_resource(game_data);

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

pub fn setup_game(
    tile_movement_costs: Vec<(TerrainType, TileMovementCosts)>,
    commands: Option<Vec<Box<dyn GameCommand>>>,
    world: &mut World,
) {
    let tilemap_type = TilemapType::Square;

    let mut schedule = Schedule::new();
    schedule.add_system(simulate_pulsers);

    let mut game = match commands {
        None => GameBuilder::<TestRunner>::new_game(TestRunner { schedule }),
        Some(commands) => {
            GameBuilder::<TestRunner>::new_game_with_commands(commands, TestRunner { schedule })
        }
    };

    game.setup_movement(tile_movement_costs);
    game.with_movement_calculator(
        SquareMovementCalculator {
            diagonal_movement: Default::default(),
        },
        vec![],
        tilemap_type,
    );
    game.setup_mapping();
    game.add_player(true);

    game.game_world.init_resource::<State<GameState>>();

    game.register_component::<ObjectInfo>();
    game.register_component::<Building<Pulser>>();
    game.register_component::<TileColor>();

    game.build(world);
}
