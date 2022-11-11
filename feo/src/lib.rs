/// A simple system resource monitoring CLI tool for Linux, especially on Raspberry Pi
/// Author: github.com/vcrn
mod colors;

use colors::Colors;
use rounded_div::RoundedDiv;
use sinfo::{Memory, SystemInfo};
use std::io::Error;
use std::process::Command;
use std::{fs, str, thread, time};
use termion::color::{Fg, Reset, Rgb};

/// Basic check for compability instead of designing all corresponding functions to handle errors well.
fn check_compability(check_gpu: bool) -> Result<(), Error> {
    Command::new("nproc").output()?; // Checking number of CPU cores
    fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")?; // Checking CPU temperature
    fs::read_to_string("/proc/stat")?; // Checking CPU time
    fs::read_to_string("/proc/meminfo")?; // Checking memory
    fs::read_to_string("/proc/uptime")?; // Checking uptime
    if check_gpu {
        Command::new("vcgencmd").arg("measure_temp").output()?; // Checking GPU temperature
    }
    Ok(())
}

pub fn run(delay: usize, gpu: bool, color: char) -> Result<(), Error> {
    check_compability(gpu)?; // TODO: add error message specifying that it failed since system is not Linux or can't read GPU temp

    let colors = Colors::new(color);

    let delay_usize = match delay {
        0 => 2,
        _ => delay,
    };

    let delay_u64 = delay_usize as u64;

    let mut mem_total = Memory::get_total();
    mem_total.save_with_unit();

    let mut cpu_time_prev = SystemInfo::new(gpu, SystemInfo::get_cpus()).cpu_time; // Initial CPU time

    loop {
        let system_info = SystemInfo::new(gpu, cpu_time_prev.len());
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
    let ram_fraction = format!("{ram_usage}/{}", mem_total.get_ram_with_unit());

    let swap_usage = Memory::format(mem_total.swap - mem_available.swap);
    let swap_fraction = format!("{swap_usage}/{}", mem_total.get_swap_with_unit());

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
        let cpu_time_prev = SystemInfo::new(gpu, SystemInfo::get_cpus()).cpu_time;
        let system_info = SystemInfo::new(gpu, SystemInfo::get_cpus());
        let mut mem_total = Memory::get_total();
        mem_total.save_with_unit();
        let delay = 3;

        update_monitor(delay, &system_info, &colors, cpu_time_prev, &mem_total)
    }
}
