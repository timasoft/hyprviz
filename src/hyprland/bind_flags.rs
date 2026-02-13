use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BindFlags {
    pub locked: bool,
    pub release: bool,
    pub click: bool,
    pub drag: bool,
    pub long_press: bool,
    pub repeat: bool,
    pub non_consuming: bool,
    pub mouse: bool,
    pub transparent: bool,
    pub ignore_mods: bool,
    pub separate: bool,
    pub has_description: bool,
    pub bypass: bool,
}

impl FromStr for BindFlags {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut flags = BindFlags::default();
        for c in s.trim().chars() {
            match c {
                'l' => flags.locked = true,
                'r' => flags.release = true,
                'c' => flags.click = true,
                'g' => flags.drag = true,
                'o' => flags.long_press = true,
                'e' => flags.repeat = true,
                'n' => flags.non_consuming = true,
                'm' => flags.mouse = true,
                't' => flags.transparent = true,
                'i' => flags.ignore_mods = true,
                's' => flags.separate = true,
                'd' => flags.has_description = true,
                'p' => flags.bypass = true,
                _ => {}
            }
        }
        Ok(flags)
    }
}

impl Display for BindFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flags = String::new();
        if self.locked {
            flags.push('l');
        }
        if self.release {
            flags.push('r');
        }
        if self.click {
            flags.push('c');
        }
        if self.drag {
            flags.push('g');
        }
        if self.long_press {
            flags.push('o');
        }
        if self.repeat {
            flags.push('e');
        }
        if self.non_consuming {
            flags.push('n');
        }
        if self.mouse {
            flags.push('m');
        }
        if self.transparent {
            flags.push('t');
        }
        if self.ignore_mods {
            flags.push('i');
        }
        if self.separate {
            flags.push('s');
        }
        if self.has_description {
            flags.push('d');
        }
        if self.bypass {
            flags.push('p');
        }
        write!(f, "{}", flags)
    }
}
