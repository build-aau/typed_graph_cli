use crate::cli::*;
use crate::{GenResult, Project};
use build_script_lang::schema::{LowerBound, Schema, SchemaStm};
use clap::Parser;
use graphviz_rust::cmd::{CommandArg, Format, Layout};
use graphviz_rust::dot_generator::{attr, edge, graph, id, node, node_id, stmt};
use graphviz_rust::dot_structures::*;
use graphviz_rust::exec;
use graphviz_rust::printer::PrinterContext;
use std::fs::create_dir_all;
use std::path::PathBuf;

/// Exports the schemas in the project to json files
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Svg {
    /// Optional path to project folder
    #[clap(flatten)]
    pub settings: ProjectSettings,

    /// Output directory to save all the schemas
    #[clap()]
    pub target_dir: PathBuf,
}

impl Process<ProjectSettings> for Svg {
    fn process(&self, settings: &ProjectSettings) -> GenResult<()> {
        let p = self.settings.clone().chain(settings).get_project_path();

        // If this simple command fails then dot is not installed and the docs should not be updated
        let is_dot_installed = exec(graph!(id!("g")), &mut PrinterContext::default(), vec![]);

        if let Err(e) = &is_dot_installed {
            println!("{}", e);
            panic!("Failed to export dot. Probably due to missing instalation of graphviz")
        }

        let prj = Project::open_project(p)?;

        if !self.target_dir.exists() {
            create_dir_all(&self.target_dir)?;
        }

        for schema_id in prj.iter_schema() {
            let schema = prj.get_schema(schema_id)?;
            export_svg(schema, &self.target_dir)?;
        }
        Ok(())
    }
}

pub(crate) fn export_svg<I>(schema: &Schema<I>, target_dir: &PathBuf) -> GenResult<PathBuf>
where
    I: Ord,
{
    let mut g = graph!(
        id!(esc schema.version);
        attr!("mode", "hier"),
        attr!("defaultdist", 100.),
        attr!("overlap", "compress"),
        attr!("nodesep", 0.625),
        attr!("levelsgap", 2)
    );
    for cnt in schema.iter() {
        match cnt {
            SchemaStm::Node(n) => {
                let name = n.name.to_string();
                let label = format!("<&lt;{name}&gt;>");
                let node = node!(
                    name;
                    attr!("label", label),
                    attr!("fontsize", 10)
                );
                g.add_stmt(node.into());
            }
            SchemaStm::Edge(e) => {
                let name = e.name.to_string();

                let edge_inc = e.get_rename_inc();
                let edge_out = e.get_rename_out();

                for ((source, target), endpoint) in &e.endpoints {
                    let mut attributes = vec![attr!("dir", "forward"), attr!("fontsize", 10)];

                    let mut name = format!("&lt;{name}&gt;");
                    if let Some(inc) = edge_inc {
                        name = format!("{name}<br/><FONT COLOR=\"GREY\">{inc}</FONT>");
                    }
                    if let Some(out) = edge_out {
                        name = format!("{name}<br/><FONT COLOR=\"GREY\">{out}</FONT>");
                    }
                    name = format!("<{name}>");
                    attributes.push(attr!("xlabel", name));

                    let mut head_label = match &endpoint.incoming_quantity.bounds {
                        Some((lower, upper)) => match lower {
                            LowerBound::Zero => vec![format!("0..{upper}")],
                            LowerBound::One => vec![format!("1..{upper}")],
                        },
                        None => Default::default(),
                    };

                    if let Some(out) = endpoint.get_rename_inc() {
                        head_label.push(format!("<FONT COLOR=\"GREY\">{out}</FONT>"));
                    }
                    let head_label = format!("<{}>", head_label.join(" - "));
                    attributes.push(attr!("headlabel", head_label));

                    let mut tail_label = match &endpoint.outgoing_quantity.bounds {
                        Some((lower, upper)) => match lower {
                            LowerBound::Zero => vec![format!("0..{upper}")],
                            LowerBound::One => vec![format!("1..{upper}")],
                        },
                        None => Default::default(),
                    };
                    if let Some(out) = endpoint.get_rename_out() {
                        tail_label.push(format!("<FONT COLOR=\"GREY\">{out}</FONT>"));
                    }
                    let tail_label = format!("<{}>", tail_label.join(" - "));
                    attributes.push(attr!("taillabel", tail_label));

                    let edge = edge!(
                        node_id!(esc source) => node_id!(esc target),
                        attributes
                    );
                    g.add_stmt(edge.into());
                }
            }
            _ => (),
        }
    }

    let svg_out_path = target_dir.join(format!("{}.svg", schema.version));
    exec(
        g,
        &mut PrinterContext::default(),
        vec![
            Format::Svg.into(),
            Layout::Neato.into(),
            CommandArg::Output(svg_out_path.to_string_lossy().to_string()),
        ],
    )?;

    Ok(svg_out_path)
}
