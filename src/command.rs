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

#[derive(Debug, Copy, Clone)]
pub struct DefaultTexture;

#[derive(Debug, Clone)]
pub enum Texture {
    Noise(PerlinTextureSettings),
    Default(DefaultTexture),
}

impl Into<Texture> for PerlinTextureSettings {
    fn into(self) -> Texture {
        Texture::Noise(self)
    }
}

impl Into<Texture> for DefaultTexture {
    fn into(self) -> Texture {
        Texture::Default(self)
    }
}

#[derive(Debug, Clone)]
pub struct DrawCommand {
    geometry: Geometry,
    color: Color,
    texture: Texture,
}

impl DrawCommand {
    pub fn new<G: Into<Geometry>, T: Into<Texture>>(
        geometry: G,
        color: Color,
        texture: T,
    ) -> DrawCommand {
        Self {
            geometry: geometry.into(),
            color,
            texture: texture.into(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ClearCommand {
    color: Color,
}

impl ClearCommand {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
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
                    if let Texture::Noise(settings) = draw.texture {
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
                Texture::Default(_) => match command.geometry {
                    Geometry::Rectangle(geometry) => gfx.draw_with_color(geometry, command.color),
                    Geometry::RegularPolygon(geometry) => {
                        gfx.draw_with_color(geometry, command.color)
                    }
                },
                Texture::Noise(settings) => {
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
