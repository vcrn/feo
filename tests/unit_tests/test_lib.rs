mod test_lib {
    use super::super::*;

    #[test]
    fn test_print_comp_temp() {
        let colors = Colors::new('s');
        let temp = 38.0;
        let component = "CPU";

        print_comp_temp(colors.cpu, temp, component);
    }
}
