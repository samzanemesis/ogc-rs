//! The ``system`` module of ``ogc-rs``.
//!
//! This module implements a safe wrapper around the OS functions found in ``system.h``.

use crate::{ffi, video::RenderConfig, OgcError, Result};
use alloc::boxed::Box;
use core::{ffi::c_void, mem, ptr, time::Duration};
use num_enum::IntoPrimitive;

/// Represents the system service.
/// The initialization of this service is done in the crt0 startup code.
pub struct System;

/// OS Reset Types
#[derive(IntoPrimitive, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ResetTypes {
    Restart = 0,
    HotReset = 1,
    Shutdown = 2,
    ReturnToMenu = 3,
    PowerOff = 4,
    PowerOffStandby = 5,
    PowerOffIdle = 6,
}

/// OS Memory Protection Modes
#[derive(IntoPrimitive, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum MemoryProtectModes {
    ProtectNone = 0,
    ProtectRead = 1,
    ProtectWrite = 2,
    ProtectReadWrite = 1 | 2,
}

/// OS Memory Protection Channels
#[derive(IntoPrimitive, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum MemoryProtectChannels {
    ChannelZero = 0,
    ChannelOne = 1,
    ChannelTwo = 2,
    ChannelThree = 3,
    All = 4,
}

/// System Font Header Structure
pub struct FontHeader {
    pub font_type: u16,
    pub first_char: u16,
    pub last_char: u16,
    pub inval_char: u16,
    pub asc: u16,
    pub desc: u16,
    pub width: u16,
    pub leading: u16,
    pub cell_dimensions: (u16, u16),
    pub sheet_size: u32,
    pub sheet_format: u16,
    pub sheet_colrow: (u16, u16),
    pub sheet_dimensions: (u16, u16),
    pub width_table: u16,
    pub sheet_image: u32,
    pub sheet_fullsize: u32,
}

impl From<&mut FontHeader> for *mut ffi::sys_fontheader {
    fn from(head: &mut FontHeader) -> *mut ffi::sys_fontheader {
        Box::into_raw(Box::new(ffi::sys_fontheader {
            font_type: head.font_type,
            first_char: head.first_char,
            last_char: head.last_char,
            inval_char: head.inval_char,
            asc: head.asc,
            desc: head.desc,
            width: head.width,
            leading: head.leading,
            cell_width: head.cell_dimensions.0,
            cell_height: head.cell_dimensions.1,
            sheet_size: head.sheet_size,
            sheet_format: head.sheet_format,
            sheet_column: head.sheet_colrow.0,
            sheet_row: head.sheet_colrow.1,
            sheet_width: head.sheet_dimensions.0,
            sheet_height: head.sheet_dimensions.1,
            width_table: head.width_table,
            sheet_image: head.sheet_image,
            sheet_fullsize: head.sheet_fullsize,
            c0: 0,
            c1: 0,
            c2: 0,
            c3: 0,
        }))
    }
}

/// Implementation of the system service.
impl System {
    /// Allocate cacheline aligned memory for the external
    /// framebuffer based on the rendermode object.
    ///
    /// This function returns a pointer to the framebuffer's startaddress which
    /// is aligned to a 32 byte boundary.
    pub fn allocate_framebuffer(render_mode: &RenderConfig) -> *mut c_void {
        unsafe { ffi::SYS_AllocateFramebuffer(render_mode.into()) }
    }

    /// Create and initialize sysalarm structure.
    pub fn create_alarm(context: &mut u32) -> Result<()> {
        let r = unsafe { ffi::SYS_CreateAlarm(context) };

        if r < 0 {
            Err(OgcError::System("system failed to create alarm".into()))
        } else {
            Ok(())
        }
    }

    /// Cancel the alarm, but do not remove from the list of contexts.
    pub fn cancel_alarm(context: u32) -> Result<()> {
        let r = unsafe { ffi::SYS_CancelAlarm(context) };

        if r < 0 {
            Err(OgcError::System("system failed to cancel alarm".into()))
        } else {
            Ok(())
        }
    }

    /// Remove the given alarm context from the list of contexts and destroy it.
    pub fn remove_alarm(context: u32) -> Result<()> {
        let r = unsafe { ffi::SYS_RemoveAlarm(context) };

        if r < 0 {
            Err(OgcError::System("system failed to remove alarm".into()))
        } else {
            Ok(())
        }
    }

    /// Set the alarm parameters for a one-shot alarm, add to the list of alarms and start.
    pub fn set_alarm<F>(context: u32, fire_time: Duration, callback: Box<F>) -> Result<()>
    where
        F: Fn(u32, *mut c_void),
    {
        unsafe {
            // Convert Duration to timespec
            let timespec: *const ffi::timespec = &ffi::timespec {
                tv_sec: fire_time.as_secs() as i64,
                tv_nsec: fire_time.as_nanos() as i32,
            };

            // TODO: Check if this implementation can be changed.
            let ptr = Box::into_raw(callback);
            let code: extern "C" fn(alarm: u32, cb_arg: *mut c_void) = mem::transmute(ptr);
            let r = ffi::SYS_SetAlarm(context, timespec, Some(code), ptr::null_mut());

            if r < 0 {
                Err(OgcError::System("system failed to set alarm".into()))
            } else {
                Ok(())
            }
        }
    }

    /// Set the alarm parameters for a periodic alarm, add to the list of alarms and start.
    /// The alarm and interval persists as long as SYS_CancelAlarm() isn't called.
    pub fn set_periodic_alarm<F>(
        context: u32,
        time_start: Duration,
        time_period: Duration,
        callback: Box<F>,
    ) -> Result<()>
    where
        F: Fn(u32, *mut c_void),
    {
        unsafe {
            // Convert Duration to timespec
            let timespec_start: *const ffi::timespec = &ffi::timespec {
                tv_sec: time_start.as_secs() as i64,
                tv_nsec: time_start.as_nanos() as i32,
            };

            let timespec_period: *const ffi::timespec = &ffi::timespec {
                tv_sec: time_period.as_secs() as i64,
                tv_nsec: time_period.as_nanos() as i32,
            };

            // TODO: Check if this implementation can be changed.
            let ptr = Box::into_raw(callback);
            let code: extern "C" fn(alarm: u32, cb_arg: *mut c_void) = mem::transmute(ptr);
            let r = ffi::SYS_SetPeriodicAlarm(
                context,
                timespec_start,
                timespec_period,
                Some(code),
                ptr::null_mut(),
            );

            if r < 0 {
                Err(OgcError::System(
                    "system failed to set periodic alarm".into(),
                ))
            } else {
                Ok(())
            }
        }
    }

    /// Init Font
    pub fn init_font(font_header: &mut FontHeader) {
        unsafe {
            let _ = ffi::SYS_InitFont(font_header.into());
        }
    }

    /// Get Font Texel
    ///
    /// # Safety
    ///
    /// The user must ensure the pointer is valid
    pub unsafe fn get_font_texel(
        c: i32,
        image: *mut c_void,
        position: i32,
        stride: i32,
        width: &mut i32,
    ) {
        ffi::SYS_GetFontTexel(c, image, position, stride, width);
    }

    /// Get Font Texture
    ///
    /// # Safety
    ///
    /// The user must ensure the pointer is valid
    pub unsafe fn get_font_texture(
        c: i32,
        image: *mut *mut c_void,
        xpos: &mut i32,
        ypos: &mut i32,
        width: &mut i32,
    ) {
        ffi::SYS_GetFontTexture(c, image, xpos, ypos, width);
    }

    /// Get Font Encoding
    pub fn get_font_encoding() -> u32 {
        unsafe { ffi::SYS_GetFontEncoding() }
    }

    /// Set Wireless ID
    pub fn set_wireless_id(channel: u32, id: u16) {
        unsafe {
            ffi::SYS_SetWirelessID(channel, id);
        }
    }

    /// Get Wireless ID
    pub fn get_wireless_id(channel: u32) -> u16 {
        unsafe { ffi::SYS_GetWirelessID(channel) }
    }

    /// Start PMC
    pub fn start_pmc(mcr0: u32, mcr1: u32) {
        unsafe {
            ffi::SYS_StartPMC(mcr0, mcr1);
        }
    }

    /// Dump PMC
    pub fn dump_pmc() {
        unsafe {
            ffi::SYS_DumpPMC();
        }
    }

    /// Stop PMC
    pub fn stop_pmc() {
        unsafe {
            ffi::SYS_StopPMC();
        }
    }

    /// Reset PMC
    pub fn reset_pmc() {
        unsafe {
            ffi::SYS_ResetPMC();
        }
    }

    /// Reset System
    pub fn reset_system(reset: i32, reset_type: ResetTypes, force_menu: i32) {
        unsafe {
            ffi::SYS_ResetSystem(reset, reset_type.into(), force_menu);
        }
    }

    /// Protect Range
    pub fn protect_range(
        channel: MemoryProtectChannels,
        address: u32,
        bytes: u32,
        control: MemoryProtectModes,
    ) {
        unsafe {
            ffi::SYS_ProtectRange(
                channel.into(),
                address as *mut c_void,
                bytes,
                control.into(),
            );
        }
    }

    /// Switch Fiber
    pub fn switch_fiber(arg0: u32, arg1: u32, arg2: u32, arg3: u32, pc: u32, newsp: u32) {
        unsafe {
            ffi::SYS_SwitchFiber(arg0, arg1, arg2, arg3, pc, newsp);
        }
    }

    /// Get system time.
    pub fn system_time() -> u64 {
        unsafe { ffi::SYS_Time() }
    }
}
