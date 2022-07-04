/// Used for storing total and available memory
use std::process::Command;
use std::{fs, str};

#[cfg(test)]
#[path = "../tests/unit_tests/test_system_info.rs"]
mod test_system_info;

pub struct Memory {
    pub ram: f32,
    pub swap: f32,
    ram_with_unit: Option<String>,
    swap_with_unit: Option<String>,
}

impl Memory {
    // Creates an instance storing memory matching the search word arguments.
    pub fn new(search_word_ram: &str, search_word_swap: &str) -> Memory {
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
    pub fn get_total() -> Memory {
        Self::new("MemTotal", "SwapTotal")
    }

    // Returns argument, given in kb, formatted to suitable unit.
    pub fn format(mem_in_kb: f32) -> String {
        if mem_in_kb > 1000000. {
            format!("{:.2}Gi", mem_in_kb / 1048576.)
        } else if mem_in_kb > 1000. {
            format!("{:.2}Mi", mem_in_kb / 1024.)
        } else {
            format!("{}Ki", mem_in_kb)
        }
    }

    // Formats RAM and Swap to suitable units, saving the result to struct's fields for cheaper usage
    pub fn save_with_unit(&mut self) {
        self.ram_with_unit = Some(Self::format(self.ram));
        self.swap_with_unit = Some(Self::format(self.swap));
    }

    pub fn get_ram_with_unit(&self) -> &str {
        self.ram_with_unit
            .as_ref()
            .expect("ram_with_unit is None")
            .as_str()
    }

    pub fn get_swap_with_unit(&self) -> &str {
        self.swap_with_unit
            .as_ref()
            .expect("swap_with_unit is None")
            .as_str()
    }
}

pub struct SystemInfo {
    pub cpu_temp: f32,
    pub gpu_temp: Option<f32>,
    pub cpu_time: Vec<usize>,
    pub mem_free: Memory,
    pub uptime: f64,
}

impl SystemInfo {
    pub fn new(gpu: bool, num_cpus: usize) -> SystemInfo {
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
    pub fn get_cpus() -> usize {
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
        let mut cpu_times = Vec::with_capacity(num_cpus);

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
