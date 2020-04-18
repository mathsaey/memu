use std::error::Error;
use std::io;

use tui::backend::CrosstermBackend;

type Terminal = tui::Terminal<CrosstermBackend<io::Stdout>>;

pub struct DebugView(Option<Terminal>);

impl DebugView {
    pub fn new(enable: bool) -> Result<DebugView, Box<dyn Error>> {
        if enable {
            let backend = CrosstermBackend::new(io::stdout());
            let mut terminal = Terminal::new(backend)?;
            draw_initial(&mut terminal);
            Ok(DebugView(Some(terminal)))
        } else {
            Ok(DebugView(None))
        }
    }
}

fn draw_initial(terminal: &mut Terminal) {}
