use iced_native::{widget, Length, Padding, Pixels};

use crate::keyframes::{as_f32, get_length, Repeat};
use crate::timeline::Frame;
use crate::{Ease, Linear, MovementType};

/// A Column's animation Id. Used for linking animation built in `update()` with widget output in `view()`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(iced_native::widget::Id);

impl Id {
    /// Creates a custom [`Id`].
    pub fn new(id: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self(widget::Id::new(id))
    }

    /// Creates a unique [`Id`].
    ///
    /// This function produces a different [`Id`] every time it is called.
    pub fn unique() -> Self {
        Self(widget::Id::unique())
    }

    pub fn to_chain(self) -> Chain {
        Chain::new(self)
    }

    pub fn to_chain_with_children(self, children: Vec<Column>) -> Chain {
        Chain::with_children(self, children)
    }

    pub fn as_widget<'a, Message, Renderer>(
        self,
        timeline: &crate::Timeline,
    ) -> widget::Column<'a, Message, Renderer>
    where
        Renderer: iced_native::Renderer,
        Renderer::Theme: widget::container::StyleSheet,
    {
        Column::as_widget(self, timeline)
    }
}

impl From<Id> for widget::Id {
    fn from(id: Id) -> Self {
        id.0
    }
}

#[derive(Debug)]
pub struct Chain {
    id: Id,
    links: Vec<Column>,
    repeat: Repeat,
}

impl Chain {
    pub fn new(id: Id) -> Self {
        Chain {
            id,
            links: Vec::new(),
            repeat: Repeat::Never,
        }
    }

    pub fn with_children(id: Id, children: Vec<Column>) -> Self {
        Chain {
            id,
            links: children,
            repeat: Repeat::Never,
        }
    }

    pub fn link(mut self, container: Column) -> Self {
        self.links.push(container);
        self
    }

    pub fn loop_forever(mut self) -> Self {
        self.repeat = Repeat::Forever;
        self
    }

    pub fn loop_once(mut self) -> Self {
        self.repeat = Repeat::Never;
        self
    }
}

impl From<Chain> for crate::timeline::Chain {
    fn from(chain: Chain) -> Self {
        crate::timeline::Chain::new(
            chain.id.into(),
            chain.repeat,
            chain
                .links
                .into_iter()
                .map(|c| c.into())
                .collect::<Vec<_>>(),
        )
    }
}

#[must_use = "Keyframes are intended to be used in an animation chain."]
#[derive(Debug, Clone, Copy)]
pub struct Column {
    at: MovementType,
    ease: Ease,
    spacing: Option<f32>,
    padding: Option<Padding>,
    width: Option<Length>,
    height: Option<Length>,
    is_eager: bool,
}

impl Column {
    pub fn new(at: impl Into<MovementType>) -> Column {
        let at = at.into();
        Column {
            at,
            ease: Linear::InOut.into(),
            spacing: None,
            width: None,
            height: None,
            padding: None,
            is_eager: true,
        }
    }

    pub fn lazy(at: impl Into<MovementType>) -> Column {
        let at = at.into();
        Column {
            at,
            ease: Linear::InOut.into(),
            spacing: None,
            width: None,
            height: None,
            padding: None,
            is_eager: false,
        }
    }

    pub fn as_widget<'a, Message, Renderer>(
        id: Id,
        timeline: &crate::Timeline,
    ) -> widget::Column<'a, Message, Renderer>
    where
        Renderer: iced_native::Renderer,
        Renderer::Theme: widget::container::StyleSheet,
    {
        let id: widget::Id = id.into();

        widget::Column::new()
            .spacing(timeline.get(&id, 0).map(|m| m.value).unwrap_or(0.))
            .padding([
                timeline.get(&id, 1).map(|m| m.value).unwrap_or(0.),
                timeline.get(&id, 2).map(|m| m.value).unwrap_or(0.),
                timeline.get(&id, 3).map(|m| m.value).unwrap_or(0.),
                timeline.get(&id, 4).map(|m| m.value).unwrap_or(0.),
            ])
            .width(get_length(&id, timeline, 5, Length::Shrink))
            .height(get_length(&id, timeline, 6, Length::Shrink))
    }

    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = Some(spacing.into().0);
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = Some(padding.into());
        self
    }

    pub fn ease<E: Into<Ease>>(mut self, ease: E) -> Self {
        self.ease = ease.into();
        self
    }
}

#[rustfmt::skip]
impl From<Column> for Vec<Option<Frame>> {
    fn from(column: Column) -> Vec<Option<Frame>> {
      if column.is_eager {
        vec![column.spacing.map(|s| Frame::eager(column.at, s, column.ease)),        // 0 = spacing
             column.padding.map(|p| Frame::eager(column.at, p.top, column.ease)),    // 1 = padding[0] (top)
             column.padding.map(|p| Frame::eager(column.at, p.right, column.ease)),  // 2 = padding[1] (right)
             column.padding.map(|p| Frame::eager(column.at, p.bottom, column.ease)), // 3 = padding[2] (bottom)
             column.padding.map(|p| Frame::eager(column.at, p.left, column.ease)),   // 4 = padding[3] (left)
             as_f32(column.width).map(|w| Frame::eager(column.at, w, column.ease)),  // 5 = width
             as_f32(column.height).map(|h| Frame::eager(column.at, h, column.ease)), // 6 = height
        ]
      } else {
        vec![Some(Frame::lazy(column.at, 0., column.ease)); 7] // lazy evaluates for all values
      }
    }
}
