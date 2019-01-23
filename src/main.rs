//! Small amethyst demo app to illustrate sprite ordering
//!
//! Press Space to change the order of the sprites.

use amethyst::{
    Logger,
    GameDataBuilder,
    Application,
    SimpleState,
    StateData,
    GameData,
    StateEvent,
    SimpleTrans,
    Trans,
    core::{
        Transform,
        TransformBundle,
        SystemBundle,
    },
    prelude::{
        Config,
    },
    ecs::{
        DispatcherBuilder,
        World,
        Builder,
        Component,
        DenseVecStorage,
        System,
        ReadStorage,
        WriteStorage,
        Join,
    },
    assets::{
        Loader,
        Handle,
        AssetStorage,
    },
    renderer::{
        DisplayConfig,
        Pipeline,
        Stage,
        DrawFlat2D,
        ColorMask,
        ALPHA,
        DepthMode,
        RenderBundle,
        VirtualKeyCode,
        Texture,
        PngFormat,
        TextureMetadata,
        Camera,
        Projection,
        Transparent,
    },
    input::{
        is_close_requested,
        is_key_down,
    },
    utils::{
        application_root_dir,
    },
};

/// Main method
fn main() -> amethyst::Result<()> {
    // Disable logging cruft
    Logger::from_config(Default::default())
        .level_for("gfx_device_gl", amethyst::LogLevelFilter::Warn)
        .level_for("amethyst_renderer", amethyst::LogLevelFilter::Error)
        .start();

    // Set up configuration paths
    let application_root = application_root_dir()?;
    let display_config_path = application_root.join("resources/display_config.ron");
    let asset_path = application_root.join("assets");

    // Load display parameters
    let display_config = DisplayConfig::load(&display_config_path);

    // Build pipeline
    let pipe = Pipeline::build()
        .with_stage(Stage::with_backbuffer()
            .clear_target([0.1, 0.1, 0.1, 1.0], 1.0)
            .with_pass( DrawFlat2D::new()
                .with_transparency(
                    ColorMask::all(),
                    ALPHA,
                    Some(DepthMode::LessEqualWrite),
                )
            )
        );

    // Set up the necessary bundles
    // The sprite visibility sorting for the render bundle
    // is what's important here.
    let game_data_builder = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(RenderBundle::new(
            pipe, Some(display_config))
            .with_sprite_visibility_sorting(&[])
        )?
        .with_bundle(GameBundle)?
        ;

    // Build the game with our game state
    let mut game = Application::build(
        asset_path,
        GameState::default()
    )?.build(game_data_builder)?;

    // Run the game.
    game.run();


    Ok(())
}


// ===================================================================
// Game Components
#[derive(Default)]
/// Sprite Order component.
struct SpriteOrder {
    /// Controls which sprite is in front
    order: i32,
}
impl Component for SpriteOrder {
    type Storage = DenseVecStorage<Self>;
}
impl SpriteOrder {
    /// push the sprite towards the camera.
    /// if the order reaches max_order,
    /// this will push the sprite to the back.
    fn bump_order(&mut self, max_order: i32) {
        self.order = (self.order + 1) % max_order;
    }
    /// Registers the type with the given world
    fn register(world: &mut World) {
        world.register::<Self>();
    }
}
/// Registers all the required components
fn register_components(world: &mut World) {
    SpriteOrder::register(world);
}

// ===================================================================
// Game Resources
/// Adds resources to the world
fn add_resources(_world: &mut World) {
}

// ===================================================================
// Game Entities
/// Adds a background sprite (i.e. no sprite order)
fn add_background(world: &mut World) {
    let path = "sprites/background.png".to_string();
    let texture_handle = load_texture_handle(world, &path);
    world.create_entity()
        .with(Transform::default())
        .with(texture_handle.clone())
        .build();
}
/// Adds a sprite.
/// Key component here is Transparent,
/// which informs the sprite ordering system to use z position in displaying this sprite.
fn add_sprite(world: &mut World, name: &'static str, order: i32) {
    let path = format!("sprites/{}.png", name);
    let texture_handle = load_texture_handle(world, &path);
    let mut my_transform = Transform::default();
    my_transform.set_y((order as f32) * 20.0); // so the sprites don't block each other completely
    world.create_entity()
        .with(SpriteOrder{order})
        .with(my_transform)
        .with(texture_handle.clone())
        .with(Transparent) // We need to tell Amethyst that this sprite has some transparency element
        .build();
}
/// Adds a camera.
/// We set the camera far enough back so we have room in which to order our sprites.
fn add_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_z(100.0); // leave room for sprite ordering
    world.create_entity()
        .with(Camera::from(
            Projection::orthographic(
                -250.0, 250.0, -250.0, 250.0,
            )))
        .with(transform)
        .build();
}
/// Adds all the entities to our world.
fn add_entities(world: &mut World) {
    add_background(world);
    add_camera(world);
    add_sprite(world, "Character Cat Girl", 0);
    add_sprite(world, "Roof North", 1)
}

// ===================================================================
// Game Systems
/// A system which updates the z position of a sprite
/// based on the value of its order component
struct SpriteOrderSystem;
impl<'s>System<'s> for SpriteOrderSystem {
    type SystemData = (
       ReadStorage<'s, SpriteOrder>,
       WriteStorage<'s, Transform>,
    );
    // set the z position of the sprite based on the sprite's order #.
    fn run(&mut self, (sprite_order_set, mut transforms): Self::SystemData) {
        for (sprite_order, transform) in (&sprite_order_set, &mut transforms).join() {
            transform.set_z(sprite_order.order as f32);
        }
    }
}

// ===================================================================
// Game Bundle
struct GameBundle;
impl<'a, 'b>SystemBundle<'a, 'b> for GameBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> amethyst::Result<()> {
        builder.add(SpriteOrderSystem, "Sprite Order System", &[] );

        Ok(())
    }
}


// ===================================================================
// Game State
#[derive(Default)]
struct GameState;

impl SimpleState for GameState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        register_components(world);
        add_resources(world);
        add_entities(world);
    }
    /// Handles the usual quit event. Also,
    /// if the user presses the space bar,
    /// it swaps the order of the sprites.
    fn handle_event(&mut self, data: StateData<'_, GameData<'_, '_>>, event: StateEvent) -> SimpleTrans{
        if let  StateEvent::Window(event) = event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit
            } else if is_key_down(&event, VirtualKeyCode::Space) {
                let world = data.world;
                // Run through the sprite orders and bump them.
                // This is an example of running a "System" from within an event response.
                let mut sprite_order_set = world.write_storage::<SpriteOrder>();
                for sprite_order in (&mut sprite_order_set).join() {
                    sprite_order.bump_order(2);
                }
                return Trans::None
            }
        }
        Trans::None
    }
    fn update (&mut self, state_data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans{
        state_data.data.update(&state_data.world);
        Trans::None
    }
}

/// Utility to collect code that loads a sprite texture handle.
fn load_texture_handle(world: &mut World, path: &String) -> Handle<Texture> {
    let loader = world.read_resource::<Loader>();
    let texture_storage = world.read_resource::<AssetStorage<Texture>>();
    loader.load(
        path.as_ref(),
        PngFormat,
        TextureMetadata::srgb_scale(),
        (),
        &texture_storage
    )
}
