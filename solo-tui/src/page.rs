// Map
//   C  
// D S E
//   L V
#[derive(Debug, Default)]
pub enum Page {
    #[default]
    Scene,
    Devices,
    Edit,
    Configure,
    Logs,
    Vars
}

impl Page {

    pub fn left(&mut self) {
        *self = match self {
            Page::Scene => Page::Devices,
            Page::Devices => Page::Devices,
            Page::Edit => Page::Scene,
            Page::Configure => Page::Configure,
            Page::Logs => Page::Logs,
            Page::Vars => Page::Vars,
        }
    }

    pub fn right(&mut self) {
        *self = match self {
            Page::Scene => Page::Edit,
            Page::Devices => Page::Scene,
            Page::Edit => Page::Edit,
            Page::Configure => Page::Configure,
            Page::Logs => Page::Logs,
            Page::Vars => Page::Logs,
        }
    }

    pub fn up(&mut self) {
        *self = match self {
            Page::Scene => Page::Configure,
            Page::Devices => Page::Devices,
            Page::Edit => Page::Edit,
            Page::Configure => Page::Configure,
            Page::Logs => Page::Scene,
            Page::Vars => Page::Edit,
        }
    }

    pub fn down(&mut self) {
        *self = match self {
            Page::Scene => Page::Logs,
            Page::Devices => Page::Devices,
            Page::Edit => Page::Vars,
            Page::Configure => Page::Scene,
            Page::Logs => Page::Logs,
            Page::Vars => Page::Vars,
        }
    }

}