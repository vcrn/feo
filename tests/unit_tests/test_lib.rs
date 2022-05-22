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
    fn test_color_new_white() {
        let colors = Colors::new('w');

        assert_eq!(Rgb(255, 255, 255), colors.temp);
        assert_eq!(Rgb(255, 255, 255), colors.cpu);
        assert_eq!(Rgb(255, 255, 255), colors.mem);
        assert_eq!(Rgb(255, 255, 255), colors.uptime);
    }

    #[test]
    fn test_color_new_no_match_returns_standard() {
        let colors_mo_match = Colors::new('x');
        let colors_std = Colors::new('s');

        assert_eq!(colors_std.temp, colors_mo_match.temp);
        assert_eq!(colors_std.cpu, colors_mo_match.cpu);
        assert_eq!(colors_std.mem, colors_mo_match.mem);
        assert_eq!(colors_std.uptime, colors_mo_match.uptime);
    }

    #[test]
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
    fn test_system_info_new_gpu_false() {
        SystemInfo::new(false, SystemInfo::get_cpus());
    }
}
