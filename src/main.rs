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
    let background_layer = world_map
        .layers()
        .find(|l| l.name == "background")
        .unwrap()
        .as_tile_layer()
        .unwrap();
    for x in 0..background_layer.width().unwrap() {
        for y in 0..background_layer.height().unwrap() {
            if let Some(tile) = background_layer.get_tile(x as i32, y as i32) {
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

#[macroquad::main("llm-playground")]
async fn main() -> Result<()> {
    let mut loader = Loader::with_cache_and_reader(DefaultResourceCache::new(), TiledReader);
    let world_map = loader.load_tmx_map("data/world.tmx")?;

    let background_tileset = loader
        .load_tsx_tileset("data/background_tiles.tsx")
        .unwrap();

    let background_texture = load_texture("data/MasterSimple.png").await.unwrap();
    background_texture.set_filter(FilterMode::Nearest);

    let mut char1_idle_sprite = AnimatedSprite::new(
        16,
        16,
        &[Animation {
            name: "idle".to_string(),
            row: 0,
            frames: 4,
            fps: 5,
        }],
        true,
    );
    let char1_idle_texture = load_texture("data/char1_idle.png").await?;

    // We want to be able to resize the window in such a way that the contents are always
    // aspect-preserved while always getting scaled in the best possible way.
    let map_width = (world_map.width * world_map.tile_width) as f32;
    let map_height = (world_map.height * world_map.tile_height) as f32;
    let render_target = render_target(map_width as u32, map_height as u32);
    render_target.texture.set_filter(FilterMode::Nearest);

    let mut render_target_camera =
        Camera2D::from_display_rect(Rect::new(0.0, 0.0, map_width, map_height));
    render_target_camera.render_target = Some(render_target.clone());

    loop {
        set_camera(&render_target_camera);

        draw_background(&world_map, &background_tileset, &background_texture);

        // Draw characters.
        draw_texture_ex(
            &char1_idle_texture,
            100.0,
            100.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(char1_idle_sprite.frame().dest_size),
                source: Some(char1_idle_sprite.frame().source_rect),
                ..Default::default()
            },
        );

        char1_idle_sprite.update();

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
