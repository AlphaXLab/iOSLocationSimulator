// Lucide icons (ISC). Paths are the upstream `d` attributes and primitives.
// https://github.com/lucide-icons/lucide
use super::Icon;

pub(super) enum Mark {
    D(&'static str),
    Circle(i32, i32, i32),
    /// Filled dot. Lucide's ellipsis marks are fills, not strokes.
    Dot(i32, i32, i32),
    Ellipse(i32, i32, i32, i32),
    Rect(i32, i32, i32, i32, i32),
}

const PLUS: &[Mark] = &[Mark::D("M5 12h14"), Mark::D("M12 5v14")];

const SEARCH: &[Mark] = &[Mark::D("m21 21-4.34-4.34"), Mark::Circle(11, 11, 8)];

const CHEVRONDOWN: &[Mark] = &[Mark::D("m6 9 6 6 6-6")];

const CHEVRONRIGHT: &[Mark] = &[Mark::D("m9 18 6-6-6-6")];

const CHEVRONLEFT: &[Mark] = &[Mark::D("m15 18-6-6 6-6")];

const CHEVRONS_UP_DOWN: &[Mark] = &[Mark::D("m7 15 5 5 5-5"), Mark::D("m7 9 5-5 5 5")];

const TABLE: &[Mark] = &[
    Mark::D("M12 3v18"),
    Mark::Rect(3, 3, 18, 18, 2),
    Mark::D("M3 9h18"),
    Mark::D("M3 15h18"),
];

const VIEW: &[Mark] = &[
    Mark::D("M12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83z"),
    Mark::D("M2 12a1 1 0 0 0 .58.91l8.6 3.91a2 2 0 0 0 1.65 0l8.58-3.9A1 1 0 0 0 22 12"),
    Mark::D("M2 17a1 1 0 0 0 .58.91l8.6 3.91a2 2 0 0 0 1.65 0l8.58-3.9A1 1 0 0 0 22 17"),
];

const FUNCTION: &[Mark] = &[
    Mark::Rect(3, 3, 18, 18, 2),
    Mark::D("M9 17c2 0 2.8-1 2.8-2.8V10c0-2 1-3.3 3.2-3"),
    Mark::D("M9 11.2h5.7"),
];

const SIDEBARLEFT: &[Mark] = &[Mark::Rect(3, 3, 18, 18, 2), Mark::D("M9 3v18")];

const SIDEBARRIGHT: &[Mark] = &[Mark::Rect(3, 3, 18, 18, 2), Mark::D("M15 3v18")];

const TRASH: &[Mark] = &[
    Mark::D("M10 11v6"),
    Mark::D("M14 11v6"),
    Mark::D("M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"),
    Mark::D("M3 6h18"),
    Mark::D("M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"),
];

const PLAY: &[Mark] = &[Mark::D(
    "M5 5a2 2 0 0 1 3.008-1.728l11.997 6.998a2 2 0 0 1 .003 3.458l-12 7A2 2 0 0 1 5 19z",
)];

const REFRESH: &[Mark] = &[
    Mark::D("M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"),
    Mark::D("M21 3v5h-5"),
    Mark::D("M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"),
    Mark::D("M8 16H3v5"),
];

const BACKUP: &[Mark] = &[
    Mark::Ellipse(12, 5, 9, 3),
    Mark::D("M3 12a9 3 0 0 0 5 2.69"),
    Mark::D("M21 9.3V5"),
    Mark::D("M3 5v14a9 3 0 0 0 6.47 2.88"),
    Mark::D("M12 12v4h4"),
    Mark::D("M13 20a5 5 0 0 0 9-3 4.5 4.5 0 0 0-4.5-4.5c-1.33 0-2.54.54-3.41 1.41L12 16"),
];

const RESTORE: &[Mark] = &[
    Mark::Rect(2, 3, 20, 5, 1),
    Mark::D("M4 8v11a2 2 0 0 0 2 2h2"),
    Mark::D("M20 8v11a2 2 0 0 1-2 2h-2"),
    Mark::D("m9 15 3-3 3 3"),
    Mark::D("M12 12v9"),
];

const PLUG: &[Mark] = &[
    Mark::D("M12 22v-5"),
    Mark::D("M15 8V2"),
    Mark::D("M17 8a1 1 0 0 1 1 1v4a4 4 0 0 1-4 4h-4a4 4 0 0 1-4-4V9a1 1 0 0 1 1-1z"),
    Mark::D("M9 8V2"),
];

const SUN: &[Mark] = &[
    Mark::Circle(12, 12, 4),
    Mark::D("M12 2v2"),
    Mark::D("M12 20v2"),
    Mark::D("m4.93 4.93 1.41 1.41"),
    Mark::D("m17.66 17.66 1.41 1.41"),
    Mark::D("M2 12h2"),
    Mark::D("M20 12h2"),
    Mark::D("m6.34 17.66-1.41 1.41"),
    Mark::D("m19.07 4.93-1.41 1.41"),
];

const MOON: &[Mark] = &[
    Mark::D("M20.985 12.486a9 9 0 1 1-9.473-9.472c.405-.022.617.46.402.803a6 6 0 0 0 8.268 8.268c.344-.215.825-.004.803.401"),
];

const CLOSE: &[Mark] = &[Mark::D("M18 6 6 18"), Mark::D("m6 6 12 12")];

const HOME: &[Mark] = &[
    Mark::D("M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"),
    Mark::D("M3 10a2 2 0 0 1 .709-1.528l7-6a2 2 0 0 1 2.582 0l7 6A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"),
];

const QUERY: &[Mark] = &[Mark::D("m16 18 6-6-6-6"), Mark::D("m8 6-6 6 6 6")];

const IMPORT: &[Mark] = &[
    Mark::D("M12 3v12"),
    Mark::D("m8 11 4 4 4-4"),
    Mark::D("M8 5H4a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-4"),
];

const SLIDERS: &[Mark] = &[
    Mark::D("M10 5H3"),
    Mark::D("M12 19H3"),
    Mark::D("M14 3v4"),
    Mark::D("M16 17v4"),
    Mark::D("M21 12h-9"),
    Mark::D("M21 19h-5"),
    Mark::D("M21 5h-7"),
    Mark::D("M8 10v4"),
    Mark::D("M8 12H3"),
];

const CIRCLEX: &[Mark] = &[
    Mark::Circle(12, 12, 10),
    Mark::D("m15 9-6 6"),
    Mark::D("m9 9 6 6"),
];

const EYE: &[Mark] = &[
    Mark::D("M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0"),
    Mark::Circle(12, 12, 3),
];

const LOCK: &[Mark] = &[
    Mark::Rect(3, 11, 18, 11, 2),
    Mark::D("M7 11V7a5 5 0 0 1 10 0v4"),
];

const UNLOCK: &[Mark] = &[
    Mark::Rect(3, 11, 18, 11, 2),
    Mark::D("M7 11V7a5 5 0 0 1 9.9-1"),
];

const DATABASE: &[Mark] = &[
    Mark::Ellipse(12, 5, 9, 3),
    Mark::D("M3 5V19A9 3 0 0 0 21 19V5"),
    Mark::D("M3 12A9 3 0 0 0 21 12"),
];

const LIST_ARROW: &[Mark] = &[
    Mark::D("M4 7h8"),
    Mark::D("M4 12h6"),
    Mark::D("M4 17h8"),
    Mark::D("M14 12h6"),
    Mark::D("M17 9l3 3-3 3"),
];

const BARCHART: &[Mark] = &[
    Mark::D("M3 3v16a2 2 0 0 0 2 2h16"),
    Mark::D("M18 17V9"),
    Mark::D("M13 17V5"),
    Mark::D("M8 17v-3"),
];

const MORE: &[Mark] = &[
    Mark::Dot(12, 5, 1),
    Mark::Dot(12, 12, 1),
    Mark::Dot(12, 19, 1),
];

const PANEL_BOTTOM: &[Mark] = &[Mark::Rect(3, 3, 18, 18, 2), Mark::D("M3 15h18")];

const SPLIT: &[Mark] = &[Mark::Rect(3, 3, 18, 18, 2), Mark::D("M12 3v18")];

pub(super) fn marks(icon: Icon) -> &'static [Mark] {
    match icon {
        Icon::Plus => PLUS,
        Icon::Search => SEARCH,
        Icon::ChevronDown => CHEVRONDOWN,
        Icon::ChevronRight => CHEVRONRIGHT,
        Icon::ChevronLeft => CHEVRONLEFT,
        Icon::ChevronsUpDown => CHEVRONS_UP_DOWN,
        Icon::Table => TABLE,
        Icon::View => VIEW,
        Icon::Function => FUNCTION,
        Icon::SidebarLeft => SIDEBARLEFT,
        Icon::SidebarRight => SIDEBARRIGHT,
        Icon::Trash => TRASH,
        Icon::Play => PLAY,
        Icon::Refresh => REFRESH,
        Icon::Backup => BACKUP,
        Icon::Restore => RESTORE,
        Icon::Plug => PLUG,
        Icon::Sun => SUN,
        Icon::Moon => MOON,
        Icon::Close => CLOSE,
        Icon::Home => HOME,
        Icon::Query => QUERY,
        Icon::Import => IMPORT,
        Icon::Sliders => SLIDERS,
        Icon::CircleX => CIRCLEX,
        Icon::Eye => EYE,
        Icon::Lock => LOCK,
        Icon::Unlock => UNLOCK,
        Icon::Database => DATABASE,
        Icon::ListArrow => LIST_ARROW,
        Icon::BarChart => BARCHART,
        Icon::More => MORE,
        Icon::PanelBottom => PANEL_BOTTOM,
        Icon::Split => SPLIT,
    }
}
