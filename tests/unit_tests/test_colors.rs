mod test_colors {
    use super::super::*;

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
}
