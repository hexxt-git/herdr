//! Sidebar panel listing the dev servers detected in this session's panes.
//!
//! Detection is server-side; the snapshot carries the result. This module only
//! decides where the panel sits and draws it.

use ratatui::{buffer::Buffer, layout::Rect, style::Modifier};

use super::render::put_text;
use super::*;

pub(super) const SERVER_PANEL_HEADER_ROWS: u16 = 3;
const SERVER_ENTRY_ROWS: u16 = 3;
/// Rows the agent panel keeps for its own header before the servers panel may
/// take any space.
const AGENT_PANEL_MIN_ROWS: u16 = 3;

pub(super) fn panel_visible(snapshot: &ClientShellSnapshot) -> bool {
    !snapshot.dev_servers.is_empty()
}

/// Split the sidebar detail area between agents (top) and servers (bottom).
///
/// Returns a zero-height server area when there is too little room to draw a
/// header plus one entry, so a stub panel never costs the agent panel space.
pub(super) fn split_detail_area(snapshot: &ClientShellSnapshot, area: Rect) -> (Rect, Rect) {
    if area.height == 0 || !panel_visible(snapshot) {
        return (area, Rect::default());
    }
    let count = u16::try_from(snapshot.dev_servers.len()).unwrap_or(u16::MAX);
    let wanted = SERVER_PANEL_HEADER_ROWS.saturating_add(count.saturating_mul(SERVER_ENTRY_ROWS));
    let available = area
        .height
        .saturating_sub(AGENT_PANEL_MIN_ROWS)
        .min(area.height / 2);
    let server_h = wanted.min(available);
    let server_h = if server_h < SERVER_PANEL_HEADER_ROWS + 1 {
        0
    } else {
        server_h
    };
    let agent_h = area.height.saturating_sub(server_h);
    (
        Rect::new(area.x, area.y, area.width, agent_h),
        Rect::new(area.x, area.y + agent_h, area.width, server_h),
    )
}

pub(super) fn render_dev_server_panel(
    buffer: &mut Buffer,
    area: Rect,
    snapshot: &ClientShellSnapshot,
    config: &ClientShellConfig,
    hits: &mut ShellHitMap,
) {
    if area.height < SERVER_PANEL_HEADER_ROWS || area.width == 0 {
        return;
    }
    let palette = &config.palette;

    put_text(
        buffer,
        area.x,
        area.y,
        area.width,
        &"─".repeat(area.width as usize),
        Style::default().fg(palette.surface_dim),
    );
    put_text(
        buffer,
        area.x,
        area.y + 1,
        area.width,
        " servers",
        Style::default()
            .fg(palette.overlay0)
            .add_modifier(Modifier::BOLD),
    );

    let body_y = area.y + SERVER_PANEL_HEADER_ROWS;
    let body_bottom = area.y + area.height;
    let mut row_y = body_y;

    for server in &snapshot.dev_servers {
        if row_y.saturating_add(1) >= body_bottom {
            break;
        }
        let focused = snapshot.focused_pane_id.as_deref() == Some(server.pane_id.as_str());
        let name_style = if focused {
            Style::default()
                .fg(palette.text)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(palette.subtext0)
                .add_modifier(Modifier::BOLD)
        };
        let port_style = if focused {
            Style::default().fg(palette.overlay0)
        } else {
            Style::default()
                .fg(palette.overlay0)
                .add_modifier(Modifier::DIM)
        };

        put_text(
            buffer,
            area.x,
            row_y,
            area.width,
            " ✓ ",
            Style::default()
                .fg(palette.green)
                .add_modifier(Modifier::BOLD),
        );
        put_text(
            buffer,
            area.x + 3,
            row_y,
            area.width.saturating_sub(3),
            &server.workspace_label,
            name_style,
        );

        put_text(
            buffer,
            area.x + 3,
            row_y + 1,
            area.width.saturating_sub(3),
            &server.tool,
            Style::default().fg(palette.green),
        );
        let tool_width = u16::try_from(server.tool.chars().count()).unwrap_or(u16::MAX);
        put_text(
            buffer,
            area.x + 3 + tool_width,
            row_y + 1,
            area.width.saturating_sub(3 + tool_width),
            &format!(" · localhost:{}", server.port),
            port_style,
        );

        hits.dev_servers.push((
            Rect::new(area.x, row_y, area.width, 2),
            server.pane_id.clone(),
        ));

        row_y = row_y.saturating_add(2);
        if row_y < body_bottom {
            row_y = row_y.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot_with(count: usize) -> ClientShellSnapshot {
        let mut snapshot = crate::client::shell::tests::snapshot();
        snapshot.dev_servers = (0..count)
            .map(|index| crate::protocol::ClientShellDevServer {
                pane_id: format!("w1:p{index}"),
                workspace_id: "w1".to_owned(),
                workspace_label: "api".to_owned(),
                tool: "vite".to_owned(),
                port: 3000 + index as u16,
            })
            .collect();
        snapshot
    }

    #[test]
    fn server_panel_never_starves_the_agent_panel() {
        let snapshot = snapshot_with(6);
        for height in 1..=24u16 {
            let area = Rect::new(0, 0, 30, height);
            let (agent, server) = split_detail_area(&snapshot, area);
            assert_eq!(agent.height + server.height, height, "height {height}");
            if server.height > 0 {
                assert!(
                    agent.height >= AGENT_PANEL_MIN_ROWS,
                    "agent panel starved at height {height}"
                );
                assert!(server.height > SERVER_PANEL_HEADER_ROWS, "stub panel drawn");
            }
        }
    }

    #[test]
    fn no_servers_leaves_the_whole_detail_area_to_agents() {
        let snapshot = snapshot_with(0);
        let area = Rect::new(0, 0, 30, 20);
        let (agent, server) = split_detail_area(&snapshot, area);
        assert_eq!(agent, area);
        assert_eq!(server.height, 0);
    }

    #[test]
    fn several_servers_in_one_pane_each_get_a_row() {
        let mut snapshot = snapshot_with(3);
        for server in &mut snapshot.dev_servers {
            server.pane_id = "w1:p1".to_owned();
        }
        // Tall enough that the half-area cap is not what limits the rows.
        let detail = Rect::new(0, 0, 30, 40);
        let mut buffer = Buffer::empty(detail);
        let mut hits = ShellHitMap::default();
        let config = ClientShellConfig::from_config(&crate::config::Config::default());
        let (_, server_area) = split_detail_area(&snapshot, detail);
        render_dev_server_panel(&mut buffer, server_area, &snapshot, &config, &mut hits);
        assert_eq!(hits.dev_servers.len(), 3);
    }
}
