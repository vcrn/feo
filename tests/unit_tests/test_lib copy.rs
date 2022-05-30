mod test_lib {
    use super::super::*;

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
        SystemInfo::get_cpu_temp();
    }

    #[test]
    fn test_get_cpu_time() {
        SystemInfo::get_cpu_time(SystemInfo::get_cpus());
    }

    #[test]
    fn test_get_uptime() {
        SystemInfo::get_uptime();
    }

    #[test]
    #[ignore] // Fails on VM such as Github Actions
    fn test_system_info_new_gpu_false() {
        SystemInfo::new(false, SystemInfo::get_cpus());
    }
}
