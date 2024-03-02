use miette::Result;
use zellij_tile::prelude::*;
use zjk8s::{
    kubernetes::{self, ListDir},
    render::{self, ColType},
};

use std::collections::BTreeMap;

#[derive(Default)]
struct State {
    userspace_configuration: BTreeMap<String, String>,
    cluster_state: kubernetes::State,
    selected_col: ColType,
    error_message: Option<Result<()>>,
    renderer: render::Render,
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

        self.renderer = render::Render::new();
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::RunCommandResult(exit_code, stdout, stderr, context) => {
                self.error_message = Some(
                    self.cluster_state
                        .parse_result(exit_code, stdout, stderr, context),
                );

                should_render = true;
            }
            Event::Key(key) => match key {
                Key::Left => {
                    self.selected_col = match self.selected_col {
                        ColType::Namespace => ColType::Namespace,
                        ColType::ResourceType => ColType::Namespace,
                        ColType::Resource => ColType::ResourceType,
                        ColType::ResourceDetails => ColType::Resource,
                    };

                    should_render = true;
                }
                Key::Right => {
                    self.selected_col = match self.selected_col {
                        ColType::Namespace => ColType::ResourceType,
                        ColType::ResourceType => ColType::Resource,
                        ColType::Resource => ColType::ResourceDetails,
                        ColType::ResourceDetails => ColType::ResourceDetails,
                    };

                    should_render = true;
                }
                Key::Up => {
                    self.cluster_state
                        .select_item(ListDir::Up, &self.selected_col);

                    should_render = true;
                }
                Key::Down => {
                    self.cluster_state
                        .select_item(ListDir::Down, &self.selected_col);

                    should_render = true;
                }
                Key::Char('\n') => {
                    if self.selected_col == ColType::Resource {
                        let namespace = self.cluster_state.get_selected_item(&ColType::Namespace);
                        let resource_type =
                            self.cluster_state.get_selected_item(&ColType::ResourceType);
                        let resource = self.cluster_state.get_selected_item(&ColType::Resource);

                        if namespace.is_some() && resource_type.is_some() && resource.is_some() {
                            self.cluster_state.refresh_resource_details = true;
                            should_render = true;

                            kubernetes::query_resource_details(
                                self.userspace_configuration
                                    .get("context")
                                    .map(|s| s.as_str()),
                                &namespace.unwrap(),
                                &resource_type.unwrap(),
                                &resource.unwrap(),
                            );

                            self.selected_col = ColType::ResourceDetails;
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        };
        should_render
    }

    fn render(&mut self, rows: usize, cols: usize) {
        let k8s_context = self
            .userspace_configuration
            .get("context")
            .map(|s| s.as_str());

        if self.cluster_state.namespaces.is_none() {
            eprintln!("Querying namespaces...");
            kubernetes::query_namespaces(k8s_context);
        }

        if self.cluster_state.refresh_resource_types {
            eprintln!("Querying resource types...");

            self.refresh_resource_types(&k8s_context);
        }

        if self.cluster_state.refresh_resources {
            eprintln!("Querying resources...");

            self.refresh_resources(&k8s_context);
        }

        if let Some(Err(e)) = &self.error_message {
            println!("Error: {:?}", e);

            return;
        }

        self.renderer
            .render_cluster_state(&self.cluster_state, &self.selected_col, rows, cols)
    }
}

impl State {
    fn refresh_resource_types(&self, k8s_context: &Option<&str>) {
        let namespace = match self.cluster_state.get_selected_item(&ColType::Namespace) {
            Some(namespace) => namespace,
            None => return,
        };

        kubernetes::query_resource_types(k8s_context, &namespace);
    }

    fn refresh_resources(&self, k8s_context: &Option<&str>) {
        let namespace = match self.cluster_state.get_selected_item(&ColType::Namespace) {
            Some(namespace) => namespace,
            None => return,
        };

        let resource_type = match self.cluster_state.get_selected_item(&ColType::ResourceType) {
            Some(resource_type) => resource_type,
            None => return,
        };

        kubernetes::query_resources(k8s_context, &namespace, &resource_type);
    }
}
