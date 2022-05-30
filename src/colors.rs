use termion::color::Rgb;

#[cfg(test)]
#[path = "../tests/unit_tests/test_colors.rs"]
mod test_colors;

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
