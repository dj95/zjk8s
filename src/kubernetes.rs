use itertools::Itertools;
use miette::{Diagnostic, SourceSpan};
use miette::{NamedSource, Result};
use std::{collections::BTreeMap, u8};
use thiserror::Error;

use zellij_tile::prelude::*;

use crate::render::ColType;

pub enum ListDir {
    Up,
    Down,
}

#[derive(Default)]
pub struct State {
    // kubectl get namespace
    pub namespaces: Option<Vec<String>>,
    pub selected_namespace: Option<String>,
    pub refresh_namespaces: bool,

    // kubectl get api-resources correlated with kubectl get
    pub resource_types: Option<Vec<String>>,
    pub selected_resource_type: Option<String>,
    pub refresh_resource_types: bool,

    // kubectl get <resource_type>
    pub resources: Option<Vec<String>>,
    pub selected_resource: Option<String>,
    pub refresh_resources: bool,

    // kubectl get <resource_type>/<resource>
    pub resource_details: Option<String>,
    pub refresh_resource_details: bool,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Wrong Exit Code")]
#[diagnostic(help("There was en error running the command"))]
struct WrongExitCode {
    #[source_code]
    src: NamedSource<String>,

    #[label("This error occured")]
    bad_bit: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Wrong Exit Code")]
#[diagnostic(help("There was en error running the command"))]
struct WrongFormat {
    #[source_code]
    src: NamedSource<String>,

    #[label("This error occured")]
    bad_bit: SourceSpan,
}

pub fn query_namespaces(kube_context: Option<&str>) {
    let command_ctx: BTreeMap<String, String> =
        BTreeMap::from([("command".to_owned(), "query_namespaces".to_owned())]);

    if let Some(context) = kube_context {
        run_command(
            &[
                "kubectl",
                "get",
                "namespaces",
                "--context",
                context,
                "--output",
                "jsonpath={.items[*].metadata.name}",
            ],
            command_ctx,
        );
    } else {
        run_command(
            &[
                "kubectl",
                "get",
                "namespaces",
                "--output",
                "jsonpath={.items[*].metadata.name}",
            ],
            command_ctx,
        );
    }
}

pub fn query_resource_types(kube_context: Option<&str>, namespace: &str) {
    let command_ctx: BTreeMap<String, String> =
        BTreeMap::from([("command".to_owned(), "query_resource_types".to_owned())]);

    if let Some(context) = kube_context {
        run_command(
            &[
                "kubectl",
                "get",
                "all,ConfigMap,Endpoints,LimitRange,PersistentVolumeClaim,PersistentVolume,Pod,ReplicationController,ResourceQuota,Secret,Service,ServiceAccount",
                "--context",
                context,
                "--namespace",
                namespace,
                "--output",
                "jsonpath={.items[*].kind}",
            ],
            command_ctx,
        );
    } else {
        run_command(
            &[
                "kubectl",
                "get",
                "all,ConfigMap,Endpoints,LimitRange,PersistentVolumeClaim,PersistentVolume,Pod,ReplicationController,ResourceQuota,Secret,Service,ServiceAccount",
                "--namespace",
                namespace,
                "--output",
                "jsonpath={.items[*].kind}",
            ],
            command_ctx,
        );
    }
}

pub fn query_resources(kube_context: Option<&str>, namespace: &str, resource_type: &str) {
    let command_ctx: BTreeMap<String, String> =
        BTreeMap::from([("command".to_owned(), "query_resources".to_owned())]);

    if let Some(context) = kube_context {
        run_command(
            &[
                "kubectl",
                "get",
                resource_type,
                "--context",
                context,
                "--namespace",
                namespace,
                "--output",
                "jsonpath={.items[*].metadata.name}",
            ],
            command_ctx,
        );
    } else {
        run_command(
            &[
                "kubectl",
                "get",
                resource_type,
                "--namespace",
                namespace,
                "--output",
                "jsonpath={.items[*].metadata.name}",
            ],
            command_ctx,
        );
    }
}

impl State {
    pub fn select_item(&mut self, direction: ListDir, col_type: &ColType) {
        match col_type {
            ColType::Namespace => {
                if let Some(namespaces) = &self.namespaces {
                    self.selected_namespace = Some(get_next_item(
                        namespaces,
                        &self.selected_namespace,
                        direction,
                    ));
                    self.refresh_resource_types = true;
                }
            }
            ColType::ResourceType => {
                if let Some(resource_types) = &self.resource_types {
                    self.selected_resource_type = Some(get_next_item(
                        resource_types,
                        &self.selected_resource_type,
                        direction,
                    ));
                    self.refresh_resources = true;
                }
            }
            ColType::Resource => {
                if let Some(resources) = &self.resources {
                    self.selected_resource =
                        Some(get_next_item(resources, &self.selected_resource, direction));
                    self.refresh_resource_details = true;
                }
            }
            ColType::ResourceDetails => {
                if let Some(resource_details) = &self.resource_details {
                    self.resource_details = Some(resource_details.to_string());
                }
            }
        }
    }

    pub fn parse_result(
        &mut self,
        exit_code: Option<i32>,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        context: BTreeMap<String, String>,
    ) -> Result<()> {
        match context.get("command") {
            Some(command) => match command.as_str() {
                "query_namespaces" => {
                    let result = self.result(exit_code, stdout, stderr, context)?;

                    self.namespaces = Some(result.0);
                    self.selected_namespace = Some(result.1);

                    self.refresh_namespaces = false;
                    self.refresh_resource_types = true;

                    Ok(())
                }
                "query_resource_types" => {
                    let result = self.result(exit_code, stdout, stderr, context)?;

                    self.resource_types = Some(result.0);
                    self.selected_resource_type = Some(result.1);

                    self.refresh_resource_types = false;
                    self.refresh_resources = true;

                    Ok(())
                }
                "query_resources" => {
                    let result = self.result(exit_code, stdout, stderr, context)?;

                    self.resources = Some(result.0);
                    self.selected_resource = Some(result.1);

                    self.refresh_resources = false;
                    self.refresh_resource_details = true;

                    Ok(())
                }
                _ => Ok(()),
            },
            None => Ok(()),
        }
    }

    fn result(
        &mut self,
        exit_code: Option<i32>,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        _context: BTreeMap<String, String>,
    ) -> Result<(Vec<String>, String)> {
        guard_exit_code(exit_code, stderr)?;

        match String::from_utf8(stdout) {
            Ok(resource) => {
                let resources = resource
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .unique()
                    .collect();

                let selected_resource =
                    resource.split_whitespace().collect::<Vec<&str>>()[0].to_owned();
                Ok((resources, selected_resource))
            }
            Err(e) => Err(WrongExitCode {
                src: NamedSource::new("kubernetes.rs", format!("Error parsing stdout: {}", e)),
                bad_bit: (1, 2).into(),
            }
            .into()),
        }
    }
}

fn get_next_item(items: &[String], selected_item: &Option<String>, direction: ListDir) -> String {
    let selected_item = match selected_item {
        Some(rt) => rt,
        None => &items[0],
    };

    let index = items.iter().position(|r| r == selected_item).unwrap();

    match direction {
        ListDir::Up => {
            if index == 0 {
                items[items.len() - 1].to_string()
            } else {
                items[index - 1].to_string()
            }
        }
        ListDir::Down => {
            if index == items.len() - 1 {
                items[0].to_string()
            } else {
                items[index + 1].to_string()
            }
        }
    }
}

fn guard_exit_code(exit_code: Option<i32>, stderr: Vec<u8>) -> Result<()> {
    if let Some(code) = exit_code {
        if code != 0 {
            let err = match String::from_utf8(stderr) {
                Ok(s) => s,
                Err(e) => format!("Error parsing stderr: {}", e),
            };

            return Err(WrongExitCode {
                src: NamedSource::new("kubernetes.rs", err),
                bad_bit: (1, 2).into(),
            }
            .into());
        }
    }

    Ok(())
}
