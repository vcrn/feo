use termion::color::Rgb;

/// Colors to print parts of the monitor in
pub struct Colors {
    pub temp: Rgb,
    pub cpu: Rgb,
    pub mem: Rgb,
    pub uptime: Rgb,
}

impl Colors {
    pub fn new(color: char) -> Colors {
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

#[cfg(test)]
mod tests {
    use super::*;

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
