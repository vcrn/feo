mod test_lib {
    use super::super::*;

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
