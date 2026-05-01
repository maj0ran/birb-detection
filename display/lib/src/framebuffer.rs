extern crate libc;
use crate::math::UVec;
use libc::ioctl;

use std::{
    fs::{File, OpenOptions},
    io,
    os::fd::AsRawFd,
    path::Path,
    ptr,
};

const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;
const FBIOGET_FSCREENINFO: libc::c_ulong = 0x4602;

// all waveform modes for documentation even though we don't use them
#[allow(unused)]
enum WaveformMode {
    Init = 0,
    Du = 1,
    Gc16 = 2,
    Gc4 = 3,
    A2 = 4,
    Gl16 = 5,
    Glr16 = 6,
    Auto = 0x101,
}

enum UpdateMode {
    Partial = 0x0,
    Full = 0x1,
}

const TEMP_USE_AMBIENT: libc::c_int = 0x1000;

#[allow(unused)]
const EPDC_FLAG_ENABLE_INVERSION: libc::c_uint = 0x01;
#[allow(unused)]
const EPDC_FLAG_FORCE_MONOCHROME: libc::c_uint = 0x02;

// these ioctl IDs seem to be specific for this platform (Kobo Aura One)
// usually you find the IDs 0x2E and 0x2F for these ioctls but here we have some custom version of
// these?
// Someone on the internet reverse engineered this, I just stole it
const MXCFB_SEND_UPDATE: libc::c_ulong = 0x4044462E;
const MXCFB_WAIT_FOR_UPDATE_COMPLETE: libc::c_ulong = 0x4004462F;

#[allow(unused)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Mode {
    Fast,
    Partial,
    Gui,
    Full,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct FixScreenInfo {
    pub id: [u8; 16],
    pub smem_start: usize,
    pub smem_len: u32,
    pub kind: u32,
    pub type_aux: u32,
    pub visual: u32,
    pub xpanstep: u16,
    pub ypanstep: u16,
    pub ywrapstep: u16,
    pub line_length: u32,
    pub mmio_start: usize,
    pub mmio_len: u32,
    pub accel: u32,
    pub capabilities: u16,
    pub reserved: [u16; 2],
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct VarScreenInfo {
    pub xres: u32,
    pub yres: u32,
    pub xres_virtual: u32,
    pub yres_virtual: u32,
    pub xoffset: u32,
    pub yoffset: u32,
    pub bits_per_pixel: u32,
    pub grayscale: u32,
    pub red: Bitfield,
    pub green: Bitfield,
    pub blue: Bitfield,
    pub transp: Bitfield,
    pub nonstd: u32,
    pub activate: u32,
    pub height: u32,
    pub width: u32,
    pub accel_flags: u32,
    pub pixclock: u32,
    pub left_margin: u32,
    pub right_margin: u32,
    pub upper_margin: u32,
    pub lower_margin: u32,
    pub hsync_len: u32,
    pub vsync_len: u32,
    pub sync: u32,
    pub vmode: u32,
    pub rotate: u32,
    pub colorspace: u32,
    pub reserved: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Bitfield {
    pub offset: u32,
    pub length: u32,
    pub msb_right: u32,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct MxcfbRect {
    pub top: u32,
    pub left: u32,
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Clone, Debug)]
struct MxcfbAltBufferData {
    virt_addr: *const libc::c_void,
    phys_addr: u32,
    width: u32,
    height: u32,
    alt_update_region: MxcfbRect,
}

#[repr(C)]
#[derive(Clone, Debug)]
struct MxcfbUpdateData {
    update_region: MxcfbRect,
    waveform_mode: u32,
    update_mode: u32,
    update_marker: u32,
    temp: libc::c_int,
    flags: libc::c_uint,
    alt_buffer_data: MxcfbAltBufferData,
}

impl ::std::default::Default for Bitfield {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}
impl ::std::default::Default for FixScreenInfo {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

impl ::std::default::Default for VarScreenInfo {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

pub struct Framebuffer {
    device: File,
    frame: *mut libc::c_void,
    frame_size: libc::size_t,
    token: u32,
    flags: u32,
    pub bytes_per_pixel: u8,
    pub var_info: VarScreenInfo,
    pub fix_info: FixScreenInfo,
}

pub fn fix_screen_info(device: &File) -> io::Result<FixScreenInfo> {
    let mut info: FixScreenInfo = Default::default();
    let result = unsafe { ioctl(device.as_raw_fd(), FBIOGET_FSCREENINFO, &mut info) };
    match result {
        -1 => Err(io::Error::last_os_error()),
        _ => Ok(info),
    }
}

pub fn var_screen_info(device: &File) -> io::Result<VarScreenInfo> {
    let mut info: VarScreenInfo = Default::default();
    let result = unsafe { ioctl(device.as_raw_fd(), FBIOGET_VSCREENINFO, &mut info) };
    match result {
        -1 => Err(io::Error::last_os_error()),
        _ => Ok(info),
    }
}

impl Framebuffer {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Framebuffer> {
        let device = OpenOptions::new().read(true).write(true).open(path)?;

        let var_info = var_screen_info(&device)?;
        let fix_info = fix_screen_info(&device)?;

        assert_eq!(var_info.bits_per_pixel % 8, 0);

        let bytes_per_pixel = var_info.bits_per_pixel / 8;

        let mut frame_size =
            (var_info.xres_virtual * var_info.yres_virtual * bytes_per_pixel) as libc::size_t;

        if frame_size > fix_info.smem_len as usize {
            frame_size = fix_info.smem_len as usize;
        }

        assert!(frame_size as u32 >= var_info.yres * fix_info.line_length);

        let frame = unsafe {
            libc::mmap(
                ptr::null_mut(),
                frame_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                device.as_raw_fd(),
                0,
            )
        };

        if frame == libc::MAP_FAILED {
            Err(io::Error::last_os_error())
        } else {
            Ok(Framebuffer {
                device,
                frame,
                frame_size,
                token: 1,
                flags: 0,
                bytes_per_pixel: bytes_per_pixel as u8,
                var_info,
                fix_info,
            })
        }
    }

    pub fn set_pixel(&mut self, p: UVec, rgb: [u8; 3]) {
        let addr = (self.var_info.xoffset as isize + p.x as isize)
            * (self.bytes_per_pixel as isize)
            + (self.var_info.yoffset as isize + p.y as isize)
                * (self.fix_info.line_length as isize);

        assert!(addr < self.frame_size as isize);

        unsafe {
            let pixel = self.frame.offset(addr) as *mut u8;
            *pixel.offset(2) = rgb[0];
            *pixel.offset(1) = rgb[1];
            *pixel.offset(0) = rgb[2];
            *pixel.offset(3) = 0;
        }
    }

    pub fn update<T: Into<MxcfbRect>>(&mut self, rect: T, mode: Mode) -> io::Result<u32> {
        let (update_mode, waveform_mode) = match mode {
            Mode::Fast => (UpdateMode::Partial, WaveformMode::A2),
            Mode::Partial => (UpdateMode::Partial, WaveformMode::Auto),
            Mode::Gui => (UpdateMode::Full, WaveformMode::Auto),
            Mode::Full => (UpdateMode::Full, WaveformMode::Gc16),
        };
        let alt_buffer_data = MxcfbAltBufferData {
            virt_addr: ptr::null(),
            phys_addr: 0,
            width: 0,
            height: 0,
            alt_update_region: MxcfbRect {
                top: 0,
                left: 0,
                width: 0,
                height: 0,
            },
        };

        let update_marker = self.token;
        let update_data = MxcfbUpdateData {
            update_region: rect.into(),
            waveform_mode: waveform_mode as u32,
            update_mode: update_mode as u32,
            update_marker,
            temp: TEMP_USE_AMBIENT,
            flags: self.flags,
            alt_buffer_data,
        };
        let result =
            unsafe { libc::ioctl(self.device.as_raw_fd(), MXCFB_SEND_UPDATE, &update_data) };
        match result {
            -1 => Err(io::Error::last_os_error()),
            _ => {
                self.token = self.token.wrapping_add(1);
                Ok(update_marker)
            }
        }
    }

    pub fn wait(&mut self) -> io::Result<i32> {
        // ??? token reqauired for ioctl, usage unclear
        let token = 1;
        let result = unsafe {
            libc::ioctl(
                self.device.as_raw_fd(),
                MXCFB_WAIT_FOR_UPDATE_COMPLETE,
                &token,
            )
        };
        match result {
            -1 => Err(io::Error::last_os_error()),
            _ => Ok(result as i32),
        }
    }
}
