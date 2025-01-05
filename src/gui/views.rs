mod unit_table;
mod unit_tree;

use cursive::{view::ViewWrapper, views::SelectView, wrap_impl, Cursive};
pub use unit_table::UnitTable;
pub use unit_tree::{UnitTree, UnitTreeSelection};

/// Same as `SelectView`, except it allows a sentinel value as a default
/// option if none is selected
pub struct DefaultSelectView<T = String> {
    view: SelectView<Option<T>>,
}

impl<T: Clone + Send + Sync + 'static> DefaultSelectView<T> {
    pub fn new(sentinel: &str) -> Self {
        let view = SelectView::new().item(sentinel, None);
        Self { view }
    }

    pub fn with_all<I>(mut self, items: I) -> Self
    where
        T: ToString,
        I: Iterator<Item = T>,
    {
        self.view.add_all(items.map(|i| (i.to_string(), Some(i))));
        self
    }

    pub fn popup(mut self) -> Self {
        self.view.set_popup(true);
        self
    }

    pub fn on_submit<F>(mut self, cb: F) -> Self
    where
        F: Fn(&mut Cursive, &Option<T>) + Send + Sync + 'static,
    {
        self.view.set_on_submit(cb);
        self
    }

    pub fn selection(&self) -> Option<T> {
        self.view.selection().and_then(|o| (*o).clone())
    }
}

impl<T: Send + Sync + 'static> ViewWrapper for DefaultSelectView<T> {
    wrap_impl!(self.view: SelectView<Option<T>>);
}
