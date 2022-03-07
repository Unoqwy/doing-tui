use tui::layout::{Alignment, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::Paragraph;

pub fn list_position<'a>(
    area: Rect,
    position: usize,
    total: usize,
) -> Option<(Paragraph<'a>, Rect)> {
    let position = format!("{} of {}", usize::min(position, total), total);
    if area.width as usize > position.len() + 5 {
        let width = position.len() as u16;
        let position = Paragraph::new(position)
            .alignment(Alignment::Right)
            .style(Style::default().add_modifier(Modifier::DIM));
        let rect = Rect::new(
            area.x + area.width - 2 - width,
            area.y + area.height - 1,
            width,
            1,
        );
        Some((position, rect))
    } else {
        None
    }
}

pub fn overlay(area: Rect, mut height: u16, footer: bool) -> (Rect, Rect, Option<Rect>) {
    if footer {
        height += 1;
    }
    let height = u16::min(height, area.height);
    let width = 70 * area.width / 100;
    let area = Rect::new(
        (area.width - width) / 2,
        area.height / 2 - height / 2,
        width,
        if footer { height - 1 } else { height },
    );
    let (clear, footer) = if footer {
        let footer = Rect::new(area.x, area.y + area.height, area.width, 1);
        let clear = area.union(footer);
        (clear, Some(footer))
    } else {
        (area, None)
    };
    (area, clear, footer)
}

pub fn bindings<'a, B>(bindings: B) -> Paragraph<'a>
where
    B: Into<Vec<(&'a str, &'a str)>>,
{
    let text = bindings
        .into()
        .iter()
        .map(|(key, action)| format!("{}: {}", key, action))
        .collect::<Vec<String>>()
        .join(", ");
    let style = Style::default().fg(Color::Blue);
    Paragraph::new(text).style(style)
}

pub fn default_list_item<'a>(value: &'a str, selected: bool) -> Spans<'a> {
    let style = if selected {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    Spans::from(vec![Span::styled(value, style)])
}
