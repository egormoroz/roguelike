use specs::prelude::*;
use crate::{
    util::DeltaTime,
    comp::*,
    util::Glyph,
};

#[derive(Default)]
pub struct ParticleSystem {
    buffer: Vec<Entity>,
}

impl ParticleSystem {
    pub fn update(&mut self, ecs: &mut World) {
        let dt = ecs.fetch::<DeltaTime>().0;
        for (e, lifetime) in 
            (&ecs.entities(), &mut ecs.write_storage::<ParticleLifetime>()).join() 
        {
            lifetime.remaining_ms -= dt;
            if lifetime.remaining_ms < 0. {
                self.buffer.push(e);
            }
        }
        ecs.delete_entities(&self.buffer)
            .expect("failed to delete particles");
        self.buffer.clear();
    }
}


#[derive(Default, Clone, Copy)]
pub struct ParticleRequest {
    pos: Position,
    r: Renderable,
    lifetime: ParticleLifetime,
}

#[derive(Default)]
pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>,
}

impl ParticleBuilder {
    pub fn request(&mut self, x: i32, y: i32, glyph: Glyph, fg: [f32; 4], bg: [f32; 4], lifetime_ms: f32) {
        self.requests.push(ParticleRequest {
            pos: Position { x, y },
            r: Renderable { glyph, fg, bg, order: 0 },
            lifetime: ParticleLifetime { remaining_ms: lifetime_ms },
        });
    }
}

pub struct ParticleSpawnSystem;

impl<'a> System<'a> for ParticleSpawnSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, ParticleBuilder>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, ParticleLifetime>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut builder, mut renderables, 
            mut positions, mut lifetimes) = data;
        for particle in builder.requests.iter() {
            entities.build_entity()
                .with(particle.pos, &mut positions)
                .with (particle.r, &mut renderables)
                .with (particle.lifetime, &mut lifetimes)
                .build();
        }
        builder.requests.clear();
    }
}
