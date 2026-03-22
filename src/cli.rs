use crate::Mode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliConfig {
    pub mode: Mode,
    pub clipboard: bool,
    pub preview: bool,
    pub print: bool,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            mode: Mode::Prose,
            clipboard: false,
            preview: false,
            print: false,
        }
    }
}

impl CliConfig {
    pub fn parse<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut config = Self::default();

        for arg in args {
            match arg.as_ref() {
                "prose" => config.mode = Mode::Prose,
                "command" => config.mode = Mode::Command,
                "auto" => config.mode = Mode::Auto,
                "--clipboard" => config.clipboard = true,
                "--preview" => config.preview = true,
                "--print" => config.print = true,
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        if config.clipboard && config.preview && config.print {
            return Err(String::from(
                "cannot combine --preview and --print with --clipboard",
            ));
        }

        Ok(config)
    }
}
