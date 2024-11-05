use super::*;
use crate::{ColumnSample, Land};
use common::terrain::{Block, BlockKind, SpriteKind};
use rand::prelude::*;
use strum::{EnumIter, IntoEnumIterator};
use vek::*;
use Dir;

#[derive(EnumIter)]
enum Crop {
    Wildflower,
    Wheat,
    Flax,
    Corn,
    Tomato,
    Carrot,
    Radish,
    Turnip,
    Cabbage,
    Pumpkin,
    Sunflower,
    Cactus,
}

impl Crop {
    // None => no rows
    // Some((row width, row coverage proportion)) =>
    fn row_spacing(&self) -> Option<(f32, f32)> {
        match self {
            Self::Wildflower => None,
            // Grains
            Self::Wheat | Self::Flax | Self::Corn => Some((6.0, 0.8)),
            // Bushes
            Self::Tomato | Self::Cactus => Some((3.0, 1.0 / 3.0)),
            // Root & brassica
            Self::Carrot | Self::Radish | Self::Turnip | Self::Cabbage | Self::Pumpkin => {
                Some((6.0, 0.75))
            },
            Self::Sunflower => Some((4.0, 0.5)),
        }
    }

    fn sprites(&self) -> &[(f32, Option<SpriteKind>)] {
        match self {
            Self::Wheat => &[
                (4.0, Some(SpriteKind::Empty)),
                (1.0, Some(SpriteKind::WheatGreen)),
                (1.0, Some(SpriteKind::WheatYellow)),
            ],
            Self::Flax => &[
                (4.0, Some(SpriteKind::Empty)),
                (1.0, Some(SpriteKind::Flax)),
            ],
            Self::Corn => &[
                (4.0, Some(SpriteKind::Empty)),
                (1.0, Some(SpriteKind::Corn)),
            ],
            Self::Wildflower => &[
                (40.0, None),
                (1.0, Some(SpriteKind::BlueFlower)),
                (1.0, Some(SpriteKind::PinkFlower)),
                (1.0, Some(SpriteKind::PurpleFlower)),
                (0.1, Some(SpriteKind::RedFlower)),
                (1.0, Some(SpriteKind::WhiteFlower)),
                (1.0, Some(SpriteKind::YellowFlower)),
                (1.0, Some(SpriteKind::Sunflower)),
                (4.0, Some(SpriteKind::LongGrass)),
                (4.0, Some(SpriteKind::MediumGrass)),
                (4.0, Some(SpriteKind::ShortGrass)),
            ],
            Self::Tomato => &[
                (1.5, Some(SpriteKind::Empty)),
                (1.0, Some(SpriteKind::Tomato)),
            ],
            Self::Carrot => &[
                (5.0, Some(SpriteKind::Empty)),
                (1.0, Some(SpriteKind::Carrot)),
            ],
            Self::Radish => &[
                (5.0, Some(SpriteKind::Empty)),
                (1.0, Some(SpriteKind::Radish)),
            ],
            Self::Turnip => &[
                (5.0, Some(SpriteKind::Empty)),
                (1.0, Some(SpriteKind::Turnip)),
            ],
            Self::Cabbage => &[
                (5.0, Some(SpriteKind::Empty)),
                (1.0, Some(SpriteKind::Cabbage)),
            ],
            Self::Pumpkin => &[
                (5.0, Some(SpriteKind::Empty)),
                (1.0, Some(SpriteKind::Pumpkin)),
            ],
            Self::Sunflower => &[
                (5.0, Some(SpriteKind::Empty)),
                (1.0, Some(SpriteKind::Sunflower)),
            ],
            Self::Cactus => &[
                (10.0, Some(SpriteKind::Empty)),
                (1.0, Some(SpriteKind::BarrelCactus)),
                (1.0, Some(SpriteKind::RoundCactus)),
                (1.0, Some(SpriteKind::ShortCactus)),
                (1.0, Some(SpriteKind::MedFlatCactus)),
                (1.0, Some(SpriteKind::ShortFlatCactus)),
                (1.0, Some(SpriteKind::LargeCactus)),
                (1.0, Some(SpriteKind::TallCactus)),
            ],
        }
    }
}

/// Represents house data generated by the `generate()` method
pub struct FarmField {
    crop: Crop,
    /// Axis aligned bounding region for the house
    bounds: Aabr<i32>,
    /// Approximate altitude of the door tile
    pub(crate) alt: i32,
    ori: Vec2<f32>,
    is_desert: bool,
}

impl FarmField {
    pub fn generate(
        land: &Land,
        rng: &mut impl Rng,
        site: &Site,
        door_tile: Vec2<i32>,
        door_dir: Vec2<i32>,
        tile_aabr: Aabr<i32>,
        is_desert: bool,
    ) -> Self {
        let bounds = Aabr {
            min: site.tile_wpos(tile_aabr.min),
            max: site.tile_wpos(tile_aabr.max),
        };

        let ori = rng.gen_range(0.0..std::f32::consts::TAU);

        let crop = if is_desert {
            Crop::Cactus
        } else {
            Crop::iter()
                .filter(|crop| !matches!(crop, Crop::Cactus))
                .choose(rng)
                .unwrap()
        };

        Self {
            bounds,
            alt: land.get_alt_approx(site.tile_center_wpos(door_tile + door_dir)) as i32,
            ori: Vec2::new(ori.sin(), ori.cos()),
            crop,
            is_desert,
        }
    }
}

impl Structure for FarmField {
    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"render_farmfield\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "render_farmfield")]

    fn terrain_surface_at<R: Rng>(
        &self,
        wpos: Vec2<i32>,
        old: Block,
        rng: &mut R,
        col: &ColumnSample,
        z_off: i32,
    ) -> Option<Block> {
        let t = (self.ori * wpos.as_()).magnitude();
        let is_trench = self
            .crop
            .row_spacing()
            .map(|(w, p)| (t / w).fract() <= p)
            .unwrap_or(false);

        let hit_min_x_bounds = wpos.x == self.bounds.min.x;
        let hit_min_y_bounds = wpos.y == self.bounds.min.y;
        let hit_max_x_bounds = wpos.x == self.bounds.max.x - 1;
        let hit_max_y_bounds = wpos.y == self.bounds.max.y - 1;

        let is_bounds =
            hit_min_x_bounds || hit_min_y_bounds || hit_max_x_bounds || hit_max_y_bounds;

        let is_corner = (hit_max_y_bounds || hit_min_y_bounds)
            && (hit_max_x_bounds || hit_min_x_bounds)
            && is_bounds;

        let ori = if is_bounds && !is_corner {
            // for FenceI, can only go in the X or Y direction
            if hit_min_x_bounds || hit_max_x_bounds {
                Dir::Y
            } else {
                Dir::X
            }
        } else if is_bounds && is_corner {
            // for FenceL, can be rotated in 4 different directions
            if hit_min_x_bounds && hit_min_y_bounds {
                Dir::Y
            } else if hit_max_x_bounds && hit_min_y_bounds {
                Dir::X
            } else if hit_min_x_bounds && hit_max_y_bounds {
                Dir::NegY
            } else {
                Dir::NegX
            }
        } else {
            Dir::Y
        };

        if z_off == 0 {
            Some(Block::new(
                if self.is_desert {
                    BlockKind::Sand
                } else {
                    BlockKind::Grass
                },
                (Lerp::lerp(
                    col.surface_color,
                    col.sub_surface_color * 0.5,
                    is_trench as i32 as f32,
                ) * 255.0)
                    .as_(),
            ))
        } else if z_off == 1 && is_bounds {
            let sprite = if is_corner {
                SpriteKind::FenceL
            } else {
                SpriteKind::FenceI
            };
            let ori = if is_corner {
                match ori {
                    Dir::Y => 4,
                    Dir::X => 6,
                    Dir::NegY => 2,
                    Dir::NegX => 0,
                }
            } else {
                match ori {
                    Dir::Y => 2,
                    Dir::X => 0,
                    _ => 0,
                }
            };

            Some(old.into_vacant().with_sprite(sprite).with_ori(ori).unwrap())
        } else if z_off == 1 && (is_trench || self.crop.row_spacing().is_none()) {
            self.crop
                .sprites()
                .choose_weighted(rng, |(w, _)| *w)
                .ok()
                .and_then(|&(_, s)| Some(old.into_vacant().with_sprite(s?)))
        } else if z_off == 1 && rng.gen_bool(0.001) {
            Some(old.into_vacant().with_sprite(SpriteKind::Scarecrow))
        } else {
            None
        }
    }
}