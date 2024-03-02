//! Game project.
use fyrox::{
    core::{
        algebra::{Vector2, Vector3},
        pool::Handle,
        reflect::prelude::*,
        type_traits::prelude::*,
        visitor::{self, prelude::*},
    },
    event::{ElementState, Event, WindowEvent},
    gui::message::UiMessage,
    keyboard::{KeyCode, PhysicalKey},
    plugin::{Plugin, PluginConstructor, PluginContext, PluginRegistrationContext},
    scene::{
        animation::spritesheet::SpriteSheetAnimation,
        dim2::{rectangle::Rectangle, rigidbody::RigidBody},
        node::Node,
        Scene,
    },
    script::{ScriptContext, ScriptTrait},
};
use std::path::Path;

#[derive(Visit, Reflect, Debug, Clone, Default, TypeUuidProvider, ComponentProvider)]
#[type_uuid(id = "c5671d19-9f1a-4286-8486-add4ebaadaec")]
#[visit(optional)]
struct Player {
    sprite: Handle<Node>,
    move_left: bool,
    move_right: bool,
    jump: bool,
    animations: Vec<SpriteSheetAnimation>,
    current_animation: u32,
}

impl ScriptTrait for Player {
    // Called once at initialization
    fn on_init(&mut self, ctx: &mut ScriptContext) {}

    // Called when every other script is initialized
    fn on_start(&mut self, ctx: &mut ScriptContext) {}

    // Called whenever there is an event from the OS (mouse click, keyboard input, etc.)
    fn on_os_event(&mut self, event: &Event<()>, ctx: &mut ScriptContext) {
        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::KeyboardInput { event, .. } = event {
                if let PhysicalKey::Code(keycode) = event.physical_key {
                    let is_pressed = event.state == ElementState::Pressed;

                    match keycode {
                        KeyCode::KeyA => self.move_left = is_pressed,
                        KeyCode::KeyD => self.move_right = is_pressed,
                        KeyCode::Space => self.jump = is_pressed,
                        _ => (),
                    }
                }
            }
        }
    }

    // Called every frame as a fixed rate of 60 FPS
    fn on_update(&mut self, ctx: &mut ScriptContext) {
        if let Some(rigid_body) = ctx.scene.graph[ctx.handle].cast_mut::<RigidBody>() {
            let x_speed = if self.move_left {
                3.0
            } else if self.move_right {
                -3.0
            } else {
                0.0
            };

            if x_speed != 0.0 {
                self.current_animation = 0;
            } else {
                self.current_animation = 1;
            }

            if self.jump {
                rigid_body.set_lin_vel(Vector2::new(x_speed, 4.0))
            } else {
                rigid_body.set_lin_vel(Vector2::new(x_speed, rigid_body.lin_vel().y))
            };

            // Good practice to check if the handles are valid, cuz otherwise it might panic if the sprite is unassigned
            if let Some(sprite) = ctx.scene.graph.try_get_mut(self.sprite) {
                // Change player orientation only if moving
                if x_speed != 0.0 {
                    let local_transform = sprite.local_transform_mut();
                    let current_scale = **local_transform.scale();

                    local_transform.set_scale(Vector3::new(
                        current_scale.x.copysign(-x_speed),
                        current_scale.y,
                        current_scale.z,
                    ));
                }
            }

            if let Some(current_animation) = self.animations.get_mut(self.current_animation as usize) {
                current_animation.update(ctx.dt);

                if let Some(sprite) = ctx.scene.graph.try_get_mut(self.sprite).and_then(|n| n.cast_mut::<Rectangle>()) {
                    sprite.material().data_ref().set_texture(&"diffuseTexture".into(), current_animation.texture()).unwrap();
                    sprite.set_uv_rect(current_animation.current_frame_uv_rect().unwrap_or_default());
                }
            }
        }
    }
}

pub struct GameConstructor;

impl PluginConstructor for GameConstructor {
    fn register(&self, context: PluginRegistrationContext) {
        let script_contructors = &context.serialization_context.script_constructors;
        script_contructors.add::<Player>("Player");
    }

    fn create_instance(&self, scene_path: Option<&str>, context: PluginContext) -> Box<dyn Plugin> {
        Box::new(Game::new(scene_path, context))
    }
}

pub struct Game {
    scene: Handle<Scene>,
}

impl Game {
    pub fn new(scene_path: Option<&str>, context: PluginContext) -> Self {
        context
            .async_scene_loader
            .request(scene_path.unwrap_or("data/scene.rgs"));

        Self {
            scene: Handle::NONE,
        }
    }
}

impl Plugin for Game {
    fn on_deinit(&mut self, _context: PluginContext) {
        // Do a cleanup here.
    }

    fn update(&mut self, _context: &mut PluginContext) {
        // Add your global update code here.
    }

    fn on_os_event(&mut self, _event: &Event<()>, _context: PluginContext) {
        // Do something on OS event here.
    }

    fn on_ui_message(&mut self, _context: &mut PluginContext, _message: &UiMessage) {
        // Handle UI events here.
    }

    fn on_scene_begin_loading(&mut self, path: &Path, ctx: &mut PluginContext) {
        if self.scene.is_some() {
            ctx.scenes.remove(self.scene);
        }
    }

    fn on_scene_loaded(
        &mut self,
        path: &Path,
        scene: Handle<Scene>,
        data: &[u8],
        context: &mut PluginContext,
    ) {
        self.scene = scene;
    }
}
