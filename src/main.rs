use std::io::Cursor;

use anyhow::Result;
use macroquad::prelude::*;
use tiled::{DefaultResourceCache, Loader, Tile};

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

#[macroquad::main("llm-playground")]
async fn main() -> Result<()> {
    let mut loader = Loader::with_cache_and_reader(DefaultResourceCache::new(), TiledReader);
    let map = loader.load_tmx_map("data/world.tmx")?;

    let background_tileset = loader
        .load_tsx_tileset("data/background_tiles.tsx")
        .unwrap();

    let background_texture = load_texture("data/MasterSimple.png").await.unwrap();

    loop {
        clear_background(BLACK);

        // Draw the background
        let background_layer = map
            .layers()
            .find(|l| l.name == "background")
            .unwrap()
            .as_tile_layer()
            .unwrap();
        for x in 0..background_layer.width().unwrap() {
            for y in 0..background_layer.height().unwrap() {
                if let Some(tile) = background_layer.get_tile(x as i32, y as i32) {
                    let tile_id = tile.id();
                    let tile_width = background_tileset.tile_width;
                    let tile_height = background_tileset.tile_height;
                    let spacing = background_tileset.spacing;
                    let margin = background_tileset.margin;
                    let tiles_per_row = (background_texture.size().x as u32 - margin + spacing)
                        / (tile_width + spacing);
                    let tileset_texture_x = tile_id % tiles_per_row * tile_width;
                    let tileset_texture_y = tile_id / tiles_per_row * tile_height;

                    draw_texture_ex(
                        &background_texture,
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

        next_frame().await
    }
}
