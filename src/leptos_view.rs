use leptos::prelude::*;

use crate::profile::apply_profile;
use crate::schema::{ProfileViewKind, TopoxideDocument};

#[component]
pub fn TopoxideExplorer(document: TopoxideDocument, selected_profile: String) -> impl IntoView {
    let profile = if selected_profile.is_empty() {
        document
            .profiles
            .first()
            .map(|profile| profile.id.clone())
            .unwrap_or_else(|| "default".to_owned())
    } else {
        selected_profile
    };
    let view = apply_profile(&document, Some(&profile)).ok();
    let profiles = document.profiles.clone();
    view! {
        <section class="topoxide-explorer" data-testid="topoxide-explorer">
            <div class="topoxide-toolbar">
                <div class="btn-group flex-wrap" role="group" aria-label="ETL graph profiles">
                    {profiles.into_iter().map(|candidate| {
                        let active = candidate.id == profile;
                        let href = format!("?profile={}", candidate.id);
                        view! {
                            <a
                                class=if active { "btn btn-primary btn-sm" } else { "btn btn-outline-primary btn-sm" }
                                href=href
                                data-testid=format!("topoxide-profile-{}", candidate.id)
                            >
                                {candidate.label}
                            </a>
                        }
                    }).collect_view()}
                </div>
                <form method="get" class="topoxide-search" role="search">
                    <input class="form-control form-control-sm" name="q" placeholder="Filter nodes"/>
                    <input type="hidden" name="profile" value=profile.clone()/>
                </form>
            </div>
            {match view {
                Some(view) if view.profile.view == ProfileViewKind::Matrix => {
                    view! { <TopoxideMatrix view=view/> }.into_any()
                }
                Some(view) => view! { <TopoxideLaneGraph view=view/> }.into_any(),
                None => view! {
                    <div class="alert alert-warning" role="alert">"The ETL graph document could not be rendered."</div>
                }.into_any(),
            }}
        </section>
    }
}

#[component]
fn TopoxideLaneGraph(view: crate::profile::ProfileView) -> impl IntoView {
    let nodes = view.graph.nodes.clone();
    let edges = view.graph.edges.clone();
    let groups = view.graph.groups.clone();
    view! {
        <div class="topoxide-layout" data-testid="topoxide-lane-graph">
            <div class="topoxide-lanes">
                {groups.into_iter().map(|group| {
                    let group_nodes = nodes
                        .iter()
                        .filter(|node| node.group == group.id)
                        .cloned()
                        .collect::<Vec<_>>();
                    view! {
                        <div class="topoxide-lane">
                            <h2>{group.label}</h2>
                            <div class="topoxide-node-list">
                                {group_nodes.into_iter().map(|node| {
                                    let node_id = node.id.clone();
                                    let node_kind = node.kind.clone();
                                    let node_label = node.label.clone();
                                    let node_detail = node.detail.clone();
                                    view! {
                                        <button
                                            class=format!("topoxide-node topoxide-kind-{}", node_kind)
                                            type="button"
                                            data-node-id=node_id
                                            title=node_detail.clone().unwrap_or_default()
                                        >
                                            <span>{node_label}</span>
                                            {node_detail.clone().map(|detail| view! { <small>{detail}</small> })}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
            <div class="topoxide-edge-summary" data-testid="topoxide-edge-summary">
                <span>{edges.len()}</span>
                " visible edge"
                {if edges.len() == 1 { "" } else { "s" }}
            </div>
        </div>
    }
}

#[component]
fn TopoxideMatrix(view: crate::profile::ProfileView) -> impl IntoView {
    let Some(matrix) = view.matrix else {
        return view! { <div></div> }.into_any();
    };
    view! {
        <div class="table-responsive topoxide-matrix" data-testid="topoxide-matrix">
            <table class="table table-sm align-middle">
                <thead>
                    <tr>
                        <th>"Dataset"</th>
                        {matrix.columns.iter().map(|column| view! { <th>{column.label.clone()}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {matrix.rows.into_iter().map(|row| {
                        view! {
                            <tr>
                                <th scope="row">{row.node.label}</th>
                                {row.cells.into_iter().map(|cell| {
                                    view! {
                                        <td>
                                            <span class={if cell.count > 0 { "topoxide-cell active" } else { "topoxide-cell" }}>
                                                {if cell.count > 0 { cell.count.to_string() } else { String::new() }}
                                            </span>
                                        </td>
                                    }
                                }).collect_view()}
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }.into_any()
}
