use zellij_tile::prelude::*;

use std::collections::BTreeMap;

#[derive(Default)]
struct State {
    userspace_configuration: BTreeMap<String, String>,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.userspace_configuration = configuration;

        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::RunCommands,
        ]);

        subscribe(&[EventType::Key, EventType::RunCommandResult]);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = true;
        match event {
            Event::RunCommandResult(_code, _stdout, _stderr, _ctx) => {
                should_render = true;
            }
            Event::Key(key) => {
                if let Key::Char('n') = key {
                    open_command_pane_floating(CommandToRun {
                        path: "cargo".into(),
                        args: vec!["test".to_owned()],
                        cwd: None,
                    });
                }
            }
            _ => (),
        };
        should_render
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        println!("zjk8s");
    }
}
