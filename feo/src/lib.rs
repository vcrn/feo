/// A simple system resource monitoring CLI tool for Linux, with GPU monitoring
/// feature for Raspberry Pi
///
/// Author: github.com/vcrn
mod colors;

use colors::Colors;
use rounded_div::RoundedDiv;
use sinfo::{Memory, SystemInfo};
use std::{str, thread, time};
use termion::color::{Fg, Reset, Rgb};

/// Entry point into lib.rs
pub fn run(delay: usize, gpu: bool, color: char) -> Result<(), anyhow::Error> {
    let colors = Colors::new(color);

    let delay_usize = match delay {
        0 => 2,
        _ => delay,
    };

    let delay_u64 = delay_usize as u64;

    let mut mem_total = Memory::get_total()?;
    mem_total.save_with_unit();

    let mut cpu_time_prev = SystemInfo::new(gpu, SystemInfo::get_cpus()?)?.cpu_time; // Initial CPU time

    loop {
        let system_info = SystemInfo::new(gpu, cpu_time_prev.len())?;
        update_monitor(
            delay_usize,
            &system_info,
            &colors,
            cpu_time_prev,
            &mem_total,
        );
        cpu_time_prev = system_info.cpu_time;
        thread::sleep(time::Duration::from_secs(delay_u64));
    }
}

/// Updates the system information monitor, returns latest cpu_time
fn update_monitor(
    delay: usize,
    system_info: &SystemInfo,
    colors: &Colors,
    cpu_time_prev: Vec<usize>,
    mem_total: &Memory,
) {
    print!("{esc}c", esc = 27 as char); // Clears the terminal

    println!("{}", "-".repeat(30)); // Line

    print_comp_temp(colors.temp, system_info.cpu_temp, "CPU");

    // TODO: Rewrite more efficiently to not do a check at every update_monitor()
    if let Some(gpu_temp) = system_info.gpu_temp {
        print_comp_temp(colors.temp, gpu_temp, "GPU");
    }

    print_cpu_load(delay, colors.cpu, &system_info.cpu_time, cpu_time_prev);
    print_mem_use(colors.mem, &system_info.mem_free, mem_total);
    print_uptime(colors.uptime, system_info.uptime);

    println!("{}", "-".repeat(30)); // Line
}

/// Prints component temperature
fn print_comp_temp(color: Rgb, temp: f32, component: &str) {
    println!(
        "{}{} temp{}:{:>18.1}\u{00B0} C",
        Fg(color),
        component,
        Fg(Reset),
        temp
    );
}

/// Prints CPU load
fn print_cpu_load(delay: usize, color: Rgb, cpu_time: &[usize], cpu_time_prev: Vec<usize>) {
    for i in 0..cpu_time.len() {
        let cpu_load_percent = (cpu_time[i] - cpu_time_prev[i]) / delay;
        let cpu_load_bars = "|".repeat(cpu_load_percent.rounded_div(5)); // TODO: Add max limit, since cpu_load_percent can be > 100. match?
        println!(
            "{}CPU{}{}[{:<20}]{:>3}%",
            Fg(color),
            i + 1,
            Fg(Reset),
            cpu_load_bars,
            cpu_load_percent
        );
    }
}

/// Prints memory usage
fn print_mem_use(color: Rgb, mem_available: &Memory, mem_total: &Memory) {
    let ram_usage = Memory::format(mem_total.ram - mem_available.ram);
    let ram_fraction = match &mem_total.ram_with_unit {
        Some(ram_with_unit) => format!("{ram_usage}/{}", ram_with_unit),
        None => format!("{ram_usage}/{}", Memory::format(mem_total.ram)),
    };

    let swap_usage = Memory::format(mem_total.swap - mem_available.swap);
    let swap_fraction = match &mem_total.swap_with_unit {
        Some(swap_with_unit) => format!("{swap_usage}/{}", swap_with_unit),
        None => format!("{swap_usage}/{}", Memory::format(mem_total.swap)),
    };

    println!("{}RAM{}:{:>26}", Fg(color), Fg(Reset), ram_fraction);
    println!("{}Swap{}:{:>25}", Fg(color), Fg(Reset), swap_fraction);
}

/// Prints uptime since boot
fn print_uptime(color: Rgb, uptime_sec: f64) {
    let uptime_sec_int = uptime_sec as u64;
    let uptime_hr = uptime_sec_int / (60 * 60);
    let uptime_min = uptime_sec_int / (60) % (60);
    let uptime_sec_remain = uptime_sec_int % 60;

    println!(
        "{}Uptime{}: {:>16}:{:02}:{:02}",
        Fg(color),
        Fg(Reset),
        uptime_hr,
        uptime_min,
        uptime_sec_remain
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Fails on VM such as Github Actions, and clears screen
    fn test_update_monitor() {
        let colors = Colors::new('s');
        let gpu = false;
        let cpu_time_prev = SystemInfo::new(gpu, SystemInfo::get_cpus().unwrap())
            .unwrap()
            .cpu_time;
        let system_info = SystemInfo::new(gpu, SystemInfo::get_cpus().unwrap()).unwrap();
        let mut mem_total = Memory::get_total().unwrap();
        mem_total.save_with_unit();
        let delay = 3;

        update_monitor(delay, &system_info, &colors, cpu_time_prev, &mem_total)
    }
}
