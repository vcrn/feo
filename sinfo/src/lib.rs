/// Checks Linux systems' CPU and GPU temperature, RAM and swap (total and free), CPU time, number of CPUs, and uptime.
use std::process::Command;
use std::{fs, str};

use anyhow::bail;

/// Used for storing total or available RAM and Swap.
pub struct Memory {
    pub ram: f32,                   // RAM in KiB
    pub swap: f32,                  // Swap in KiB
    ram_with_unit: Option<String>,  // RAM formatted with a suitable unit
    swap_with_unit: Option<String>, // Swap formatted with a suitable unit
}

impl Memory {
    /// Creates an instance storing memory matching the search word arguments.
    fn new(search_word_ram: &str, search_word_swap: &str) -> Result<Memory, anyhow::Error> {
        let meminfo_content = fs::read_to_string("/proc/meminfo")?;

        let mut meminfo_split: Vec<&str> = meminfo_content.split(&[':', ' ', '\n'][..]).collect();

        meminfo_split.retain(|&x| !x.is_empty()); // Removes empty elements.

        let index_ram_search = match meminfo_split.iter().position(|&r| r == search_word_ram) {
            Some(i) => i,
            None => bail!("Couldn't find position of {}", search_word_ram),
        };

        let index_swap_search = match meminfo_split.iter().position(|&r| r == search_word_swap) {
            Some(i) => i,
            None => bail!("Couldn't find position of {}", search_word_swap),
        };

        let ram_kib = meminfo_split[index_ram_search + 1].parse::<f32>()?;

        let swap_kib = meminfo_split[index_swap_search + 1].parse::<f32>()?;

        Ok(Memory {
            ram: ram_kib,
            swap: swap_kib,
            ram_with_unit: None,
            swap_with_unit: None,
        })
    }

    /// Returns available RAM and free Swap
    pub fn get_free() -> Result<Memory, anyhow::Error> {
        Self::new("MemAvailable", "SwapFree")
    }

    /// Returns Memory instance storing total RAM and total Swap
    pub fn get_total() -> Result<Memory, anyhow::Error> {
        Self::new("MemTotal", "SwapTotal")
    }

    /// Returns argument, given in kb, formatted to suitable unit.
    pub fn format(mem_in_kb: f32) -> String {
        if mem_in_kb > 1000000. {
            format!("{:.2}Gi", mem_in_kb / 1048576.)
        } else if mem_in_kb > 1000. {
            format!("{:.2}Mi", mem_in_kb / 1024.)
        } else {
            format!("{mem_in_kb}Ki")
        }
    }

    /// Formats RAM and Swap to suitable units, saving the result to struct's fields for cheaper usage
    pub fn save_with_unit(&mut self) {
        self.ram_with_unit = Some(Self::format(self.ram));
        self.swap_with_unit = Some(Self::format(self.swap));
    }

    /// Returns the RAM with unit as a &str
    pub fn get_ram_with_unit(&self) -> &str {
        self.ram_with_unit
            .as_ref()
            .expect("ram_with_unit is None")
            .as_str()
    }

    /// Returns the Swap with unit as a &str
    pub fn get_swap_with_unit(&self) -> &str {
        self.swap_with_unit
            .as_ref()
            .expect("swap_with_unit is None")
            .as_str()
    }
}

/// Stores CPU and GPU temperature, CPU time since boot, free RAM and swap, and uptime.
pub struct SystemInfo {
    pub cpu_temp: f32,
    pub gpu_temp: Option<f32>, // Only available for Raspberry Pi
    pub cpu_time: Vec<usize>,  // Sum of cumulative CPU times for user and system since boot
    pub mem_free: Memory,      // Available RAM and swap
    pub uptime: f64,           // Uptime since boot in seconds
}

impl SystemInfo {
    /// Returns an object containing system information.
    /// `gpu` controls whether GPU temperature is to be checked, which only works for Raspberry Pi.
    /// `num_cpus` is the number of CPUs and can be supplied with SystemInfo::get_num_cpus.
    pub fn new(gpu: bool, num_cpus: usize) -> Result<SystemInfo, anyhow::Error> {
        let cpu_temp = Self::get_cpu_temp()?;
        let gpu_temp = match gpu {
            true => Some(Self::get_gpu_temp()?),
            false => None,
        };
        let cpu_time = Self::get_cpu_time(num_cpus)?;
        let mem_free = Memory::get_free()?;
        let uptime = Self::get_uptime()?;

        Ok(SystemInfo {
            cpu_temp,
            gpu_temp,
            cpu_time,
            mem_free,
            uptime,
        })
    }

    /// Returns the CPU temperature
    fn get_cpu_temp() -> Result<f32, anyhow::Error> {
        // Returns CPU temp * 1000
        let cpu_temp_content = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")?;
        let cpu_temp_1000_f32 = cpu_temp_content.trim_end().parse::<f32>()?;
        Ok(cpu_temp_1000_f32 / 1000.)
    }

    /// Returns the GPU temperature (only available for Raspberry Pi)
    fn get_gpu_temp() -> Result<f32, anyhow::Error> {
        // Returns stdout (standard output stream) and stderr (standard error stream).
        let gpu_temp_output = Command::new("vcgencmd").arg("measure_temp").output()?;
        // Takes stdout (bytestring), turns to utf8, splits at '=' and ''' and saves as vector.
        let gpu_temp_str_splitted: Vec<&str> = str::from_utf8(&gpu_temp_output.stdout)
            .expect("Failed to convert from byte string")
            .split(['=', '\''].as_ref())
            .collect();
        //  Takes second element of vector, i.e. the temperature.
        Ok(gpu_temp_str_splitted[1].parse::<f32>()?)
    }

    /// Returns the number of CPU(s)
    pub fn get_cpus() -> Result<usize, anyhow::Error> {
        let n_cpus_output = Command::new("nproc").output()?;
        let n_cpus_utf8 = str::from_utf8(&n_cpus_output.stdout)?;
        Ok(n_cpus_utf8.trim_end().parse::<usize>()?)
    }

    /// Returns the sum of cumulative CPU times for user and system since boot. TODO: Consider getting time for I/O wait?
    fn get_cpu_time(num_cpus: usize) -> Result<Vec<usize>, anyhow::Error> {
        let proc_stat_content =
            fs::read_to_string("/proc/stat").expect("Failed to read cpu times from file");

        let proc_stat_split: Vec<&str> = proc_stat_content.split(&[' ', '\n'][..]).collect();
        let mut cpu_times = Vec::with_capacity(num_cpus);

        for i in 0..num_cpus {
            let search_word = format!("cpu{i}");

            let index_cpun = match proc_stat_split.iter().position(|&r| r == search_word) {
                Some(i) => i,
                None => bail!("Couldn't find position of {search_word}"),
            };

            let cpu_i_time_sum = proc_stat_split[index_cpun + 1].parse::<usize>().unwrap()
                + proc_stat_split[index_cpun + 1 + 2].parse::<usize>()?;
            cpu_times.push(cpu_i_time_sum);
        }
        Ok(cpu_times)
    }

    /// Returns uptime since boot in seconds
    fn get_uptime() -> Result<f64, anyhow::Error> {
        let uptime_content = fs::read_to_string("/proc/uptime").expect("Failed to read uptime");
        let uptime_split: Vec<&str> = uptime_content.split(&[' '][..]).collect();
        Ok(uptime_split[0].parse::<f64>()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_mem_with_unit() {
        let mut mem = Memory {
            ram: 16000000.0,
            swap: 100000.0,
            ram_with_unit: None,
            swap_with_unit: None,
        };
        mem.save_with_unit();
        let correct_format_ram = format!("{:.2}Gi", mem.ram / (1024.0 * 1024.0));
        let correct_format_swap = format!("{:.2}Mi", mem.swap / (1024.0));

        assert_eq!(correct_format_ram, mem.get_ram_with_unit());
        assert_eq!(correct_format_swap, mem.get_swap_with_unit());
    }

    #[test]
    #[ignore] // Fails on VM such as Github Actions
    fn test_get_cpu_temp() {
        SystemInfo::get_cpu_temp().unwrap();
    }

    #[test]
    fn test_get_cpu_time() {
        SystemInfo::get_cpu_time(SystemInfo::get_cpus().unwrap()).unwrap();
    }

    #[test]
    fn test_get_uptime() {
        SystemInfo::get_uptime().unwrap();
    }

    #[test]
    #[ignore] // Fails on VM such as Github Actions
    fn test_system_info_new_gpu_false() {
        SystemInfo::new(false, SystemInfo::get_cpus().unwrap()).unwrap();
    }
}
