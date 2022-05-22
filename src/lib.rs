/// A simple system resource monitoring CLI tool for Linux, especially on Raspberry Pi
/// Author: github.com/vcrn
use rounded_div::RoundedDiv;
use std::io::Error;
use std::process::Command;
use std::{fs, str, thread, time};
use termion::color::{Fg, Reset, Rgb};

#[cfg(test)]
#[path = "../tests/unit_tests/test_lib.rs"]
mod test_lib;

/// Colors to print parts of the monitor in
struct Colors {
    temp: Rgb,
    cpu: Rgb,
    mem: Rgb,
    uptime: Rgb,
}

impl Colors {
    fn new(color: char) -> Colors {
        match color {
            'w' => Colors {
                temp: Rgb(255, 255, 255),
                cpu: Rgb(255, 255, 255),
                mem: Rgb(255, 255, 255),
                uptime: Rgb(255, 255, 255),
            },
            'b' => Colors {
                temp: Rgb(0, 0, 0),
                cpu: Rgb(0, 0, 0),
                mem: Rgb(0, 0, 0),
                uptime: Rgb(0, 0, 0),
            },
            _ => Colors {
                // Standard color theme.
                temp: Rgb(255, 255, 0),
                cpu: Rgb(0, 220, 0),
                mem: Rgb(255, 0, 255),
                uptime: Rgb(0, 230, 230),
            },
        }
    }
}

/// Used for storing total and available memory
struct Memory {
    ram: f32,
    swap: f32,
    ram_with_unit: Option<String>,
    swap_with_unit: Option<String>,
}

impl Memory {
    // Creates an instance storing memory matching the search word arguments.
    fn new(search_word_ram: &str, search_word_swap: &str) -> Memory {
        let meminfo_content = fs::read_to_string("/proc/meminfo").expect("Could not read meminfo");

        let mut meminfo_split: Vec<&str> = meminfo_content.split(&[':', ' ', '\n'][..]).collect();

        meminfo_split.retain(|&x| !x.is_empty()); // Removes empty elements.
        let index_ram_search = meminfo_split
            .iter()
            .position(|&r| r == search_word_ram)
            .unwrap_or_else(|| panic!("Couldn't find position of {}", search_word_ram));

        let index_swap_search = meminfo_split
            .iter()
            .position(|&r| r == search_word_swap)
            .unwrap_or_else(|| panic!("Couldn't find position of {}", search_word_swap));

        let ram_kib = meminfo_split[index_ram_search + 1]
            .parse::<f32>()
            .expect("Failed to convert ram_kib to f32");

        let swap_kib = meminfo_split[index_swap_search + 1]
            .parse::<f32>()
            .expect("Failed to convert swap_kib to f32");

        Memory {
            ram: ram_kib,
            swap: swap_kib,
            ram_with_unit: None,
            swap_with_unit: None,
        }
    }

    // Returns available RAM and free swap
    fn get_free() -> Memory {
        Self::new("MemAvailable", "SwapFree")
    }

    // Returns Memory instance storing total RAM and total swap
    fn get_total() -> Memory {
        Self::new("MemTotal", "SwapTotal")
    }

    // Returns argument, given in kb, formatted to suitable unit.
    fn format(mem_in_kb: f32) -> String {
        if mem_in_kb > 1000000. {
            format!("{:.2}Gi", mem_in_kb / 1048576.)
        } else if mem_in_kb > 1000. {
            format!("{:.2}Mi", mem_in_kb / 1024.)
        } else {
            format!("{}Ki", mem_in_kb)
        }
    }

    // Formats RAM and Swap to suitable units, saving the result to struct's fields for cheaper usage
    fn save_with_unit(&mut self) {
        self.ram_with_unit = Some(Self::format(self.ram));
        self.swap_with_unit = Some(Self::format(self.swap));
    }

    fn get_ram_with_unit(&self) -> &str {
        self.ram_with_unit
            .as_ref()
            .expect("ram_with_unit is None")
            .as_str()
    }

    fn get_swap_with_unit(&self) -> &str {
        self.swap_with_unit
            .as_ref()
            .expect("swap_with_unit is None")
            .as_str()
    }
}

struct SystemInfo {
    cpu_temp: f32,
    gpu_temp: Option<f32>,
    cpu_time: Vec<usize>,
    mem_free: Memory,
    uptime: f64,
}

impl SystemInfo {
    fn new(gpu: bool, num_cpus: usize) -> SystemInfo {
        let cpu_temp = Self::get_cpu_temp();
        let gpu_temp = match gpu {
            true => Some(Self::get_gpu_temp()),
            false => None,
        };
        let cpu_time = Self::get_cpu_time(num_cpus);
        let mem_free = Memory::get_free();
        let uptime = Self::get_uptime();

        SystemInfo {
            cpu_temp,
            gpu_temp,
            cpu_time,
            mem_free,
            uptime,
        }
    }

    /// Returns the CPU temperature
    fn get_cpu_temp() -> f32 {
        // Returns CPU temp * 1000
        let cpu_temp_content = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
            .expect("Failed to read CPU temp");
        let cpu_temp_1000_f32 = cpu_temp_content
            .trim_end()
            .parse::<f32>()
            .expect("Could not convert CPU temp to f32");
        cpu_temp_1000_f32 / 1000.
    }

    /// Returns the GPU temperature
    fn get_gpu_temp() -> f32 {
        // Returns stdout (standard output stream) and stderr (standard error stream).
        let gpu_temp_output = Command::new("vcgencmd")
            .arg("measure_temp")
            .output()
            .expect("Failed to execute command");
        // Takes stdout (bytestring), turns to utf8, splits at '=' and ''' and saves as vector.
        let gpu_temp_str_splitted: Vec<&str> = str::from_utf8(&gpu_temp_output.stdout)
            .expect("Failed to convert from byte string")
            .split(['=', '\''].as_ref())
            .collect();
        //  Takes second element of vector, i.e. the temperature.
        gpu_temp_str_splitted[1]
            .parse::<f32>()
            .expect("Couldn't convert GPU temp to f32")
    }

    /// Returns the number of CPU(s)
    fn get_cpus() -> usize {
        let n_cpus_output = Command::new("nproc")
            .output()
            .expect("Failed to execute command");
        // Returns the CPU temperature multiplied by 1000.
        let n_cpus_utf8 = str::from_utf8(&n_cpus_output.stdout)
            .expect("Failed to convert n_cpus_outputto utf8 from byte string");

        n_cpus_utf8
            .trim_end()
            .parse::<usize>()
            .expect("Couldn't convert n_cpus_utf8 to usize")
    }

    /// Returns the sum of cumulative CPU times for user and system since boot. TODO: Consider getting time for I/O wait?
    fn get_cpu_time(num_cpus: usize) -> Vec<usize> {
        let proc_stat_content =
            fs::read_to_string("/proc/stat").expect("Failed to read cpu times from file");

        let proc_stat_split: Vec<&str> = proc_stat_content.split(&[' ', '\n'][..]).collect();
        let mut cpu_times = Vec::new();

        for i in 0..num_cpus {
            let search_word = format!("cpu{}", i);
            let index_cpun = proc_stat_split
                .iter()
                .position(|&r| r == search_word)
                .unwrap_or_else(|| panic!("Couldn't find position of {}", search_word));
            let cpu_i_time_sum = proc_stat_split[index_cpun + 1].parse::<usize>().unwrap()
                + proc_stat_split[index_cpun + 1 + 2]
                    .parse::<usize>()
                    .unwrap();
            cpu_times.push(cpu_i_time_sum);
        }
        cpu_times
    }

    /// Returns uptime since boot in seconds
    fn get_uptime() -> f64 {
        let uptime_content = fs::read_to_string("/proc/uptime").expect("Failed to read uptime");
        let uptime_split: Vec<&str> = uptime_content.split(&[' '][..]).collect();
        uptime_split[0]
            .parse::<f64>()
            .expect("Failed to convert uptime to f64")
    }
}

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

    let mut cpu_time_prev = SystemInfo::get_cpu_time(SystemInfo::get_cpus()); // Initial CPU time

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
