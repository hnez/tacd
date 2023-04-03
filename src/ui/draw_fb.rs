// This file is part of tacd, the LXA TAC system daemon
// Copyright (C) 2022 Pengutronix e.K.
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

use std::io::Cursor;

use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use png::{BitDepth, ColorType, Encoder};

/*
#[cfg(feature = "demo_mode")]
mod backend {
    use framebuffer::{FixScreeninfo, VarScreeninfo};

    pub struct Framebuffer {
        pub device: (),
        pub var_screen_info: VarScreeninfo,
        pub fix_screen_info: FixScreeninfo,
        pub frame: [u8; 240 * 240 * 2],
    }

    impl Framebuffer {
        pub fn new(_: &str) -> Result<Self, ()> {
            Ok(Self {
                device: (),
                var_screen_info: VarScreeninfo {
                    bits_per_pixel: 16,
                    xres: 240,
                    yres: 240,
                    ..Default::default()
                },
                fix_screen_info: FixScreeninfo {
                    line_length: 480,
                    ..Default::default()
                },
                frame: [0; 240 * 240 * 2],
            })
        }

        pub fn put_var_screeninfo(_: &(), _: &VarScreeninfo) -> Result<(), ()> {
            Ok(())
        }
    }
}

#[cfg(not(feature = "demo_mode"))]
mod backend {
    pub use framebuffer::*;
}
*/

#[cfg(not(feature = "demo_mode"))]
mod backend {
    use std::fs::OpenOptions;
    use std::os::unix::io::{AsFd, BorrowedFd};

    use drm::buffer::DrmFourcc;
    use drm::control::connector;
    use drm::control::crtc;
    use drm::control::PageFlipFlags;
    use drm::control::dumbbuffer::DumbMapping;
    use drm::control::framebuffer;
    use drm::control::Device as ControlDevice;
    use drm::Device;
    use drm_ffi::drm_clip_rect;

    struct Card(std::fs::File);

    impl AsFd for Card {
        fn as_fd(&self) -> BorrowedFd<'_> {
            self.0.as_fd()
        }
    }

    impl Device for Card {}
    impl ControlDevice for Card {}

    impl Card {
        pub fn open(path: &str) -> Self {
            let mut options = OpenOptions::new();
            options.read(true);
            options.write(true);
            Card(options.open(path).unwrap())
        }
    }

    pub(super) struct Framebuffer {
        card: Card,
        pub(super) map: DumbMapping<'static>,
        pub(super) size: (u32, u32),
        fb: framebuffer::Handle,
        crtc: crtc::Handle,
    }

    impl Framebuffer {
        pub fn mark_dirty(
            &self,
            minx: u16,
            miny: u16,
            maxx: u16,
            maxy: u16,
        ) -> Result<(), drm::SystemError> {
            if minx >= maxx {
                println!("mark_dirty -- empty in x dir");
                return Ok(());
            }

            if miny >= maxy {
                println!("mark_dirty -- empty in y dir");
                return Ok(());
            }

            println!("{minx} {miny} {maxx} {maxy}");

            /*

            self.card.dirty_framebuffer(
                self.fb,
                &[drm_clip_rect {
                    x1: minx,
                    y1: miny,
                    x2: maxx,
                    y2: maxy,
                }],
            ).unwrap();
            */

            if let Err(e) = self.card.page_flip(self.crtc, self.fb, PageFlipFlags::EVENT, None) {
                println!("{e}");
            }

            Ok(())
        }

        pub fn new(card: &str) -> Result<Self, ()> {
            let card = Card::open(card);

            /*
                        card.set_client_capability(ClientCapability::Atomic, true)
                            .expect("Unable to request Atomic capability");
            */
            let res = card
                .resource_handles()
                .expect("Could not load normal resource ids.");

            let coninfo: Vec<connector::Info> = res
                .connectors()
                .iter()
                .flat_map(|con| card.get_connector(*con, true))
                .collect();

            let crtcinfo: Vec<crtc::Info> = res
                .crtcs()
                .iter()
                .flat_map(|crtc| card.get_crtc(*crtc))
                .collect();

            let con = coninfo
                .iter()
                .find(|&i| i.state() == connector::State::Connected)
                .expect("No connected connectors");

            let &mode = con.modes().get(0).expect("No modes found on connector");

            let crtc = crtcinfo.get(0).expect("No crtcs found");

            let size = {
                let (x, y) = mode.size();
                (x.into(), y.into())
            };

            let db = card
                .create_dumb_buffer(size, DrmFourcc::Rgb565, 16)
                .expect("Could not create dumb buffer");

            // What do we say to handling liftimes correctly?
            // Not today.
            // The DumbMapping contains a reference to DumbBuffer,
            // making the resulting struct a bit unwieldy.
            let db: &'static mut _ = Box::leak(Box::new(db));

            {
                let mut map = card.map_dumb_buffer(db).expect("Could not map dumbbuffer");

                for b in map.as_mut() {
                    *b = 128;
                }
            }

            let fb = card
                .add_framebuffer(db, 16, 16)
                .expect("Could not create FB");

            let crtc = crtc.handle();

            card.set_crtc(crtc, Some(fb), (0, 0), &[con.handle()], Some(mode))
                .expect("Could not set CRTC");

            let map = card.map_dumb_buffer(db).expect("Could not map dumbbuffer");

            Ok(Self {
                card,
                map,
                size,
                fb,
                crtc,
            })
        }
    }
}

use backend::Framebuffer;

pub struct FramebufferDrawTarget {
    fb: Framebuffer,
}

impl FramebufferDrawTarget {
    pub fn new() -> FramebufferDrawTarget {
        let fb = Framebuffer::new("/dev/dri/card0").unwrap();
        FramebufferDrawTarget { fb }
    }

    pub fn clear(&mut self) {
        self.fb.map.as_mut().iter_mut().for_each(|p| *p = 0x00);
    }

    pub fn as_png(&mut self) -> Vec<u8> {
        let mut dst = Cursor::new(Vec::new());

        let bpp = 2;
        let (xres, yres) = self.fb.size;
        let res = (xres as usize) * (yres as usize);

        let frame = self.fb.map.as_mut();

        let image: Vec<u8> = (0..res)
            .map(|i| if frame[i * bpp] != 0 { 0xff } else { 0 })
            .collect();

        let mut writer = {
            let mut enc = Encoder::new(&mut dst, xres, yres);
            enc.set_color(ColorType::Grayscale);
            enc.set_depth(BitDepth::Eight);
            enc.write_header().unwrap()
        };

        writer.write_image_data(&image).unwrap();
        writer.finish().unwrap();

        dst.into_inner()
    }
}

impl DrawTarget for FramebufferDrawTarget {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let bpp = 2;
        let (xres, yres) = self.fb.size;
        let line_length = xres * 2;

        let frame = self.fb.map.as_mut();

        let (mut minx, mut miny, mut maxx, mut maxy) = (xres, yres, 0, 0);

        for Pixel(coord, color) in pixels {
            let x = coord.x as u32;
            let y = coord.y as u32;

            if x >= xres || y >= yres {
                continue;
            }

            if x < minx {
                minx = x;
            }

            if y < miny {
                miny = y;
            }

            if x > maxx {
                maxx = x;
            }

            if y > maxy {
                maxy = y;
            }

            let offset = line_length * y + bpp * x;

            for b in 0..bpp {
                frame[(offset + b) as usize] = match color {
                    BinaryColor::Off => 0x00,
                    BinaryColor::On => 0xff,
                }
            }
        }

        self.fb
            .mark_dirty(minx as _, miny as _, maxx as _, maxy as _)
            .unwrap();

        Ok(())
    }
}

impl OriginDimensions for FramebufferDrawTarget {
    fn size(&self) -> Size {
        let (xres, yres) = self.fb.size;
        Size::new(xres, yres)
    }
}
