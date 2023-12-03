use std::io::Cursor;

use anyhow::Result;
use macroquad::{
    experimental::animation::{AnimatedSprite, Animation},
    prelude::*,
};
use tiled::{DefaultResourceCache, Loader, Map, Tileset};

/// Custom Reader so that we can read tiles in wasm.
struct TiledReader;
impl tiled::ResourceReader for TiledReader {
    type Resource = Cursor<&'static [u8]>;
    type Error = std::io::Error;

    fn read_from(
        &mut self,
        path: &std::path::Path,
    ) -> std::result::Result<Self::Resource, Self::Error> {
        if path == std::path::Path::new("data/world.tmx") {
            Ok(Cursor::new(include_bytes!("../data/world.tmx")))
        } else if path == std::path::Path::new("data/background_tiles.tsx") {
            Ok(Cursor::new(include_bytes!("../data/background_tiles.tsx")))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            ))
        }
    }
}

fn draw_background(world_map: &Map, tileset: &Tileset, texture: &Texture2D) {
    for layer in world_map.layers() {
        let layer = layer.as_tile_layer().unwrap();
        for x in 0..layer.width().unwrap() {
            for y in 0..layer.height().unwrap() {
                if let Some(tile) = layer.get_tile(x as i32, y as i32) {
                    let tile_id = tile.id();
                    let tile_width = tileset.tile_width;
                    let tile_height = tileset.tile_height;
                    let spacing = tileset.spacing;
                    let margin = tileset.margin;
                    let tiles_per_row =
                        (texture.size().x as u32 - margin + spacing) / (tile_width + spacing);
                    let tileset_texture_x = tile_id % tiles_per_row * tile_width;
                    let tileset_texture_y = tile_id / tiles_per_row * tile_height;

                    draw_texture_ex(
                        &texture,
                        (x * tile_width) as f32,
                        (y * tile_height) as f32,
                        WHITE,
                        DrawTextureParams {
                            flip_x: tile.flip_h,
                            flip_y: tile.flip_v,
                            source: Some(Rect::new(
                                tileset_texture_x as f32,
                                tileset_texture_y as f32,
                                tile_width as f32,
                                tile_height as f32,
                            )),
                            ..Default::default()
                        },
                    )
                }
            }
        }
    }
}

pub struct Character {
    pub name: String,
    pub job: String,
    pub position: Vec2,
    pub idle_sprite: AnimatedSprite,
    pub idle_texture: Texture2D,
    pub walk_sprite: AnimatedSprite,
    pub walk_texture: Texture2D,
}

impl Character {
    async fn new(
        name: &str,
        job: &str,
        position: Vec2,
        idle_animation: &str,
        walk_animation: &str,
    ) -> Result<Character> {
        let idle_sprite = AnimatedSprite::new(
            16,
            16,
            &[
                Animation {
                    name: "idle_down".to_string(),
                    row: 0,
                    frames: 4,
                    fps: 5,
                },
                Animation {
                    name: "idle_left".to_string(),
                    row: 1,
                    frames: 4,
                    fps: 5,
                },
                Animation {
                    name: "idle_up".to_string(),
                    row: 2,
                    frames: 4,
                    fps: 5,
                },
                Animation {
                    name: "idle_right".to_string(),
                    row: 3,
                    frames: 4,
                    fps: 5,
                },
            ],
            true,
        );
        let idle_texture = load_texture(idle_animation).await?;

        let walk_sprite = AnimatedSprite::new(
            16,
            16,
            &[
                Animation {
                    name: "walk_down".to_string(),
                    row: 0,
                    frames: 4,
                    fps: 5,
                },
                Animation {
                    name: "walk_left".to_string(),
                    row: 1,
                    frames: 4,
                    fps: 5,
                },
                Animation {
                    name: "walk_up".to_string(),
                    row: 2,
                    frames: 4,
                    fps: 5,
                },
                Animation {
                    name: "walk_right".to_string(),
                    row: 3,
                    frames: 4,
                    fps: 5,
                },
            ],
            true,
        );
        let walk_texture = load_texture(walk_animation).await?;

        Ok(Character {
            name: name.to_string(),
            job: job.to_string(),
            position,
            idle_sprite,
            idle_texture,
            walk_sprite,
            walk_texture,
        })
    }

    pub fn draw(&mut self) {
        draw_texture_ex(
            &self.idle_texture,
            self.position.x,
            self.position.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.idle_sprite.frame().dest_size),
                source: Some(self.idle_sprite.frame().source_rect),
                ..Default::default()
            },
        );

        self.idle_sprite.update();
    }
}

#[macroquad::main("llm-playground")]
async fn main() -> Result<()> {
    let mut loader = Loader::with_cache_and_reader(DefaultResourceCache::new(), TiledReader);
    let world_map = loader.load_tmx_map("data/world.tmx")?;

    let background_tileset = loader
        .load_tsx_tileset("data/background_tiles.tsx")
        .unwrap();

    let background_texture = load_texture("data/MasterSimple.png").await.unwrap();
    background_texture.set_filter(FilterMode::Nearest);

    // We want to be able to resize the window in such a way that the contents are always
    // aspect-preserved while always getting scaled in the best possible way.
    let map_width = (world_map.width * world_map.tile_width) as f32;
    let map_height = (world_map.height * world_map.tile_height) as f32;
    let render_target = render_target(map_width as u32, map_height as u32);
    render_target.texture.set_filter(FilterMode::Nearest);

    let mut render_target_camera =
        Camera2D::from_display_rect(Rect::new(0.0, 0.0, map_width, map_height));
    render_target_camera.render_target = Some(render_target.clone());

    let mut characters = vec![
        Character::new(
            "Kas",
            "Shopkeeper",
            vec2(100.0, 100.0),
            "data/char1_idle.png",
            "data/char1_walk.png",
        )
        .await?,
        Character::new(
            "Jeid",
            "Barkeeper",
            vec2(200.0, 200.0),
            "data/char2_idle.png",
            "data/char2_walk.png",
        )
        .await?,
        Character::new(
            "Bres",
            "Peasant",
            vec2(400.0, 230.0),
            "data/char3_idle.png",
            "data/char3_walk.png",
        )
        .await?,
    ];

    loop {
        set_camera(&render_target_camera);

        draw_background(&world_map, &background_tileset, &background_texture);

        // Draw characters.
        for character in &mut characters {
            character.draw();
        }

        // Draw FPS.
        draw_text(format!("FPS: {}", get_fps()).as_str(), 8., 16., 16., WHITE);

        set_default_camera();
        clear_background(BLACK);

        let zoom = f32::min(screen_width() / map_width, screen_height() / map_height);
        draw_texture_ex(
            &render_target.texture,
            (screen_width() - (map_width * zoom)) * 0.5,
            (screen_height() - (map_height * zoom)) * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(map_width * zoom, map_height * zoom)),
                flip_y: true, // Must flip y otherwise 'render_target' will be upside down
                ..Default::default()
            },
        );

        next_frame().await
    }
}
