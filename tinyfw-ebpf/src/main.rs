#![no_std]
#![no_main]


use core::mem;
use memoffset::offset_of;
use aya_bpf::{
    bindings::xdp_action,
    macros::xdp,
    programs::XdpContext,
};

mod bindings;
use bindings::{ethhdr, iphdr, tcphdr};

#[xdp(name="tinyfw")]
pub fn tinyfw(ctx: XdpContext) -> u32 {
    match try_tinyfw(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

#[inline(always)]
unsafe fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return Err(());
    }

    Ok((start + offset) as *const T)
}

fn try_tinyfw(ctx: XdpContext) -> Result<u32, ()> {
    let h_proto = u16::from_be(unsafe {
        *ptr_at(&ctx, offset_of!(ethhdr, h_proto))?
    });

    if h_proto != ETH_P_IP {
        return Ok(xdp_action::XDP_PASS);
    }

    let ip_proto = u8::from_be(unsafe {
        *ptr_at(&ctx, ETH_HDR_LEN + offset_of!(iphdr, protocol))?
    });

    if ip_proto != IP_P_TCP {
        return Ok(xdp_action::XDP_PASS);
    }

    let d_port = u16::from_be(
        unsafe {
            *ptr_at(&ctx, ETH_HDR_LEN + IP_HDR_LEN + offset_of!(tcphdr, dest))?
        }
    );

    if DISALLOWED_DST_PORTS.contains(&d_port) {
        return Ok(xdp_action::XDP_DROP);
    }

    Ok(xdp_action::XDP_PASS)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}

const ETH_P_IP: u16 = 0x0800;
const ETH_HDR_LEN: usize = mem::size_of::<ethhdr>();
const IP_HDR_LEN: usize = mem::size_of::<iphdr>();
const IP_P_TCP: u8 = 0x06;
// See: https://repository.root-me.org/R%C3%A9seau/EN%20-%20Clear%20Text%20Protocols.pdf
const DISALLOWED_DST_PORTS: &'static [u16] = &[80, 20, 21,23,25, 110,143,139,445,1521,161,162, 70];
