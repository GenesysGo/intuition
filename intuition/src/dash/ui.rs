use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{Axis, Block, Borders, Chart, Dataset, List, ListItem, Tabs},
    Frame,
};

use crate::{profiler::ProfilerExt, timer::LogLevel};

use super::Dash;

const INFO_LOG_STYLE: Style = Style {
    fg: Some(Color::Blue),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};
const WARN_LOG_STYLE: Style = Style {
    fg: Some(Color::Yellow),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};
const ERROR_LOG_STYLE: Style = Style {
    fg: Some(Color::Red),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

pub fn draw<B, P>(f: &mut Frame<B>, dash: &mut Dash<P>)
where
    B: Backend,
    P: ProfilerExt,
{
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());
    let titles = dash
        .tabs
        .titles
        .iter()
        .map(|t| Spans::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(P::TITLE))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(dash.tabs.index);

    // Render small widget for displaying different tabs
    f.render_widget(tabs, chunks[0]);

    // Render particular tab
    match dash.tabs.index {
        0 => draw_first_tab(f, dash, chunks[1]),
        1 => {} //draw_second_tab(f, dash, chunks[1]),
        2 => {} //draw_third_tab(f, dash, chunks[1]),
        _ => {}
    };
}

fn draw_first_tab<B, P>(f: &mut Frame<B>, dash: &mut Dash<P>, area: Rect)
where
    B: Backend,
    P: ProfilerExt,
{
    // let chunks = Layout::default()
    //     .constraints(
    //         [
    //             Constraint::Length(9),
    //             Constraint::Min(8),
    //             Constraint::Length(7),
    //         ]
    //         .as_ref(),
    //     )
    //     .split(area);
    // draw_gauges(f, dash, chunks[0]);
    // draw_charts(f, dash, chunks[1]);
    // draw_text(f, chunks[2]);

    draw_charts(f, dash, area);
}

// fn draw_gauges<B, P>(f: &mut Frame<B>, dash: &mut Dash<P>, area: Rect)
// where
//     B: Backend,
//     P: ProfilerExt,
// {
//     let chunks = Layout::default()
//         .constraints(
//             [
//                 Constraint::Length(2),
//                 Constraint::Length(3),
//                 Constraint::Length(1),
//             ]
//             .as_ref(),
//         )
//         .margin(1)
//         .split(area);
//     let block = Block::default().borders(Borders::ALL).title("Graphs");
//     f.render_widget(block, area);

//     let label = format!("{:.2}%", app.progress * 100.0);
//     let gauge = Gauge::default()
//         .block(Block::default().title("Gauge:"))
//         .gauge_style(
//             Style::default()
//                 .fg(Color::Magenta)
//                 .bg(Color::Black)
//                 .add_modifier(Modifier::ITALIC | Modifier::BOLD),
//         )
//         .label(label)
//         .ratio(app.progress);
//     f.render_widget(gauge, chunks[0]);

//     // let sparkline = Sparkline::default()
//     //     .block(Block::default().title("Sparkline:"))
//     //     .style(Style::default().fg(Color::Green))
//     //     .data(&app.sparkline.points)
//     //     .bar_set(if app.enhanced_graphics {
//     //         symbols::bar::NINE_LEVELS
//     //     } else {
//     //         symbols::bar::THREE_LEVELS
//     //     });
//     // f.render_widget(sparkline, chunks[1]);

//     // let line_gauge = LineGauge::default()
//     //     .block(Block::default().title("LineGauge:"))
//     //     .gauge_style(Style::default().fg(Color::Magenta))
//     //     .line_set(if app.enhanced_graphics {
//     //         symbols::line::THICK
//     //     } else {
//     //         symbols::line::NORMAL
//     //     })
//     //     .ratio(app.progress);
//     // f.render_widget(line_gauge, chunks[2]);
// }

#[allow(unused_labels)]
fn draw_charts<B, P>(f: &mut Frame<B>, app: &mut Dash<P>, area: Rect)
where
    B: Backend,
    P: ProfilerExt,
{
    // If show log, we need to split horizontally
    let constraints = if app.show_log {
        vec![Constraint::Percentage(50), Constraint::Percentage(50)]
    } else {
        vec![Constraint::Percentage(100)]
    };

    // Chunks for profiler plots and log
    let chunks = Layout::default()
        .constraints(constraints)
        .direction(Direction::Horizontal)
        .split(area);
    'plot_scope: {
        // Get the chunks for each scope
        let percentage_per_scope = 100 / P::SCOPES.len() as u16;
        let chunks = Layout::default()
            .constraints(
                vec![Constraint::Percentage(percentage_per_scope); P::SCOPES.len()].as_ref(),
            )
            .split(chunks[0]);

        // Draw scope plots

        // Colors we cycle through
        const COLORS: [Color; 4] = [Color::Cyan, Color::Red, Color::Yellow, Color::Magenta];

        for chunk in 0..chunks.len() {
            // Scope for this chunk
            let (scope_name, averages) = &app.state_buffer[chunk];
            let max_average = *averages.iter().max().unwrap_or(&0) as f64;

            let pairs: Vec<(f64, f64)> = app
                .domain
                .clone()
                .into_iter()
                .zip(
                    averages
                        .into_iter()
                        .map(|a| *a as f64)
                        .collect::<Vec<f64>>(),
                )
		.filter(|(x, y)| *y > 0.00)
                .collect();

            let dataset: Dataset = Dataset::default()
                .name(*scope_name)
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(COLORS[chunk % COLORS.len()]))
                .data(&pairs);

            let x_labels = if P::NUM_AVERAGES > 50 {
                vec![
                    Span::styled(
                        format!("{}", P::NUM_AVERAGES as f64),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!("{}", P::NUM_AVERAGES as f64 / 2.0)),
                    Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                ]
            } else {
                vec![
                    Span::styled(
                        format!("{}", P::NUM_AVERAGES as f64),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                ]
            };

            let chart = Chart::new(vec![dataset])
                .block(
                    Block::default()
                        .title(Span::styled(
                            format!("{scope_name}"),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ))
                        .borders(Borders::ALL),
                )
                .x_axis(
                    Axis::default()
                        .title("History of Averages")
                        .style(Style::default().fg(Color::Gray))
                        .bounds([0.0, P::NUM_AVERAGES as f64])
                        .labels(x_labels),
                )
                .y_axis(
                    Axis::default()
                        .title("Average")
                        .style(Style::default().fg(Color::Gray))
                        .bounds([0.0, max_average * 1.5])
                        .labels(vec![
                            Span::raw("0"),
                            Span::raw(format!("{}", max_average * 0.5)),
                            Span::raw(format!("{}", max_average)),
                            Span::raw(format!("{}", max_average * 1.5)),
                        ]),
                );
            f.render_widget(chart, chunks[chunk]);
        }
    }

    // log scope
    if app.show_log {
        // // Draw logs
        // Get the chunks for each scope
        let percentage_per_scope = 100 / P::SCOPES.len() as u16;
        let chunks = Layout::default()
            .constraints(
                vec![Constraint::Percentage(percentage_per_scope); P::SCOPES.len()].as_ref(),
            )
            .split(chunks[1]);

        for chunk in 0..chunks.len() {
            let (scope_name, ref scope_logs) = &app.log_buffer[chunk];

            let logs: Vec<ListItem> = scope_logs[scope_logs.len().saturating_sub(100)..]
                .iter()
                .map(|log| {
                    let s = match log.level {
                        LogLevel::Error => ERROR_LOG_STYLE,
                        LogLevel::Warn => WARN_LOG_STYLE,
                        LogLevel::Info => INFO_LOG_STYLE,
                    };
                    let content = vec![Spans::from(vec![
                        Span::styled(format!("{:<9}", log.level), s),
                        Span::raw(log.log.clone()),
                    ])];
                    ListItem::new(content)
                })
                .collect();
            let logs =
                List::new(logs).block(Block::default().borders(Borders::ALL).title(*scope_name));
            // f.render_stateful_widget(logs, chunks[1], &mut app.logs.state);
            f.render_widget(logs, chunks[chunk]);
        }
    }
}
