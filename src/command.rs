use solstice_2d::solstice::Context;
use solstice_2d::{
    solstice::image::Image, Color, Draw, Graphics, GraphicsLock, PerlinTextureSettings, Rectangle,
    RegularPolygon,
};

#[derive(Debug, Clone)]
pub enum Geometry {
    Rectangle(Rectangle),
    RegularPolygon(RegularPolygon),
}

impl Into<Geometry> for Rectangle {
    fn into(self) -> Geometry {
        Geometry::Rectangle(self)
    }
}

impl Into<Geometry> for RegularPolygon {
    fn into(self) -> Geometry {
        Geometry::RegularPolygon(self)
    }
}

#[derive(Debug, Clone)]
pub struct DrawCommand {
    geometry: Geometry,
    color: Color,
    texture: Option<PerlinTextureSettings>,
}

impl DrawCommand {
    pub fn new<G: Into<Geometry>>(geometry: G, color: Color) -> DrawCommand {
        Self {
            geometry: geometry.into(),
            color,
            texture: None,
        }
    }

    pub fn with_texture<G: Into<Geometry>>(
        geometry: G,
        color: Color,
        texture: PerlinTextureSettings,
    ) -> DrawCommand {
        Self {
            geometry: geometry.into(),
            color,
            texture: Some(texture),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ClearCommand {
    color: Color,
}

#[derive(Default, Debug)]
pub struct ResourcesCache {
    textures: std::collections::HashMap<PerlinTextureSettings, Image>,
}

impl ResourcesCache {
    pub fn warm(&mut self, commands: &[Command], ctx: &mut Context) {
        for command in commands {
            match command {
                Command::Draw(draw) => {
                    if let Some(settings) = draw.texture {
                        use std::collections::hash_map::Entry::Vacant;
                        if let Vacant(v) = self.textures.entry(settings) {
                            v.insert(solstice_2d::create_perlin_texture(ctx, settings).unwrap());
                        }
                    }
                }
                Command::Clear(_) => {}
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    Draw(DrawCommand),
    Clear(ClearCommand),
}

impl Command {
    pub fn batch_execute(
        ctx: &mut Context,
        ctx_2d: &mut Graphics,
        cache: &mut ResourcesCache,
        commands: &[Command],
    ) {
        cache.warm(&commands, ctx);
        let mut gfx = ctx_2d.lock(ctx);
        for command in commands {
            command.execute(&mut gfx, cache);
        }
    }

    pub fn execute(&self, gfx: &mut GraphicsLock, cache: &ResourcesCache) {
        match self {
            Command::Draw(command) => match &command.texture {
                None => match command.geometry {
                    Geometry::Rectangle(geometry) => gfx.draw_with_color(geometry, command.color),
                    Geometry::RegularPolygon(geometry) => {
                        gfx.draw_with_color(geometry, command.color)
                    }
                },
                Some(settings) => {
                    let texture = cache
                        .textures
                        .get(settings)
                        .expect("Cache should be warmed prior to execution.")
                        .clone();
                    match command.geometry {
                        Geometry::Rectangle(geometry) => {
                            gfx.image_with_color(geometry, texture, command.color)
                        }
                        Geometry::RegularPolygon(geometry) => {
                            gfx.image_with_color(geometry, texture, command.color)
                        }
                    }
                }
            },
            Command::Clear(command) => {
                gfx.clear(command.color);
            }
        }
    }
}
